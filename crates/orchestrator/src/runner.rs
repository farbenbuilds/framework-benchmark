use crate::error::OrchestratorError;
use crate::lifecycle::run_framework_suite;
use crate::poller::wait_for_ready;
use crate::{start_container, stop_container};
use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, RunResult};
use reporter::archive::write_result_json;
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
    pub manage_container: bool,
}

impl Default for BenchmarkSessionConfig {
    fn default() -> Self {
        Self {
            target_host: "127.0.0.1".into(),
            concurrency: 64,
            duration_secs: 15,
            libc: LibcVariant::Glibc,
            output_dir: "results".into(),
            readme_path: Some("README.md".into()),
            target_pid: None,
            cgroup_dir: None,
            manage_container: true,
        }
    }
}

pub async fn execute_framework_session(
    framework_dir: &Path,
    manifest: &FrameworkManifest,
    config: &BenchmarkSessionConfig,
) -> Result<RunResult, OrchestratorError> {
    let target = if config.manage_container {
        Some(start_container(framework_dir, manifest, config.libc)?)
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
        let result = run_framework_suite(
            manifest,
            &target_addr,
            config.concurrency,
            config.duration_secs,
            config.libc,
            target_pid,
            config.cgroup_dir.as_deref(),
        )
        .await?;
        persist_session_results(&result, &config.output_dir)?;
        if let Some(readme) = &config.readme_path {
            let _ = update_readme_leaderboard(Path::new(readme), std::slice::from_ref(&result));
        }
        Ok(result)
    }
    .await;
    if let Some(target) = &target {
        stop_container(framework_dir, target);
    }
    result
}

fn persist_session_results(result: &RunResult, base_dir: &str) -> Result<(), OrchestratorError> {
    let base = Path::new(base_dir);
    let run_file = format!("{}.json", result.run_id);
    let json_path = base.join("data").join("current").join(&run_file);
    let index_path = base.join("index.json");

    write_result_json(&json_path, result)?;
    update_historical_index(&index_path, result, json_path.to_str().unwrap_or_default())?;

    let rows = aggregate_leaderboard(std::slice::from_ref(result), "plaintext");
    let table = render_markdown_table(&rows, "plaintext");
    let svg = generate_throughput_svg(&rows, "Plaintext Throughput Benchmark");

    let charts_dir = base.join("charts");
    fs::create_dir_all(&charts_dir)?;
    fs::write(charts_dir.join("plaintext_throughput.svg"), svg)?;
    fs::write(base.join("leaderboard_plaintext.md"), table)?;

    Ok(())
}
