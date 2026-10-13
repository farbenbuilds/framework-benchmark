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

    run_docker(
        framework_dir,
        &["build", "--file", dockerfile, "--tag", &image, "."],
    )?;
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
    let pid = run_docker(framework_dir, &["inspect", "--format={{.State.Pid}}", id])?
        .trim()
        .parse()
        .map_err(|error| OrchestratorError::Process(format!("invalid container pid: {error}")))?;
    Ok(ContainerTarget { name, pid })
}

pub fn stop_container(framework_dir: &Path, target: &ContainerTarget) {
    let _ = run_docker(framework_dir, &["rm", "--force", &target.name]);
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
    let output = Command::new("docker")
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
