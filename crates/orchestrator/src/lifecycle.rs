use collector::BackgroundSampler;
use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, MachineInfo, RunResult, TestMetrics};
use load_gen::engine::execute_benchmark;
use load_gen::plan::BenchmarkPlan;
use std::path::Path;
use std::time::Duration;

pub async fn run_framework_test(
    manifest: &FrameworkManifest,
    target_addr: &str,
    category: &str,
    endpoint_path: &str,
    concurrency: u32,
    duration_secs: u32,
    libc: LibcVariant,
) -> Result<RunResult, crate::error::OrchestratorError> {
    let metrics = execute_single_workload(
        target_addr,
        category,
        endpoint_path,
        concurrency,
        duration_secs,
        None,
        None,
    )
    .await?;

    Ok(RunResult {
        run_id: "test-run-1".into(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc,
        timestamp_epoch_secs: 1_700_000_000,
        machine: default_machine_info(),
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
            benchmarks.push(
                execute_single_workload(
                    target_addr,
                    cat,
                    path,
                    concurrency,
                    duration_secs,
                    target_pid,
                    cgroup_dir,
                )
                .await?,
            );
        }
    }

    if caps.websocket {
        benchmarks.push(
            crate::ws_bench::execute_websocket_workload(
                target_addr,
                "/ws",
                concurrency,
                duration_secs,
                target_pid,
                cgroup_dir,
            )
            .await?,
        );
    }

    Ok(RunResult {
        run_id: format!("{}-{libc:?}-suite", manifest.framework.name).to_lowercase(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc,
        timestamp_epoch_secs: 1_700_000_000,
        machine: default_machine_info(),
        benchmarks,
    })
}

async fn execute_single_workload(
    target_addr: &str,
    category: &str,
    endpoint_path: &str,
    concurrency: u32,
    duration_secs: u32,
    target_pid: Option<u32>,
    cgroup_dir: Option<&Path>,
) -> Result<TestMetrics, crate::error::OrchestratorError> {
    let warmup_plan = BenchmarkPlan {
        target_addr: target_addr.to_string(),
        path: endpoint_path.to_string(),
        concurrency: (concurrency as usize).min(16),
        duration_secs: 5,
        pipeline_depth: 1,
        target_rps: None,
    };
    let _ = execute_benchmark(warmup_plan).await?;

    let sampler =
        BackgroundSampler::spawn_target(target_pid, cgroup_dir, Duration::from_millis(50));

    let measurement_plan = BenchmarkPlan {
        target_addr: target_addr.to_string(),
        path: endpoint_path.to_string(),
        concurrency: concurrency as usize,
        duration_secs: u64::from(duration_secs),
        pipeline_depth: 1,
        target_rps: None,
    };
    let exec_res = execute_benchmark(measurement_plan).await?;
    let resources = sampler.stop().await;

    Ok(TestMetrics {
        category: category.to_string(),
        concurrency,
        duration_secs,
        requests_total: exec_res.total_requests,
        requests_per_sec: exec_res.requests_per_sec,
        transfer_mb_per_sec: exec_res.transfer_mb_per_sec,
        latency: exec_res.latency.export_distribution(),
        resources,
        status_codes: exec_res.stats.to_status_code_counts(),
    })
}

fn default_machine_info() -> MachineInfo {
    MachineInfo {
        hostname: "benchmark-host".into(),
        os_release: "linux".into(),
        kernel_version: "6.6".into(),
        cpu_model: "synthetic".into(),
        physical_cores: 8,
        logical_cores: 16,
        total_memory_mb: 16384,
    }
}
