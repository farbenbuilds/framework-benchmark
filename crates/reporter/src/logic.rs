use common::result::RunResult;

#[derive(Debug, Clone)]
pub struct LeaderboardRow {
    pub rank: usize,
    pub framework: String,
    pub framework_version: String,
    pub libc: String,
    pub requests_per_sec: f64,
    pub p99_latency_ms: f64,
    pub peak_rss_mb: f64,
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
                let p99_ms = (test.latency.p99_us as f64) / 1000.0;
                let rss_mb = (test.resources.peak_rss_bytes as f64) / (1024.0 * 1024.0);

                entries.push(LeaderboardRow {
                    rank: 0,
                    framework: run.framework_name.clone(),
                    framework_version: run.framework_version.clone(),
                    libc: libc_str.into(),
                    requests_per_sec: test.requests_per_sec,
                    p99_latency_ms: p99_ms,
                    peak_rss_mb: rss_mb,
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
