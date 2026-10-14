use crate::aggregator::HarvestAggregator;
use crate::cgroups::{CgroupFiles, open_cgroup_files, sample_cgroup_files};
use crate::metrics::{RuntimeCounters, apply_runtime_counters};
use crate::procfs::{parse_statm, parse_status_switches};
use crate::rapl::{RaplZoneSnapshot, energy_delta_joules, read_rapl_snapshot};
use common::result::ResourceUsage;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

const CGROUP_INTERVAL: Duration = Duration::from_millis(100);

#[derive(Debug)]
pub struct BackgroundSampler {
    running: Arc<AtomicBool>,
    aggregator: Arc<Mutex<HarvestAggregator>>,
    handle: Option<JoinHandle<()>>,
    rapl_start: Vec<RaplZoneSnapshot>,
    counters: CounterPaths,
    counter_start: RuntimeCounters,
    started: Instant,
}

#[derive(Debug)]
struct Sources {
    cgroups: Option<CgroupFiles>,
    statm: Option<File>,
}

#[derive(Debug, Default)]
struct CounterPaths {
    cpu_stat: Option<PathBuf>,
    status: Option<PathBuf>,
}

impl BackgroundSampler {
    pub fn spawn_target(
        target_pid: Option<u32>,
        cgroup_dir: Option<&Path>,
        interval: Duration,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let aggregator = Arc::new(Mutex::new(HarvestAggregator::default()));
        let counters = counter_paths(target_pid, cgroup_dir);
        let counter_start = read_counters(&counters);
        let run_flag = Arc::clone(&running);
        let agg_lock = Arc::clone(&aggregator);
        let mut sources = open_sources(target_pid, cgroup_dir);
        let interval = cgroup_dir.map_or(interval, |_| CGROUP_INTERVAL);
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            while run_flag.load(Ordering::Relaxed) {
                ticker.tick().await;
                let (memory_bytes, peak_bytes) = sample_memory(&mut sources);
                let mut agg = agg_lock.lock().await;
                agg.record_sample(memory_bytes);
                agg.peak_rss_bytes = agg.peak_rss_bytes.max(peak_bytes);
            }
        });
        Self {
            running,
            aggregator,
            handle: Some(handle),
            rapl_start: read_rapl_snapshot(Path::new("/sys/class/powercap")).unwrap_or_default(),
            counters,
            counter_start,
            started: Instant::now(),
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
                agg_lock.lock().await.record_sample(static_rss);
            }
        });
        Self {
            running,
            aggregator,
            handle: Some(handle),
            rapl_start: Vec::new(),
            counters: CounterPaths::default(),
            counter_start: RuntimeCounters::default(),
            started: Instant::now(),
        }
    }

    pub async fn stop(mut self) -> ResourceUsage {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.await;
        }
        let mut usage = self.aggregator.lock().await.clone().finalize();
        let elapsed_usec = self.started.elapsed().as_micros() as u64;
        apply_runtime_counters(
            &mut usage,
            self.counter_start,
            read_counters(&self.counters),
            elapsed_usec,
        );
        let rapl_end = read_rapl_snapshot(Path::new("/sys/class/powercap")).unwrap_or_default();
        if !self.rapl_start.is_empty() && !rapl_end.is_empty() {
            usage.rapl_joules = Some(energy_delta_joules(&self.rapl_start, &rapl_end));
        }
        usage
    }
}

fn counter_paths(target_pid: Option<u32>, cgroup_dir: Option<&Path>) -> CounterPaths {
    CounterPaths {
        cpu_stat: cgroup_dir.map(|path| path.join("cpu.stat")),
        status: target_pid.map(|pid| PathBuf::from(format!("/proc/{pid}/status"))),
    }
}

fn read_counters(paths: &CounterPaths) -> RuntimeCounters {
    let (user_usec, kernel_usec) = paths
        .cpu_stat
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| crate::cgroups::parse_cgroup_cpu_stat(&content))
        .unwrap_or_default();
    let context_switches = paths
        .status
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| parse_status_switches(&content))
        .map(|(voluntary, involuntary)| voluntary.saturating_add(involuntary))
        .unwrap_or_default();
    RuntimeCounters {
        user_usec,
        kernel_usec,
        context_switches,
    }
}

fn open_sources(target_pid: Option<u32>, cgroup_dir: Option<&Path>) -> Sources {
    let pid = target_pid.map_or_else(|| "self".to_owned(), |pid| pid.to_string());
    Sources {
        cgroups: cgroup_dir.map(open_cgroup_files),
        statm: File::open(format!("/proc/{pid}/statm")).ok(),
    }
}

fn sample_memory(sources: &mut Sources) -> (u64, u64) {
    let process_rss = read_statm(&mut sources.statm);
    let cgroup = sources.cgroups.as_mut().map(sample_cgroup_files);
    let current = cgroup.map_or(0, |sample| sample.current_bytes);
    let peak = cgroup.map_or(0, |sample| sample.peak_bytes);
    (process_rss.max(current), process_rss.max(peak))
}

fn read_statm(file: &mut Option<File>) -> u64 {
    let Some(file) = file.as_mut() else {
        return 0;
    };
    let mut buffer = [0; 128];
    if file.seek(SeekFrom::Start(0)).is_err() {
        return 0;
    }
    let Ok(count) = file.read(&mut buffer) else {
        return 0;
    };
    let Ok(content) = std::str::from_utf8(&buffer[..count]) else {
        return 0;
    };
    parse_statm(content, 4096).unwrap_or(0)
}
