use crate::error::OrchestratorError;
use common::manifest::FrameworkManifest;
use common::result::LibcVariant;
use std::env;
use std::path::Path;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct ContainerTarget {
    pub name: String,
    pub pid: u32,
    pub cgroup_dir: Option<std::path::PathBuf>,
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
    let suffix = match libc {
        LibcVariant::Glibc => "glibc",
        LibcVariant::Musl => "musl",
    };
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| OrchestratorError::Process(error.to_string()))?
        .as_millis();
    let base = sanitize_name(&manifest.framework.name);
    let image = format!("framework-benchmark/{base}:{suffix}");
    let name = format!("benchmark-{base}-{suffix}-{stamp}");
    let workers = benchmark_workers();

    let build_args = crate::build_options::docker_build_args(dockerfile, &image)?;
    run_docker(framework_dir, &build_args)?;
    let image_id = image_id(framework_dir, &image)?;
    let mut args = vec![
        "run".into(),
        "--detach".into(),
        "--name".into(),
        name.clone(),
        "--label".into(),
        "framework-benchmark.managed=true".into(),
        "--network".into(),
        "host".into(),
        "--env".into(),
        format!("BENCHMARK_WORKERS={workers}"),
    ];
    if small_host() {
        args.push("--cpus".into());
        args.push("1".into());
        args.push("--memory".into());
        args.push("2g".into());
        args.push("--pids-limit".into());
        args.push("128".into());
    }
    if let Some(database_url) = database_url {
        args.push("--env".into());
        args.push(format!("DATABASE_URL={database_url}"));
    }
    args.push(image);
    let output = run_docker(framework_dir, &args)?;
    let id = output.trim();
    let pid = match container_pid(framework_dir, id) {
        Ok(pid) => pid,
        Err(error) => {
            let _ = stop_container(framework_dir, &name);
            return Err(error);
        }
    };
    Ok(ContainerTarget {
        name,
        pid,
        cgroup_dir: cgroup_dir_for_pid(pid),
        image_id,
    })
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
    run_docker(framework_dir, &["rm", "--force", name]).map(|_| ())
}

fn container_pid(framework_dir: &Path, id: &str) -> Result<u32, OrchestratorError> {
    run_docker(framework_dir, &["inspect", "--format={{.State.Pid}}", id])?
        .trim()
        .parse()
        .map_err(|error| OrchestratorError::Process(format!("invalid container pid: {error}")))
}

fn cgroup_dir_for_pid(pid: u32) -> Option<std::path::PathBuf> {
    let path = format!("/proc/{pid}/cgroup");
    let content = std::fs::read_to_string(path).ok()?;
    cgroup_dir_from_content(&content)
}

fn cgroup_dir_from_content(content: &str) -> Option<std::path::PathBuf> {
    let cgroup = content.lines().find_map(|line| line.strip_prefix("0::"))?;
    Some(std::path::Path::new("/sys/fs/cgroup").join(cgroup.trim_start_matches('/')))
}

fn benchmark_workers() -> usize {
    let configured = env::var("BENCHMARK_WORKERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &usize| *value > 0);
    configured.unwrap_or_else(|| std::thread::available_parallelism().map_or(1, usize::from))
}

fn small_host() -> bool {
    env::var("BENCHMARK_SMALL_HOST").is_ok_and(|value| value == "1")
}

fn run_docker<S: AsRef<str>>(cwd: &Path, args: &[S]) -> Result<String, OrchestratorError> {
    let runtime = env::var("CONTAINER_RUNTIME").unwrap_or_else(|_| "docker".into());
    let output = Command::new(runtime)
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

fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|char| {
            if char.is_ascii_alphanumeric() {
                char
            } else {
                '-'
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::cgroup_dir_from_content;
    use std::path::Path;

    #[test]
    fn cgroup_directory_uses_the_unified_hierarchy() {
        let content = "11:memory:/legacy\n0::/system.slice/docker.scope\n";
        assert_eq!(
            cgroup_dir_from_content(content).as_deref(),
            Some(Path::new("/sys/fs/cgroup/system.slice/docker.scope"))
        );
    }
}
