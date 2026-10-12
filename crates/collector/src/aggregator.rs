use common::result::ResourceUsage;

#[derive(Debug, Default, Clone)]
pub struct HarvestAggregator {
    pub peak_rss_bytes: u64,
    pub samples: Vec<u64>,
    pub cpu_user_pct: f32,
    pub cpu_kernel_pct: f32,
    pub context_switches: u64,
}

impl HarvestAggregator {
    pub fn record_sample(&mut self, rss_bytes: u64) {
        if rss_bytes > self.peak_rss_bytes {
            self.peak_rss_bytes = rss_bytes;
        }
        self.samples.push(rss_bytes);
    }

    pub fn finalize(self) -> ResourceUsage {
        let avg_rss = if self.samples.is_empty() {
            0
        } else {
            let sum: u64 = self.samples.iter().sum();
            sum / (self.samples.len() as u64)
        };

        ResourceUsage {
            peak_rss_bytes: self.peak_rss_bytes,
            avg_rss_bytes: avg_rss,
            cpu_user_pct: self.cpu_user_pct,
            cpu_kernel_pct: self.cpu_kernel_pct,
            context_switches: self.context_switches,
        }
    }
}
