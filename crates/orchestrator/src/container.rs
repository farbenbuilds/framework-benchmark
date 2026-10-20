#[path = "container_config.rs"]
mod container_config;
use crate::error::OrchestratorError;
use common::manifest::FrameworkManifest;
use common::result::LibcVariant;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Debug, Clone)]
pub struct ContainerTarget {
    pub name: String,
    pub pid: u32,
    pub cgroup_dir: Option<PathBuf>,
    pub image_id: String,
}
pub fn start_container(
    framework_dir: &Path,
    manifest: &FrameworkManifest,
    libc: LibcVariant,
    database_url: Option<&str>,
) -> Result<ContainerTarget, OrchestratorError> {
    let docker =
        manifest.build.docker.as_ref().ok_or_else(|| {
            OrchestratorError::Process("framework has no Docker build config".into())
        })?;
    let dockerfile = match libc {
        LibcVariant::Glibc => docker.dockerfile_glibc.as_deref(),
        LibcVariant::Musl => docker.dockerfile_musl.as_deref(),
    }
    .ok_or_else(|| OrchestratorError::Process("framework lacks requested libc image".into()))?;
    let dockerfile = crate::container_paths::secure_dockerfile(framework_dir, dockerfile)?;
    let dockerfile = dockerfile
        .to_str()
        .ok_or_else(|| OrchestratorError::Process("dockerfile path must be valid UTF-8".into()))?;
    let cpuset_cpus =
        container_config::optional_config("BENCHMARK_TARGET_CPUS", container_config::valid_cpuset)?;
    let cgroup_parent = container_config::optional_config(
        "BENCHMARK_CGROUP_PARENT",
        container_config::valid_cgroup_parent,
    )?;
    let shutdown_signal =
        container_config::shutdown_signal(manifest.runtime.shutdown_signal.as_deref())?;
    let suffix = match libc {
        LibcVariant::Glibc => "glibc",
        LibcVariant::Musl => "musl",
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| OrchestratorError::Process(error.to_string()))?
        .as_millis();
    let base = crate::container_paths::sanitize_name(&manifest.framework.name);
    let image = format!("framework-benchmark/{base}:{suffix}");
    let name = format!("benchmark-{base}-{suffix}-{stamp}");
    let build_args =
        crate::build_options::docker_build_args(dockerfile, &image, &docker.build_args)?;
    run_docker_stream(framework_dir, &build_args)?;
    let image_id = image_id(framework_dir, &image)?;
    let workers = crate::host_profile::target_workers(cpuset_cpus.as_deref());
    let spec = crate::container_args::RunSpec {
        name: &name,
        shutdown_signal,
        workers,
        cpuset_cpus: cpuset_cpus.as_deref(),
        cgroup_parent: cgroup_parent.as_deref(),
        database_url,
        environment: &manifest.runtime.environment,
        image: &image,
    };
    let args = crate::container_args::run_args(&spec);
    let id = run_docker(framework_dir, &args)?.trim().to_owned();
    let pid = match container_pid(framework_dir, &id) {
        Ok(pid) => pid,
        Err(error) => return start_error(framework_dir, &name, error),
    };
    let cgroup_dir = match cgroup_dir_for_pid(pid) {
        Ok(path) => path,
        Err(error) => return start_error(framework_dir, &name, error),
    };
    Ok(ContainerTarget {
        name,
        pid,
        cgroup_dir: Some(cgroup_dir),
        image_id,
    })
}
fn start_error(
    framework_dir: &Path,
    name: &str,
    error: OrchestratorError,
) -> Result<ContainerTarget, OrchestratorError> {
    let _ = stop_container(framework_dir, name);
    Err(error)
}
fn image_id(framework_dir: &Path, image: &str) -> Result<String, OrchestratorError> {
    let id = run_docker(
        framework_dir,
        &["image", "inspect", "--format={{.Id}}", image],
    )?;
    let id = id.trim();
    if id.is_empty() {
        return Err(OrchestratorError::Process(
            "container image has no ID".into(),
        ));
    }
    Ok(id.into())
}
pub fn stop_container(framework_dir: &Path, name: &str) -> Result<(), OrchestratorError> {
    let _ = run_docker(framework_dir, &container_config::stop_args(name));
    if run_docker(framework_dir, &["rm", name]).is_ok() {
        return Ok(());
    }
    run_docker(framework_dir, &["rm", "--force", name]).map(|_| ())
}
fn container_pid(framework_dir: &Path, id: &str) -> Result<u32, OrchestratorError> {
    run_docker(framework_dir, &["inspect", "--format={{.State.Pid}}", id])?
        .trim()
        .parse()
        .map_err(|error| OrchestratorError::Process(format!("invalid container pid: {error}")))
}
fn cgroup_dir_for_pid(pid: u32) -> Result<PathBuf, OrchestratorError> {
    let content = std::fs::read_to_string(format!("/proc/{pid}/cgroup"))?;
    let path = cgroup_dir_from_content(&content)
        .ok_or_else(|| OrchestratorError::Process("container lacks a unified cgroup".into()))?;
    if path.is_dir() {
        return Ok(path);
    }
    Err(OrchestratorError::Process(
        "container cgroup path is unavailable".into(),
    ))
}
fn cgroup_dir_from_content(content: &str) -> Option<PathBuf> {
    let cgroup = content.lines().find_map(|line| line.strip_prefix("0::"))?;
    let relative = cgroup.trim_start_matches('/');
    if relative.is_empty() {
        return None;
    }
    Some(Path::new("/sys/fs/cgroup").join(relative))
}
fn run_docker<S: AsRef<str>>(cwd: &Path, args: &[S]) -> Result<String, OrchestratorError> {
    let output = Command::new(container_runtime())
        .args(args.iter().map(AsRef::as_ref))
        .current_dir(cwd)
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(OrchestratorError::Process(
        String::from_utf8_lossy(&output.stderr).trim().to_string(),
    ))
}

fn run_docker_stream<S: AsRef<str>>(cwd: &Path, args: &[S]) -> Result<(), OrchestratorError> {
    let status = Command::new(container_runtime())
        .args(args.iter().map(AsRef::as_ref))
        .current_dir(cwd)
        .status()?;
    if status.success() {
        return Ok(());
    }
    Err(OrchestratorError::Process(format!(
        "container command failed with {status}"
    )))
}

fn container_runtime() -> String {
    env::var("CONTAINER_RUNTIME").unwrap_or_else(|_| "docker".into())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cgroup_directory_uses_the_unified_hierarchy() {
        assert_eq!(
            cgroup_dir_from_content("11:memory:/legacy\n0::/system.slice/docker.scope\n")
                .as_deref(),
            Some(Path::new("/sys/fs/cgroup/system.slice/docker.scope"))
        );
    }

    #[test]
    fn cgroup_root_is_not_accepted_as_a_target_scope() {
        assert_eq!(cgroup_dir_from_content("0::/\n"), None);
    }
}
