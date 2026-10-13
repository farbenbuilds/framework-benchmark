use crate::aggregator::HarvestAggregator;
use crate::cgroups::parse_cgroup_memory_stat;
use crate::procfs::parse_statm;
use crate::rapl::{RaplZoneSnapshot, energy_delta_joules, read_rapl_snapshot};
use common::result::ResourceUsage;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct BackgroundSampler {
    running: Arc<AtomicBool>,
    aggregator: Arc<Mutex<HarvestAggregator>>,
    handle: Option<JoinHandle<()>>,
    rapl_start: Vec<RaplZoneSnapshot>,
}

#[derive(Debug)]
struct RssSource {
    file: Option<File>,
    is_cgroup: bool,
}

impl BackgroundSampler {
    pub fn spawn_target(
        target_pid: Option<u32>,
        cgroup_dir: Option<&Path>,
        interval: Duration,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let aggregator = Arc::new(Mutex::new(HarvestAggregator::default()));

        let run_flag = Arc::clone(&running);
        let agg_lock = Arc::clone(&aggregator);

        let mut source = open_rss_source(target_pid, cgroup_dir);
        let rapl_start = read_rapl_snapshot(Path::new("/sys/class/powercap")).unwrap_or_default();
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            while run_flag.load(Ordering::Relaxed) {
                ticker.tick().await;
                let sample_rss = sample_rss_bytes(&mut source);
                let mut agg = agg_lock.lock().await;
                agg.record_sample(sample_rss);
            }
        });

        Self {
            running,
            aggregator,
            handle: Some(handle),
            rapl_start,
        }
    }

    pub fn spawn_simulated(interval: Duration, static_rss: u64) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let aggregator = Arc::new(Mutex::new(HarvestAggregator::default()));

        let run_flag = Arc::clone(&running);
        let agg_lock = Arc::clone(&aggregator);

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            while run_flag.load(Ordering::Relaxed) {
                ticker.tick().await;
                let mut agg = agg_lock.lock().await;
                agg.record_sample(static_rss);
            }
        });

        Self {
            running,
            aggregator,
            handle: Some(handle),
            rapl_start: Vec::new(),
        }
    }

    pub async fn stop(mut self) -> ResourceUsage {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.await;
        }
        let agg = self.aggregator.lock().await;
        let mut usage = agg.clone().finalize();
        let rapl_end = read_rapl_snapshot(Path::new("/sys/class/powercap")).unwrap_or_default();
        if !self.rapl_start.is_empty() && !rapl_end.is_empty() {
            usage.rapl_joules = Some(energy_delta_joules(&self.rapl_start, &rapl_end));
        }
        usage
    }
}

fn open_rss_source(target_pid: Option<u32>, cgroup_dir: Option<&Path>) -> RssSource {
    if let Some(cg) = cgroup_dir {
        let mem_path = cg.join("memory.current");
        if let Ok(file) = File::open(mem_path) {
            return RssSource {
                file: Some(file),
                is_cgroup: true,
            };
        }
    }

    let pid_str = target_pid.map_or_else(|| "self".to_string(), |p| p.to_string());
    let statm_path = format!("/proc/{pid_str}/statm");
    RssSource {
        file: File::open(statm_path).ok(),
        is_cgroup: false,
    }
}

fn sample_rss_bytes(source: &mut RssSource) -> u64 {
    let Some(file) = source.file.as_mut() else {
        return 0;
    };
    let mut buffer = [0u8; 128];
    if file.seek(SeekFrom::Start(0)).is_err() {
        return 0;
    }
    let Ok(count) = file.read(&mut buffer) else {
        return 0;
    };
    let Ok(content) = std::str::from_utf8(&buffer[..count]) else {
        return 0;
    };
    if source.is_cgroup {
        return parse_cgroup_memory_stat(content).unwrap_or(0);
    }
    parse_statm(content, 4096).unwrap_or(0)
}
