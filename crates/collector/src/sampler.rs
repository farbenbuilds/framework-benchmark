use crate::aggregator::HarvestAggregator;
use common::result::ResourceUsage;
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
