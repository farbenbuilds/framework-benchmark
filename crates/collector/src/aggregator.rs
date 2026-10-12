use common::result::ResourceUsage;

#[derive(Debug, Default, Clone)]
pub struct HarvestAggregator {
    pub peak_rss_bytes: u64,
    pub rss_sum_bytes: u64,
    pub sample_count: u64,
    pub cpu_user_pct: f32,
    pub cpu_kernel_pct: f32,
    pub context_switches: u64,
}

impl HarvestAggregator {
    pub fn record_sample(&mut self, rss_bytes: u64) {
        if rss_bytes > self.peak_rss_bytes {
            self.peak_rss_bytes = rss_bytes;
        }
        self.rss_sum_bytes = self.rss_sum_bytes.saturating_add(rss_bytes);
        self.sample_count += 1;
    }

    pub fn finalize(self) -> ResourceUsage {
        let avg_rss = if self.sample_count == 0 {
            0
        } else {
            self.rss_sum_bytes / self.sample_count
        };

        ResourceUsage {
            peak_rss_bytes: self.peak_rss_bytes,
            avg_rss_bytes: avg_rss,
            cpu_user_pct: self.cpu_user_pct,
            cpu_kernel_pct: self.cpu_kernel_pct,
            context_switches: self.context_switches,
            rapl_joules: None,
        }
    }
}
