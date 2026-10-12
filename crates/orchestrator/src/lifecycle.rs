use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, MachineInfo, ResourceUsage, RunResult, TestMetrics};
use load_gen::engine::execute_benchmark;
use load_gen::plan::BenchmarkPlan;

pub async fn run_framework_test(
    manifest: &FrameworkManifest,
    target_addr: &str,
    category: &str,
    endpoint_path: &str,
    concurrency: u32,
    duration_secs: u32,
    libc: LibcVariant,
) -> Result<RunResult, crate::error::OrchestratorError> {
    let warmup_plan = BenchmarkPlan {
        target_addr: target_addr.to_string(),
        path: endpoint_path.to_string(),
        concurrency: (concurrency as usize).min(16),
        duration_secs: 15,
        pipeline_depth: 1,
        target_rps: None,
    };
    let _ = execute_benchmark(warmup_plan).await?;

    let measurement_plan = BenchmarkPlan {
        target_addr: target_addr.to_string(),
        path: endpoint_path.to_string(),
        concurrency: concurrency as usize,
        duration_secs: u64::from(duration_secs),
        pipeline_depth: 1,
        target_rps: None,
    };
    let exec_res = execute_benchmark(measurement_plan).await?;

    let metrics = TestMetrics {
        category: category.to_string(),
        concurrency,
        duration_secs,
        requests_total: exec_res.total_requests,
        requests_per_sec: exec_res.requests_per_sec,
        transfer_mb_per_sec: exec_res.transfer_mb_per_sec,
        latency: exec_res.latency.export_distribution(),
        resources: ResourceUsage {
            peak_rss_bytes: 0,
            avg_rss_bytes: 0,
            cpu_user_pct: 0.0,
            cpu_kernel_pct: 0.0,
            context_switches: 0,
        },
        status_codes: exec_res.stats.to_status_code_counts(),
    };

    let result = RunResult {
        run_id: "test-run-1".into(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc,
        timestamp_epoch_secs: 1_700_000_000,
        machine: MachineInfo {
            hostname: "benchmark-host".into(),
            os_release: "linux".into(),
            kernel_version: "6.6".into(),
            cpu_model: "synthetic".into(),
            physical_cores: 8,
            logical_cores: 16,
            total_memory_mb: 16384,
        },
        benchmarks: vec![metrics],
    };

    Ok(result)
}
