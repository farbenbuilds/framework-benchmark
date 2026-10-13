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
    let sampler =
        BackgroundSampler::spawn_target(target_pid, cgroup_dir, Duration::from_millis(50));

    let started = Instant::now();
    let deadline = started + Duration::from_secs(u64::from(duration_secs));
    let workers = concurrency.max(1);
    let mut handles = Vec::with_capacity(workers as usize);
    for _ in 0..workers {
        let addr = target_addr.to_string();
        let path = endpoint_path.to_string();
        handles.push(tokio::spawn(async move {
            let mut recorder = LatencyRecorder::default();
            let mut stats = WorkerStats::default();
            let _ = AutobahnSuiteRunner::run_echo_benchmark(
                &addr,
                &path,
                deadline,
                &mut recorder,
                &mut stats,
            )
            .await;
            (recorder, stats)
        }));
    }

    let mut recorder = LatencyRecorder::default();
    let mut stats = WorkerStats::default();
    for handle in handles {
        if let Ok((worker_recorder, worker_stats)) = handle.await {
            recorder.combine(&worker_recorder);
            stats.merge(&worker_stats);
        }
    }

    let resources = sampler.stop().await;
    let elapsed_secs = started.elapsed().as_secs_f64().max(0.001);
    let rps = stats.requests_completed as f64 / elapsed_secs;
    let mb_per_sec = (stats.bytes_transferred as f64 / (1024.0 * 1024.0)) / elapsed_secs;

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
