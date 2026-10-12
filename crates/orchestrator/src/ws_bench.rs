use collector::BackgroundSampler;
use common::result::TestMetrics;
use load_gen::autobahn::AutobahnSuiteRunner;
use load_gen::histogram::LatencyRecorder;
use load_gen::stats::WorkerStats;
use std::path::Path;
use std::time::{Duration, Instant};

pub async fn execute_websocket_workload(
    target_addr: &str,
    endpoint_path: &str,
    concurrency: u32,
    duration_secs: u32,
    target_pid: Option<u32>,
    cgroup_dir: Option<&Path>,
) -> Result<TestMetrics, crate::error::OrchestratorError> {
    let sampler = BackgroundSampler::spawn_target(
        target_pid,
        cgroup_dir.map(Path::to_path_buf),
        Duration::from_millis(50),
    );

    let deadline = Instant::now() + Duration::from_secs(u64::from(duration_secs));
    let mut recorder = LatencyRecorder::default();
    let mut stats = WorkerStats::default();

    let _ = AutobahnSuiteRunner::run_echo_benchmark(
        target_addr,
        endpoint_path,
        deadline,
        &mut recorder,
        &mut stats,
    )
    .await;

    let resources = sampler.stop().await;
    let rps = stats.requests_completed as f64 / f64::from(duration_secs.max(1));
    let mb_per_sec =
        (stats.bytes_transferred as f64 / (1024.0 * 1024.0)) / f64::from(duration_secs.max(1));

    Ok(TestMetrics {
        category: "websocket".into(),
        concurrency,
        duration_secs,
        requests_total: stats.requests_completed,
        requests_per_sec: rps,
        transfer_mb_per_sec: mb_per_sec,
        latency: recorder.export_distribution(),
        resources,
        status_codes: stats.to_status_code_counts(),
    })
}
