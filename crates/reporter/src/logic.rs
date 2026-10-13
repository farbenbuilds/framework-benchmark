use common::result::RunResult;

#[derive(Debug, Clone)]
pub struct LeaderboardRow {
    pub rank: usize,
    pub framework: String,
    pub framework_version: String,
    pub libc: String,
    pub concurrency: u32,
    pub duration_secs: u32,
    pub requests_total: u64,
    pub requests_per_sec: f64,
    pub transfer_mb_per_sec: f64,
    pub min_latency_ms: f64,
    pub p50_latency_ms: f64,
    pub p75_latency_ms: f64,
    pub p90_latency_ms: f64,
    pub p99_latency_ms: f64,
    pub p99_9_latency_ms: f64,
    pub p99_99_latency_ms: f64,
    pub max_latency_ms: f64,
    pub latency_std_dev_ms: f64,
    pub peak_rss_mb: f64,
    pub avg_rss_mb: f64,
    pub cpu_user_pct: f32,
    pub cpu_kernel_pct: f32,
    pub context_switches: u64,
    pub rapl_joules: Option<f64>,
    pub http_2xx: u64,
    pub http_4xx: u64,
    pub http_5xx: u64,
    pub errors: u64,
}

pub fn aggregate_leaderboard(results: &[RunResult], category: &str) -> Vec<LeaderboardRow> {
    let mut entries = Vec::new();

    for run in results {
        for test in &run.benchmarks {
            if test.category == category {
                let libc_str = match run.libc {
                    common::result::LibcVariant::Glibc => "glibc",
                    common::result::LibcVariant::Musl => "musl",
                };
                entries.push(LeaderboardRow {
                    rank: 0,
                    framework: run.framework_name.clone(),
                    framework_version: run.framework_version.clone(),
                    libc: libc_str.into(),
                    concurrency: test.concurrency,
                    duration_secs: test.duration_secs,
                    requests_total: test.requests_total,
                    requests_per_sec: test.requests_per_sec,
                    transfer_mb_per_sec: test.transfer_mb_per_sec,
                    min_latency_ms: microseconds_to_milliseconds(test.latency.min_us),
                    p50_latency_ms: microseconds_to_milliseconds(test.latency.p50_us),
                    p75_latency_ms: microseconds_to_milliseconds(test.latency.p75_us),
                    p90_latency_ms: microseconds_to_milliseconds(test.latency.p90_us),
                    p99_latency_ms: microseconds_to_milliseconds(test.latency.p99_us),
                    p99_9_latency_ms: microseconds_to_milliseconds(test.latency.p99_9_us),
                    p99_99_latency_ms: microseconds_to_milliseconds(test.latency.p99_99_us),
                    max_latency_ms: microseconds_to_milliseconds(test.latency.max_us),
                    latency_std_dev_ms: test.latency.std_dev_us / 1000.0,
                    peak_rss_mb: bytes_to_megabytes(test.resources.peak_rss_bytes),
                    avg_rss_mb: bytes_to_megabytes(test.resources.avg_rss_bytes),
                    cpu_user_pct: test.resources.cpu_user_pct,
                    cpu_kernel_pct: test.resources.cpu_kernel_pct,
                    context_switches: test.resources.context_switches,
                    rapl_joules: test.resources.rapl_joules,
                    http_2xx: test.status_codes.http_2xx,
                    http_4xx: test.status_codes.http_4xx,
                    http_5xx: test.status_codes.http_5xx,
                    errors: test.status_codes.errors,
                });
            }
        }
    }

    entries.sort_by(|a, b| {
        b.requests_per_sec
            .total_cmp(&a.requests_per_sec)
            .then_with(|| a.p99_latency_ms.total_cmp(&b.p99_latency_ms))
            .then_with(|| a.framework.cmp(&b.framework))
            .then_with(|| a.framework_version.cmp(&b.framework_version))
            .then_with(|| a.libc.cmp(&b.libc))
    });

    for (idx, row) in entries.iter_mut().enumerate() {
        row.rank = idx + 1;
    }

    entries
}

fn microseconds_to_milliseconds(value: u64) -> f64 {
    (value as f64) / 1000.0
}

fn bytes_to_megabytes(value: u64) -> f64 {
    (value as f64) / (1024.0 * 1024.0)
}

#[cfg(test)]
mod tests {
    use super::{bytes_to_megabytes, microseconds_to_milliseconds};

    #[test]
    fn microseconds_to_milliseconds_preserves_fractional_values() {
        assert!((microseconds_to_milliseconds(125) - 0.125).abs() < f64::EPSILON);
    }

    #[test]
    fn bytes_to_megabytes_converts_binary_megabytes() {
        assert!((bytes_to_megabytes(2 * 1024 * 1024) - 2.0).abs() < f64::EPSILON);
    }
}
