use common::result::LatencyDistribution;
use hdrhistogram::Histogram;

#[derive(Clone)]
pub struct LatencyRecorder {
    histogram: Histogram<u64>,
}

impl LatencyRecorder {
    pub fn new() -> Result<Self, hdrhistogram::CreationError> {
        let histogram = Histogram::<u64>::new_with_bounds(1, 100_000_000, 3)?;
        Ok(Self { histogram })
    }

    #[inline]
    pub fn record_micros(&mut self, micros: u64) {
        let val = micros.clamp(1, 100_000_000);
        let _ = self.histogram.record(val);
    }

    #[inline]
    pub fn record_micros_corrected(&mut self, micros: u64, expected_interval_us: u64) {
        let val = micros.clamp(1, 100_000_000);
        if expected_interval_us > 0 {
            let _ = self.histogram.record_correct(val, expected_interval_us);
        } else {
            let _ = self.histogram.record(val);
        }
    }

    pub fn combine(&mut self, other: &Self) {
        let _ = self.histogram.add(&other.histogram);
    }

    pub fn export_distribution(&self) -> LatencyDistribution {
        LatencyDistribution {
            min_us: self.histogram.min(),
            p50_us: self.histogram.value_at_quantile(0.50),
            p75_us: self.histogram.value_at_quantile(0.75),
            p90_us: self.histogram.value_at_quantile(0.90),
            p99_us: self.histogram.value_at_quantile(0.99),
            p99_9_us: self.histogram.value_at_quantile(0.999),
            p99_99_us: self.histogram.value_at_quantile(0.9999),
            max_us: self.histogram.max(),
            std_dev_us: self.histogram.stdev(),
        }
    }
}

impl std::fmt::Debug for LatencyRecorder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LatencyRecorder")
            .field("entries", &self.histogram.len())
            .finish()
    }
}
