use crate::workload::{Workload, execute_single_workload};
use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, RunProvenance, RunResult};
use std::path::Path;

pub use crate::suite_timing::SuiteTiming;

#[derive(Debug, Clone, Copy)]
pub struct SuiteExecutionConfig<'a> {
    pub target_addr: &'a str,
    pub concurrency: u32,
    pub duration_secs: u32,
    pub libc: LibcVariant,
    pub target_pid: Option<u32>,
    pub cgroup_dir: Option<&'a Path>,
    pub timing: SuiteTiming,
    pub extended_profiles: bool,
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
    let config = SuiteExecutionConfig {
        target_addr,
        concurrency,
        duration_secs,
        libc,
        target_pid: None,
        cgroup_dir: None,
        timing: SuiteTiming::for_environment(),
        extended_profiles: false,
    };
    let metrics =
        execute_single_workload(config, Workload::standard(category, endpoint_path)).await?;
    Ok(test_result(manifest, libc, metrics))
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
        timing: SuiteTiming::for_environment(),
        extended_profiles: true,
    };
    run_framework_suite_with_timing(manifest, config).await
}

pub async fn run_framework_suite_with_timing(
    manifest: &FrameworkManifest,
    config: SuiteExecutionConfig<'_>,
) -> Result<RunResult, crate::error::OrchestratorError> {
    let mut benchmarks = Vec::new();
    for workload in techempower_workloads(manifest) {
        benchmarks.push(execute_single_workload(config, workload).await?);
    }
    if config.extended_profiles && manifest.capabilities.plaintext {
        for workload in extended_workloads(config.concurrency) {
            benchmarks.push(execute_single_workload(config, workload).await?);
        }
    }
    if manifest.capabilities.websocket {
        let path = manifest.runtime.websocket_path.as_deref().unwrap_or("/ws");
        benchmarks.push(
            crate::ws_bench::execute_websocket_workload(
                config.target_addr,
                path,
                config.concurrency,
                config.duration_secs,
                config.target_pid,
                config.cgroup_dir,
                config.timing,
            )
            .await?,
        );
    }
    Ok(suite_result(manifest, config.libc, benchmarks))
}

fn techempower_workloads(manifest: &FrameworkManifest) -> Vec<Workload<'static>> {
    let caps = &manifest.capabilities;
    [
        (
            caps.plaintext,
            Workload::standard("plaintext", "/plaintext"),
        ),
        (caps.json, Workload::standard("json", "/json")),
        (caps.db_single, Workload::standard("single_query", "/db")),
        (
            caps.db_multi,
            Workload::standard("multiple_queries", "/queries?queries=20"),
        ),
        (caps.fortunes, Workload::standard("fortunes", "/fortunes")),
        (
            caps.updates,
            Workload::standard("data_updates", "/updates?queries=20"),
        ),
    ]
    .into_iter()
    .filter_map(|(enabled, workload)| enabled.then_some(workload))
    .collect()
}

fn extended_workloads(concurrency: u32) -> [Workload<'static>; 3] {
    let target_rps = u64::from(concurrency).saturating_mul(100);
    [
        Workload::pipeline("http1_pipelining_16", "/plaintext", 16),
        Workload::churn("connection_churn", "/plaintext"),
        Workload::paced("paced_plaintext", "/plaintext", target_rps),
    ]
}

fn test_result(
    manifest: &FrameworkManifest,
    libc: LibcVariant,
    metrics: common::result::TestMetrics,
) -> RunResult {
    RunResult {
        run_id: "test-run-1".into(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc,
        timestamp_epoch_secs: crate::machine::current_epoch_secs(),
        machine: crate::machine::current_machine_info(),
        provenance: RunProvenance::default(),
        benchmarks: vec![metrics],
    }
}

fn suite_result(
    manifest: &FrameworkManifest,
    libc: LibcVariant,
    benchmarks: Vec<common::result::TestMetrics>,
) -> RunResult {
    let timestamp_epoch_secs = crate::machine::current_epoch_secs();
    RunResult {
        run_id: format!(
            "{}-{libc:?}-suite-{timestamp_epoch_secs}",
            manifest.framework.name
        )
        .to_lowercase(),
        framework_name: manifest.framework.name.clone(),
        framework_version: manifest.framework.version.clone(),
        libc,
        timestamp_epoch_secs,
        machine: crate::machine::current_machine_info(),
        provenance: RunProvenance::default(),
        benchmarks,
    }
}

#[cfg(test)]
mod tests {
    use super::extended_workloads;

    #[test]
    fn extended_profiles_use_a_bounded_paced_rate() {
        assert_eq!(extended_workloads(8)[2].target_rps, Some(800));
    }
}
