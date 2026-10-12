use crate::client::run_connection_worker;
use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::plan::BenchmarkPlan;
use crate::stats::WorkerStats;
use std::time::{Duration, Instant};

#[derive(Debug, Clone)]
pub struct BenchmarkExecutionResult {
    pub total_requests: u64,
    pub requests_per_sec: f64,
    pub transfer_mb_per_sec: f64,
    pub latency: LatencyRecorder,
    pub stats: WorkerStats,
}

pub async fn execute_benchmark(
    plan: BenchmarkPlan,
) -> Result<BenchmarkExecutionResult, LoadGenError> {
    let req_str = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: keep-alive\r\nUser-Agent: framework-benchmark\r\n\r\n",
        plan.path, plan.target_addr
    );
    let req_bytes = req_str.into_bytes();

    let deadline = Instant::now() + Duration::from_secs(plan.duration_secs);
    let mut handles = Vec::with_capacity(plan.concurrency);

    for _ in 0..plan.concurrency {
        let addr = plan.target_addr.clone();
        let bytes = req_bytes.clone();
        let depth = plan.pipeline_depth;

        let handle = tokio::spawn(async move {
            let mut recorder = LatencyRecorder::default();
            let mut stats = WorkerStats::default();
            let _ =
                run_connection_worker(&addr, &bytes, deadline, depth, &mut recorder, &mut stats)
                    .await;
            (recorder, stats)
        });
        handles.push(handle);
    }

    let mut final_recorder = LatencyRecorder::default();
    let mut final_stats = WorkerStats::default();

    for handle in handles {
        if let Ok((rec, st)) = handle.await {
            final_recorder.combine(&rec);
            final_stats.merge(&st);
        }
    }

    let duration_secs = plan.duration_secs as f64;
    let rps = final_stats.requests_completed as f64 / duration_secs;
    let mb_per_sec = (final_stats.bytes_transferred as f64 / (1024.0 * 1024.0)) / duration_secs;

    Ok(BenchmarkExecutionResult {
        total_requests: final_stats.requests_completed,
        requests_per_sec: rps,
        transfer_mb_per_sec: mb_per_sec,
        latency: final_recorder,
        stats: final_stats,
    })
}
