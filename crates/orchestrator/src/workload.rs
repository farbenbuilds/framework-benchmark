use crate::lifecycle::{SuiteExecutionConfig, SuiteTiming};
use collector::BackgroundSampler;
use common::result::{TestMetrics, TrafficSettings};
use load_gen::engine::execute_benchmark;
use load_gen::{BenchmarkPlan, ConnectionMode};
use std::time::Duration;

#[derive(Clone, Copy)]
pub(crate) struct Workload<'a> {
    pub category: &'a str,
    path: &'a str,
    pipeline_depth: usize,
    connection_mode: ConnectionMode,
    pub target_rps: Option<u64>,
}

impl<'a> Workload<'a> {
    pub(crate) const fn standard(category: &'a str, path: &'a str) -> Self {
        Self {
            category,
            path,
            pipeline_depth: 1,
            connection_mode: ConnectionMode::KeepAlive,
            target_rps: None,
        }
    }

    pub(crate) const fn pipeline(category: &'a str, path: &'a str, depth: usize) -> Self {
        Self {
            category,
            path,
            pipeline_depth: depth,
            connection_mode: ConnectionMode::KeepAlive,
            target_rps: None,
        }
    }

    pub(crate) const fn churn(category: &'a str, path: &'a str) -> Self {
        Self {
            category,
            path,
            pipeline_depth: 1,
            connection_mode: ConnectionMode::Churn,
            target_rps: None,
        }
    }

    pub(crate) const fn paced(category: &'a str, path: &'a str, target_rps: u64) -> Self {
        Self {
            category,
            path,
            pipeline_depth: 1,
            connection_mode: ConnectionMode::KeepAlive,
            target_rps: Some(target_rps),
        }
    }

    fn traffic_settings(self) -> TrafficSettings {
        TrafficSettings {
            pipeline_depth: self.pipeline_depth,
            connection_mode: match self.connection_mode {
                ConnectionMode::KeepAlive => "keep_alive".into(),
                ConnectionMode::Churn => "churn".into(),
            },
            target_rps: self.target_rps,
        }
    }
}

pub(crate) async fn execute_single_workload(
    config: SuiteExecutionConfig<'_>,
    workload: Workload<'_>,
) -> Result<TestMetrics, crate::error::OrchestratorError> {
    execute_benchmark(plan(config, workload, config.timing, true)).await?;
    let sampler = BackgroundSampler::spawn_target(
        config.target_pid,
        config.cgroup_dir,
        Duration::from_millis(50),
    );
    let result = execute_benchmark(plan(config, workload, config.timing, false)).await?;
    let resources = sampler.stop().await;
    tokio::time::sleep(Duration::from_secs(config.timing.cooldown_secs)).await;
    Ok(TestMetrics {
        category: workload.category.into(),
        concurrency: config.concurrency,
        duration_secs: config.duration_secs,
        requests_total: result.total_requests,
        requests_per_sec: result.requests_per_sec,
        transfer_mb_per_sec: result.transfer_mb_per_sec,
        latency: result.latency.export_distribution(),
        resources,
        status_codes: result.stats.to_status_code_counts(),
        traffic: workload.traffic_settings(),
    })
}

fn plan(
    config: SuiteExecutionConfig<'_>,
    workload: Workload<'_>,
    timing: SuiteTiming,
    warmup: bool,
) -> BenchmarkPlan {
    BenchmarkPlan {
        target_addr: config.target_addr.into(),
        path: workload.path.into(),
        concurrency: if warmup {
            (config.concurrency as usize).min(16)
        } else {
            config.concurrency as usize
        },
        duration_secs: if warmup {
            timing.warmup_secs
        } else {
            u64::from(config.duration_secs)
        },
        pipeline_depth: workload.pipeline_depth,
        connection_mode: workload.connection_mode,
        target_rps: (!warmup).then_some(workload.target_rps).flatten(),
    }
}
