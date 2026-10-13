use common::result::{LibcVariant, RunResult, TestMetrics};
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
#[derive(Debug, Clone, PartialEq)]
pub struct DualLibcDelta {
    pub framework: String,
    pub framework_version: String,
    pub concurrency: u32,
    pub glibc_requests_per_sec: f64,
    pub musl_requests_per_sec: f64,
    pub requests_per_sec_change_pct: Option<f64>,
    pub glibc_p99_latency_ms: f64,
    pub musl_p99_latency_ms: f64,
    pub p99_latency_change_pct: Option<f64>,
}
pub fn aggregate_leaderboard(results: &[RunResult], category: &str) -> Vec<LeaderboardRow> {
    let mut entries = Vec::new();

    for run in results {
        for test in &run.benchmarks {
            if test.category == category {
                let libc_str = libc_name(run.libc);
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
pub fn dual_libc_deltas(results: &[RunResult], category: &str) -> Vec<DualLibcDelta> {
    let glibc = category_metrics(results, category, LibcVariant::Glibc);
    let musl = category_metrics(results, category, LibcVariant::Musl);
    let mut paired = vec![false; musl.len()];
    let mut deltas = Vec::new();

    for (glibc_run, glibc_test) in glibc {
        let Some((index, (_musl_run, musl_test))) =
            musl.iter().enumerate().find(|(index, item)| {
                !paired[*index] && compatible_runs(glibc_run, glibc_test, item.0, item.1)
            })
        else {
            continue;
        };
        paired[index] = true;
        deltas.push(DualLibcDelta {
            framework: glibc_run.framework_name.clone(),
            framework_version: glibc_run.framework_version.clone(),
            concurrency: glibc_test.concurrency,
            glibc_requests_per_sec: glibc_test.requests_per_sec,
            musl_requests_per_sec: musl_test.requests_per_sec,
            requests_per_sec_change_pct: percentage_change(
                glibc_test.requests_per_sec,
                musl_test.requests_per_sec,
            ),
            glibc_p99_latency_ms: microseconds_to_milliseconds(glibc_test.latency.p99_us),
            musl_p99_latency_ms: microseconds_to_milliseconds(musl_test.latency.p99_us),
            p99_latency_change_pct: percentage_change(
                glibc_test.latency.p99_us as f64,
                musl_test.latency.p99_us as f64,
            ),
        });
    }
    deltas.sort_by(|a, b| {
        a.framework
            .cmp(&b.framework)
            .then_with(|| a.framework_version.cmp(&b.framework_version))
            .then_with(|| a.concurrency.cmp(&b.concurrency))
    });
    deltas
}
fn category_metrics<'a>(
    results: &'a [RunResult],
    category: &'a str,
    libc: LibcVariant,
) -> Vec<(&'a RunResult, &'a TestMetrics)> {
    results
        .iter()
        .filter(|run| run.libc == libc)
        .flat_map(|run| {
            run.benchmarks
                .iter()
                .filter(move |test| test.category == category)
                .map(move |test| (run, test))
        })
        .collect()
}
fn compatible_runs(
    left: &RunResult,
    left_test: &TestMetrics,
    right: &RunResult,
    right_test: &TestMetrics,
) -> bool {
    left.framework_name == right.framework_name
        && left.framework_version == right.framework_version
        && left.machine == right.machine
        && left_test.concurrency == right_test.concurrency
        && left_test.duration_secs == right_test.duration_secs
        && left_test.traffic == right_test.traffic
        && compatible_provenance(left, right)
}
fn compatible_provenance(left: &RunResult, right: &RunResult) -> bool {
    left.provenance.revision != "unknown"
        && left.provenance.revision == right.provenance.revision
        && left.provenance.framework_input_sha256.is_some()
        && left.provenance.framework_input_sha256 == right.provenance.framework_input_sha256
        && left.provenance.image_id.is_some()
        && right.provenance.image_id.is_some()
        && left.provenance.target_cpus == right.provenance.target_cpus
        && left.provenance.load_cpus == right.provenance.load_cpus
}

fn percentage_change(baseline: f64, value: f64) -> Option<f64> {
    (baseline != 0.0).then(|| ((value - baseline) / baseline) * 100.0)
}

fn libc_name(libc: LibcVariant) -> &'static str {
    match libc {
        LibcVariant::Glibc => "glibc",
        LibcVariant::Musl => "musl",
    }
}

fn microseconds_to_milliseconds(value: u64) -> f64 {
    (value as f64) / 1000.0
}

fn bytes_to_megabytes(value: u64) -> f64 {
    (value as f64) / (1024.0 * 1024.0)
}
