use crate::host_profile;
use std::collections::HashMap;
use std::env;

pub(crate) struct RunSpec<'a> {
    pub name: &'a str,
    pub shutdown_signal: &'a str,
    pub workers: usize,
    pub cpuset_cpus: Option<&'a str>,
    pub cgroup_parent: Option<&'a str>,
    pub database_url: Option<&'a str>,
    pub environment: &'a HashMap<String, String>,
    pub image: &'a str,
}

pub(crate) fn run_args(spec: &RunSpec<'_>) -> Vec<String> {
    let mut args = vec![
        "run".into(),
        "--detach".into(),
        "--name".into(),
        spec.name.into(),
        "--label".into(),
        "framework-benchmark.managed=true".into(),
        "--network".into(),
        "host".into(),
        "--init".into(),
        "--stop-signal".into(),
        spec.shutdown_signal.into(),
        "--env".into(),
        format!("BENCHMARK_WORKERS={}", spec.workers),
    ];
    if !logs_enabled() {
        args.extend(["--log-driver".into(), "none".into()]);
    }
    if hardening_enabled() {
        args.extend([
            "--cap-drop".into(),
            "ALL".into(),
            "--security-opt".into(),
            "no-new-privileges".into(),
        ]);
    }
    args.extend(limit_args());
    if let Some(cpus) = spec.cpuset_cpus {
        args.extend(["--cpuset-cpus".into(), cpus.into()]);
    }
    if let Some(parent) = spec.cgroup_parent {
        args.extend(["--cgroup-parent".into(), parent.into()]);
    }
    let mut environment: Vec<_> = spec.environment.iter().collect();
    environment.sort_unstable_by_key(|(key, _)| *key);
    for (key, value) in environment {
        args.extend(["--env".into(), format!("{key}={value}")]);
    }
    if let Some(url) = spec.database_url {
        args.extend(["--env".into(), format!("DATABASE_URL={url}")]);
    }
    args.push(spec.image.into());
    args
}

fn limit_args() -> Vec<String> {
    let small = host_profile::small_host();
    let mut args = Vec::new();
    let pids = env::var("BENCHMARK_PIDS_LIMIT")
        .ok()
        .and_then(|value| value.parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(if small { 128 } else { 512 });
    args.extend(["--pids-limit".into(), pids.to_string()]);
    if small {
        args.extend(["--cpus".into(), "1".into()]);
    }
    let memory = env::var("BENCHMARK_TARGET_MEMORY")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .or_else(|| small.then(|| "2g".into()));
    if let Some(memory) = memory {
        args.extend(["--memory".into(), memory.clone()]);
        args.extend(["--memory-swap".into(), memory]);
    }
    args
}

fn logs_enabled() -> bool {
    host_profile::env_flag("BENCHMARK_CONTAINER_LOGS")
}

fn hardening_enabled() -> bool {
    !host_profile::env_flag("BENCHMARK_DISABLE_HARDENING")
}

#[cfg(test)]
mod tests {
    use super::{RunSpec, run_args};
    use std::collections::HashMap;

    #[test]
    fn run_arguments_keep_isolation_and_environment_boundaries() {
        let environment = HashMap::from([
            ("Z_VAR".into(), "last".into()),
            ("A_VAR".into(), "first".into()),
        ]);
        let spec = RunSpec {
            name: "benchmark-test",
            shutdown_signal: "SIGTERM",
            workers: 2,
            cpuset_cpus: Some("0-1"),
            cgroup_parent: Some("benchmark.slice"),
            database_url: Some("postgres://example/db"),
            environment: &environment,
            image: "framework-benchmark/test:glibc",
        };
        let args = run_args(&spec).join(" ");
        assert!(args.contains("--cpuset-cpus 0-1"));
        assert!(args.contains("--cgroup-parent benchmark.slice"));
        assert!(args.contains("--env A_VAR=first --env Z_VAR=last"));
        assert!(args.contains("DATABASE_URL=postgres://example/db"));
        assert!(args.ends_with("framework-benchmark/test:glibc"));
    }
}
