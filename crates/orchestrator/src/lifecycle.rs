use collector::BackgroundSampler;
use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, RunResult, TestMetrics};
use load_gen::engine::execute_benchmark;
use load_gen::plan::BenchmarkPlan;
use std::path::Path;
use std::time::Duration;

#[derive(Debug, Clone, Copy)]
pub struct SuiteTiming {
    pub warmup_secs: u64,
    pub cooldown_secs: u64,
}

impl Default for SuiteTiming {
    fn default() -> Self {
        Self {
            warmup_secs: 15,
            cooldown_secs: 10,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SuiteExecutionConfig<'a> {
    pub target_addr: &'a str,
    pub concurrency: u32,
    pub duration_secs: u32,
    pub libc: LibcVariant,
    pub target_pid: Option<u32>,
    pub cgroup_dir: Option<&'a Path>,
    pub timing: SuiteTiming,
}

#[derive(Clone, Copy)]
struct WorkloadConfig<'a> {
    target_addr: &'a str,
    concurrency: u32,
    duration_secs: u32,
    target_pid: Option<u32>,
    cgroup_dir: Option<&'a Path>,
    timing: SuiteTiming,
}

pub async fn run_framework_test(
    manifest: &FrameworkManifest,
    target_addr: &str,
    category: &str,
    endpoint_path: &str,
    concurrency: u32,
    duration_secs: u32,
    libc: LibcVariant,
) -> Result<RunResult, crate::error::OrchestratorError> {
    let config = WorkloadConfig {
        target_addr,
        concurrency,
        duration_secs,
        target_pid: None,
        cgroup_dir: None,
        timing: SuiteTiming::default(),
    };
    let metrics = execute_single_workload(config, category, endpoint_path).await?;
    Ok(RunResult {
        run_id: "test-run-1".into(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc,
        timestamp_epoch_secs: crate::machine::current_epoch_secs(),
        machine: crate::machine::current_machine_info(),
        benchmarks: vec![metrics],
    })
}

pub async fn run_framework_suite(
    manifest: &FrameworkManifest,
    target_addr: &str,
    concurrency: u32,
    duration_secs: u32,
    libc: LibcVariant,
    target_pid: Option<u32>,
    cgroup_dir: Option<&Path>,
) -> Result<RunResult, crate::error::OrchestratorError> {
    let config = SuiteExecutionConfig {
        target_addr,
        concurrency,
        duration_secs,
        libc,
        target_pid,
        cgroup_dir,
        timing: SuiteTiming::default(),
    };
    run_framework_suite_with_timing(manifest, config).await
}

pub async fn run_framework_suite_with_timing(
    manifest: &FrameworkManifest,
    config: SuiteExecutionConfig<'_>,
) -> Result<RunResult, crate::error::OrchestratorError> {
    let mut benchmarks = Vec::new();
    let caps = &manifest.capabilities;
    let endpoints = [
        (caps.plaintext, "plaintext", "/plaintext"),
        (caps.json, "json", "/json"),
        (caps.db_single, "db_single", "/db"),
        (caps.db_multi, "db_multi", "/queries?queries=20"),
        (caps.fortunes, "fortunes", "/fortunes"),
        (caps.updates, "updates", "/updates?queries=20"),
    ];

    for (enabled, cat, path) in endpoints {
        if enabled {
            benchmarks
                .push(execute_single_workload(WorkloadConfig::from(config), cat, path).await?);
        }
    }

    if caps.websocket {
        benchmarks.push(
            crate::ws_bench::execute_websocket_workload(
                config.target_addr,
                "/ws",
                config.concurrency,
                config.duration_secs,
                config.target_pid,
                config.cgroup_dir,
            )
            .await?,
        );
    }

    Ok(RunResult {
        run_id: format!("{}-{:?}-suite", manifest.framework.name, config.libc).to_lowercase(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc: config.libc,
        timestamp_epoch_secs: crate::machine::current_epoch_secs(),
        machine: crate::machine::current_machine_info(),
        benchmarks,
    })
}

impl<'a> From<SuiteExecutionConfig<'a>> for WorkloadConfig<'a> {
    fn from(config: SuiteExecutionConfig<'a>) -> Self {
        Self {
            target_addr: config.target_addr,
            concurrency: config.concurrency,
            duration_secs: config.duration_secs,
            target_pid: config.target_pid,
            cgroup_dir: config.cgroup_dir,
            timing: config.timing,
        }
    }
}

async fn execute_single_workload(
    config: WorkloadConfig<'_>,
    category: &str,
    endpoint_path: &str,
) -> Result<TestMetrics, crate::error::OrchestratorError> {
    let warmup_plan = BenchmarkPlan {
        target_addr: config.target_addr.to_string(),
        path: endpoint_path.to_string(),
        concurrency: (config.concurrency as usize).min(16),
        duration_secs: config.timing.warmup_secs,
        pipeline_depth: 1,
        target_rps: None,
    };
    let _ = execute_benchmark(warmup_plan).await?;

    let sampler = BackgroundSampler::spawn_target(
        config.target_pid,
        config.cgroup_dir,
        Duration::from_millis(50),
    );

    let measurement_plan = BenchmarkPlan {
        target_addr: config.target_addr.to_string(),
        path: endpoint_path.to_string(),
        concurrency: config.concurrency as usize,
        duration_secs: u64::from(config.duration_secs),
        pipeline_depth: 1,
        target_rps: None,
    };
    let exec_res = execute_benchmark(measurement_plan).await?;
    let resources = sampler.stop().await;
    tokio::time::sleep(Duration::from_secs(config.timing.cooldown_secs)).await;

    Ok(TestMetrics {
        category: category.to_string(),
        concurrency: config.concurrency,
        duration_secs: config.duration_secs,
        requests_total: exec_res.total_requests,
        requests_per_sec: exec_res.requests_per_sec,
        transfer_mb_per_sec: exec_res.transfer_mb_per_sec,
        latency: exec_res.latency.export_distribution(),
        resources,
        status_codes: exec_res.stats.to_status_code_counts(),
    })
}
