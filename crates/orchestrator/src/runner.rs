use crate::container_guard::ContainerGuard;
use crate::error::OrchestratorError;
use crate::lifecycle::{SuiteExecutionConfig, SuiteTiming, run_framework_suite_with_timing};
use crate::poller::wait_for_ready;
use crate::provenance::framework_input_sha256;
use crate::session_results::persist_session_results;
use crate::start_container;
use common::manifest::FrameworkManifest;
use common::result::{LibcVariant, RunProvenance, RunResult};
use reporter::markdown::update_readme_leaderboard;
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
    pub timing: SuiteTiming,
}

impl Default for BenchmarkSessionConfig {
    fn default() -> Self {
        let small_host = crate::host_profile::small_host();
        let target_host = crate::host_profile::target_host();
        let remote = crate::host_profile::target_is_remote(&target_host);
        let manage_container =
            std::env::var("BENCHMARK_MANAGE_CONTAINER").map_or(!remote, |value| value != "0");
        let default_duration = if small_host { 10 } else { 30 };
        Self {
            target_host,
            concurrency: configured_concurrency(small_host),
            duration_secs: configured_duration(default_duration),
            libc: LibcVariant::Glibc,
            output_dir: "results".into(),
            readme_path: Some("README.md".into()),
            target_pid: std::env::var("BENCHMARK_TARGET_PID")
                .ok()
                .and_then(|value| value.parse().ok()),
            cgroup_dir: std::env::var_os("BENCHMARK_TARGET_CGROUP").map(PathBuf::from),
            database_url: None,
            manage_container,
            timing: SuiteTiming::for_environment(),
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

fn configured_concurrency(small_host: bool) -> u32 {
    std::env::var("BENCHMARK_CONCURRENCY")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| (1..=256).contains(value))
        .unwrap_or_else(|| default_concurrency(small_host))
}

fn configured_duration(default: u32) -> u32 {
    std::env::var("BENCHMARK_DURATION_SECS")
        .ok()
        .and_then(|value| value.parse::<u32>().ok())
        .filter(|value| (1..=3600).contains(value))
        .unwrap_or(default)
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
    let mut guard = target
        .as_ref()
        .map(|target| ContainerGuard::new(framework_dir, &target.name));
    let target_addr = crate::host_profile::target_addr(&config.target_host, manifest.runtime.port);

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
        let suite = SuiteExecutionConfig {
            target_addr: &target_addr,
            concurrency: config.concurrency,
            duration_secs: config.duration_secs,
            libc: config.libc,
            target_pid,
            cgroup_dir,
            timing: config.timing,
            extended_profiles: true,
        };
        let mut result = run_framework_suite_with_timing(manifest, suite).await?;
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
    let cleanup = match guard.as_mut() {
        Some(guard) => guard.stop(),
        None => Ok(()),
    };
    match (result, cleanup) {
        (Err(error), _) | (Ok(_), Err(error)) => Err(error),
        (Ok(result), Ok(())) => Ok(result),
    }
}

fn current_revision(framework_dir: &Path) -> String {
    if let Ok(revision) = std::env::var("BENCHMARK_REVISION")
        && !revision.trim().is_empty()
    {
        return revision;
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
