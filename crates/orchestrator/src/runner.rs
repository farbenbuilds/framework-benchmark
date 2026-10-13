use crate::error::OrchestratorError;
use crate::lifecycle::run_framework_suite;
use crate::poller::wait_for_ready;
use crate::provenance::framework_input_sha256;
use crate::{start_container, stop_container};
use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, RunProvenance, RunResult};
use reporter::archive::{
    timestamped_result_path, timestamped_result_relative_path, write_result_json,
};
use reporter::chart::generate_throughput_svg;
use reporter::index::update_historical_index;
use reporter::logic::aggregate_leaderboard;
use reporter::markdown::{render_markdown_table, update_readme_leaderboard};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct BenchmarkSessionConfig {
    pub target_host: String,
    pub concurrency: u32,
    pub duration_secs: u32,
    pub libc: LibcVariant,
    pub output_dir: String,
    pub readme_path: Option<String>,
    pub target_pid: Option<u32>,
    pub cgroup_dir: Option<PathBuf>,
    pub database_url: Option<String>,
    pub manage_container: bool,
}

impl Default for BenchmarkSessionConfig {
    fn default() -> Self {
        let small_host = std::env::var("BENCHMARK_SMALL_HOST").is_ok_and(|value| value == "1");
        Self {
            target_host: "127.0.0.1".into(),
            concurrency: default_concurrency(small_host),
            duration_secs: if small_host { 10 } else { 30 },
            libc: LibcVariant::Glibc,
            output_dir: "results".into(),
            readme_path: Some("README.md".into()),
            target_pid: None,
            cgroup_dir: None,
            database_url: None,
            manage_container: true,
        }
    }
}

fn default_concurrency(small_host: bool) -> u32 {
    if small_host {
        return 8;
    }
    let cpus = std::thread::available_parallelism().map_or(1, usize::from);
    u32::try_from(cpus.saturating_mul(4).clamp(4, 64)).unwrap_or(64)
}

pub async fn execute_framework_session(
    framework_dir: &Path,
    manifest: &FrameworkManifest,
    config: &BenchmarkSessionConfig,
) -> Result<RunResult, OrchestratorError> {
    let revision = current_revision(framework_dir);
    let framework_input_sha256 = framework_input_sha256(framework_dir)?;
    let target = if config.manage_container {
        Some(start_container(
            framework_dir,
            manifest,
            config.libc,
            config.database_url.as_deref(),
        )?)
    } else {
        None
    };
    let port = manifest.runtime.port;
    let target_addr = format!("{}:{}", config.target_host, port);

    let result = async {
        wait_for_ready(
            &target_addr,
            &manifest.runtime.healthcheck_path,
            manifest.runtime.startup_timeout_seconds,
        )
        .await?;
        let target_pid = target
            .as_ref()
            .map_or(config.target_pid, |target| Some(target.pid));
        let cgroup_dir = target
            .as_ref()
            .and_then(|target| target.cgroup_dir.as_deref())
            .or(config.cgroup_dir.as_deref());
        let mut result = run_framework_suite(
            manifest,
            &target_addr,
            config.concurrency,
            config.duration_secs,
            config.libc,
            target_pid,
            cgroup_dir,
        )
        .await?;
        result.provenance = RunProvenance {
            revision,
            image_id: target
                .as_ref()
                .map(|target| target.image_id.clone())
                .or_else(|| std::env::var("BENCHMARK_IMAGE_ID").ok()),
            framework_input_sha256: Some(framework_input_sha256),
            target_cpus: std::env::var("BENCHMARK_TARGET_CPUS").ok(),
            load_cpus: std::env::var("BENCHMARK_LOAD_CPUS").ok(),
            cgroup_path: target.as_ref().and_then(|target| {
                target
                    .cgroup_dir
                    .as_ref()
                    .map(|path| path.display().to_string())
            }),
        };
        persist_session_results(&result, &config.output_dir)?;
        if let Some(readme) = &config.readme_path {
            update_readme_leaderboard(Path::new(readme), std::slice::from_ref(&result))?;
        }
        Ok(result)
    }
    .await;
    let cleanup = match &target {
        Some(target) => stop_container(framework_dir, &target.name),
        None => Ok(()),
    };
    match (result, cleanup) {
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        (Ok(result), Ok(())) => Ok(result),
    }
}

fn current_revision(framework_dir: &Path) -> String {
    if let Ok(revision) = std::env::var("BENCHMARK_REVISION") {
        if !revision.trim().is_empty() {
            return revision;
        }
    }
    std::process::Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(framework_dir)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|revision| revision.trim().into())
        .filter(|revision: &String| !revision.is_empty())
        .unwrap_or_else(|| "unknown".into())
}

fn persist_session_results(result: &RunResult, base_dir: &str) -> Result<(), OrchestratorError> {
    let base = Path::new(base_dir);
    let json_path = timestamped_result_path(base, result)?;
    let relative_path = timestamped_result_relative_path(result)?;
    let index_path = base.join("index.json");

    write_result_json(&json_path, result)?;
    update_historical_index(
        &index_path,
        result,
        relative_path
            .to_str()
            .ok_or_else(|| OrchestratorError::Process("result path must be valid UTF-8".into()))?,
    )?;
    let charts_dir = base.join("charts");
    fs::create_dir_all(&charts_dir)?;
    for category in categories(result) {
        let rows = aggregate_leaderboard(std::slice::from_ref(result), &category);
        let table = render_markdown_table(&rows, &category);
        let title = format!("{category} Throughput Benchmark");
        let svg = generate_throughput_svg(&rows, &title);
        fs::write(charts_dir.join(format!("{category}_throughput.svg")), svg)?;
        fs::write(base.join(format!("leaderboard_{category}.md")), table)?;
    }

    Ok(())
}

fn categories(result: &RunResult) -> Vec<String> {
    let mut categories: Vec<String> = result
        .benchmarks
        .iter()
        .map(|benchmark| benchmark.category.clone())
        .collect();
    categories.sort_unstable();
    categories.dedup();
    categories
}
