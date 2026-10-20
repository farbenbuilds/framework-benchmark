use crate::aggregator::HarvestAggregator;
use crate::metrics::{RuntimeCounters, apply_runtime_counters};
use crate::rapl::{RaplZoneSnapshot, energy_delta_joules, read_rapl_snapshot};
use common::result::ResourceUsage;
use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::MissedTickBehavior;

mod sources;

use sources::{CounterPaths, counter_paths, open_sources, read_counters, sample_memory};

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
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
            while run_flag.load(Ordering::Relaxed) {
                ticker.tick().await;
                let (memory_bytes, _) = sample_memory(&mut sources);
                agg_lock.lock().await.record_sample(memory_bytes);
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
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);
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
        self.stop_task().await;
        let mut usage = self.aggregator.lock().await.clone().finalize();
        let elapsed_usec = u64::try_from(self.started.elapsed().as_micros()).unwrap_or(u64::MAX);
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

    async fn stop_task(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        let Some(handle) = self.handle.take() else {
            return;
        };
        handle.abort();
        let _ = handle.await;
    }
}

impl Drop for BackgroundSampler {
    fn drop(&mut self) {
        self.running.store(false, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            handle.abort();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BackgroundSampler;
    use std::sync::Arc;
    use std::sync::atomic::Ordering;
    use std::time::Duration;

    #[tokio::test]
    async fn drop_aborts_sampling_task() {
        let sampler = BackgroundSampler::spawn_simulated(Duration::from_secs(60), 1024);
        let running = Arc::clone(&sampler.running);
        let abort = sampler
            .handle
            .as_ref()
            .map(tokio::task::JoinHandle::abort_handle);
        drop(sampler);

        assert!(!running.load(Ordering::Relaxed));
        let abort = abort.expect("sampler task handle");
        let wait = async {
            while !abort.is_finished() {
                tokio::task::yield_now().await;
            }
        };
        tokio::time::timeout(Duration::from_secs(1), wait)
            .await
            .expect("aborted sampler task should finish");
    }

    #[tokio::test]
    async fn stop_task_is_idempotent() {
        let mut sampler = BackgroundSampler::spawn_simulated(Duration::from_secs(60), 1024);
        sampler.stop_task().await;
        sampler.stop_task().await;
        assert!(!sampler.running.load(Ordering::Relaxed));
    }
}
