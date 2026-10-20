use crate::client::{preallocate_worker_buffers, run_connection_worker_with_buffers};
use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::plan::{BenchmarkPlan, ConnectionMode};
use crate::stats::WorkerStats;
use std::sync::Arc;
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
    plan.validate()?;
    let connection = match plan.connection_mode {
        ConnectionMode::KeepAlive => "keep-alive",
        ConnectionMode::Churn => "close",
    };
    let req_str = format!(
        "GET {} HTTP/1.1\r\nHost: {}\r\nConnection: {connection}\r\nUser-Agent: framework-benchmark\r\n\r\n",
        plan.path, plan.target_addr,
    );
    let req_bytes = Arc::<[u8]>::from(req_str.into_bytes());
    let target_addr: Arc<str> = plan.target_addr.into();
    let concurrency = plan.concurrency;
    let expected_interval_us = match plan.target_rps {
        Some(rps) => (u64::try_from(concurrency)
            .unwrap_or(u64::MAX)
            .saturating_mul(1_000_000)
            / rps)
            .max(1),
        _ => 0,
    };
    let mut worker_buffers = Vec::with_capacity(concurrency);
    let mut recorders = Vec::with_capacity(concurrency);
    for _ in 0..concurrency {
        worker_buffers.push(preallocate_worker_buffers(plan.pipeline_depth));
        recorders.push(LatencyRecorder::new()?);
    }
    let start_instant = Instant::now();
    let deadline = start_instant + Duration::from_secs(plan.duration_secs);
    let mut handles = Vec::with_capacity(concurrency);

    for (buffers, recorder) in worker_buffers.into_iter().zip(recorders) {
        let addr = Arc::clone(&target_addr);
        let bytes = Arc::clone(&req_bytes);
        let depth = plan.pipeline_depth;
        let connection_mode = plan.connection_mode;
        let interval = expected_interval_us;

        let handle = tokio::spawn(async move {
            let mut recorder = recorder;
            let mut stats = WorkerStats::default();
            let result = run_connection_worker_with_buffers(
                &addr,
                &bytes,
                deadline,
                depth,
                connection_mode,
                interval,
                &mut recorder,
                &mut stats,
                buffers,
            )
            .await;
            if result.is_err() && Instant::now() < deadline {
                stats.record_error();
            }
            (recorder, stats)
        });
        handles.push(handle);
    }

    let mut final_recorder = LatencyRecorder::new()?;
    let mut final_stats = WorkerStats::default();

    for handle in handles {
        match handle.await {
            Ok((rec, stats)) => {
                final_recorder.combine(&rec);
                final_stats.merge(&stats);
            }
            Err(_) => final_stats.record_error(),
        }
    }

    if final_stats.requests_completed == 0 {
        return Err(LoadGenError::Protocol(
            "load generator completed no responses".into(),
        ));
    }

    let actual_duration_secs = start_instant.elapsed().as_secs_f64().max(0.001);
    let rps = final_stats.requests_completed as f64 / actual_duration_secs;
    let mb_per_sec =
        (final_stats.bytes_transferred as f64 / (1024.0 * 1024.0)) / actual_duration_secs;

    Ok(BenchmarkExecutionResult {
        total_requests: final_stats.requests_completed,
        requests_per_sec: rps,
        transfer_mb_per_sec: mb_per_sec,
        latency: final_recorder,
        stats: final_stats,
    })
}
