use crate::aggregator::HarvestAggregator;
use crate::cgroups::parse_cgroup_memory_stat;
use crate::procfs::parse_statm;
use common::result::ResourceUsage;
use std::fs;
use std::path::{Path, PathBuf};
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
}

impl BackgroundSampler {
    pub fn spawn_target(
        target_pid: Option<u32>,
        cgroup_dir: Option<PathBuf>,
        interval: Duration,
    ) -> Self {
        let running = Arc::new(AtomicBool::new(true));
        let aggregator = Arc::new(Mutex::new(HarvestAggregator::default()));

        let run_flag = Arc::clone(&running);
        let agg_lock = Arc::clone(&aggregator);

        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            while run_flag.load(Ordering::Relaxed) {
                ticker.tick().await;
                let sample_rss = sample_rss_bytes(target_pid, cgroup_dir.as_deref());
                let mut agg = agg_lock.lock().await;
                agg.record_sample(sample_rss);
            }
        });

        Self {
            running,
            aggregator,
            handle: Some(handle),
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
        }
    }

    pub async fn stop(mut self) -> ResourceUsage {
        self.running.store(false, Ordering::Relaxed);
        if let Some(h) = self.handle.take() {
            let _ = h.await;
        }
        let agg = self.aggregator.lock().await;
        agg.clone().finalize()
    }
}

fn sample_rss_bytes(target_pid: Option<u32>, cgroup_dir: Option<&Path>) -> u64 {
    if let Some(cg) = cgroup_dir {
        let mem_path = cg.join("memory.current");
        if let Ok(content) = fs::read_to_string(mem_path) {
            if let Ok(bytes) = parse_cgroup_memory_stat(&content) {
                return bytes;
            }
        }
    }

    let pid_str = target_pid.map_or_else(|| "self".to_string(), |p| p.to_string());
    let statm_path = format!("/proc/{pid_str}/statm");
    if let Ok(content) = fs::read_to_string(&statm_path) {
        if let Ok(bytes) = parse_statm(&content, 4096) {
            return bytes;
        }
    }

    0
}
