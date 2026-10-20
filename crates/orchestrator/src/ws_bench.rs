use crate::lifecycle::SuiteTiming;
use collector::BackgroundSampler;
use common::result::{ResourceUsage, TestMetrics, TrafficSettings};
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
    timing: SuiteTiming,
) -> Result<TestMetrics, crate::error::OrchestratorError> {
    let workers = concurrency.max(1);
    if !(1..=256).contains(&workers) {
        return Err(crate::error::OrchestratorError::Process(
            "websocket concurrency must be between 1 and 256".into(),
        ));
    }
    if duration_secs == 0 {
        return Err(crate::error::OrchestratorError::Process(
            "websocket duration must be positive".into(),
        ));
    }
    if timing.warmup_secs > 0 {
        let deadline = Instant::now() + Duration::from_secs(timing.warmup_secs);
        let warmup_workers = workers.min(16);
        run_round(target_addr, endpoint_path, warmup_workers, deadline).await?;
    }

    let sampler = (target_pid.is_some() || cgroup_dir.is_some()).then(|| {
        BackgroundSampler::spawn_target(target_pid, cgroup_dir, Duration::from_millis(50))
    });
    let started = Instant::now();
    let deadline = started + Duration::from_secs(u64::from(duration_secs));
    let result = run_round(target_addr, endpoint_path, workers, deadline).await;
    let resources = match sampler {
        Some(sampler) => sampler.stop().await,
        None => ResourceUsage::default(),
    };
    let (recorder, stats) = result?;
    tokio::time::sleep(Duration::from_secs(timing.cooldown_secs)).await;

    let rejected = stats
        .status_4xx
        .saturating_add(stats.status_5xx)
        .saturating_add(stats.errors);
    if rejected > 0 {
        return Err(crate::error::OrchestratorError::Process(format!(
            "websocket workload recorded {rejected} rejected or failed responses"
        )));
    }

    let elapsed_secs = started.elapsed().as_secs_f64().max(0.001);
    let rps = stats.requests_completed as f64 / elapsed_secs;
    let mb_per_sec = (stats.bytes_transferred as f64 / (1024.0 * 1024.0)) / elapsed_secs;

    Ok(TestMetrics {
        category: "websocket".into(),
        concurrency: workers,
        duration_secs,
        requests_total: stats.requests_completed,
        requests_per_sec: rps,
        transfer_mb_per_sec: mb_per_sec,
        latency: recorder.export_distribution(),
        resources,
        status_codes: stats.to_status_code_counts(),
        traffic: TrafficSettings {
            pipeline_depth: 1,
            connection_mode: "keep_alive".into(),
            target_rps: None,
        },
    })
}

fn new_recorder() -> Result<LatencyRecorder, crate::error::OrchestratorError> {
    LatencyRecorder::new().map_err(|error| crate::error::OrchestratorError::LoadGen(error.into()))
}

async fn run_round(
    target_addr: &str,
    endpoint_path: &str,
    workers: u32,
    deadline: Instant,
) -> Result<(LatencyRecorder, WorkerStats), crate::error::OrchestratorError> {
    let mut recorders = Vec::with_capacity(workers as usize);
    for _ in 0..workers {
        recorders.push(new_recorder()?);
    }
    let mut handles = Vec::with_capacity(workers as usize);
    for recorder in recorders {
        let addr = target_addr.to_string();
        let path = endpoint_path.to_string();
        handles.push(tokio::spawn(async move {
            let mut recorder = recorder;
            let mut stats = WorkerStats::default();
            let result = AutobahnSuiteRunner::run_echo_benchmark(
                &addr,
                &path,
                deadline,
                &mut recorder,
                &mut stats,
            )
            .await;
            (result, recorder, stats)
        }));
    }

    let mut recorder = new_recorder()?;
    let mut stats = WorkerStats::default();
    let mut failure = None;
    for handle in handles {
        match handle.await {
            Ok((result, worker_recorder, worker_stats)) => {
                recorder.combine(&worker_recorder);
                stats.merge(&worker_stats);
                if let Err(error) = result {
                    failure.get_or_insert_with(|| error.to_string());
                }
            }
            Err(error) => {
                stats.record_error();
                failure.get_or_insert_with(|| error.to_string());
            }
        }
    }
    if let Some(error) = failure {
        return Err(crate::error::OrchestratorError::Process(format!(
            "websocket workload failed: {error}"
        )));
    }
    Ok((recorder, stats))
}
