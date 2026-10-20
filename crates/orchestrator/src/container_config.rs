use crate::error::OrchestratorError;
use std::env;

pub(super) fn optional_config(
    name: &str,
    validator: fn(&str) -> bool,
) -> Result<Option<String>, OrchestratorError> {
    let value = match env::var(name) {
        Ok(value) => value,
        Err(env::VarError::NotPresent) => return Ok(None),
        Err(error) => {
            return Err(OrchestratorError::Process(format!(
                "invalid {name}: {error}"
            )));
        }
    };
    if validator(&value) {
        return Ok(Some(value));
    }
    Err(OrchestratorError::Process(format!("invalid {name}")))
}

pub(super) fn valid_cpuset(value: &str) -> bool {
    !value.is_empty()
        && value.split(',').all(|part| match part.split_once('-') {
            Some((start, end)) => cpu_index(start)
                .zip(cpu_index(end))
                .is_some_and(|(start, end)| start <= end),
            None => cpu_index(part).is_some(),
        })
}

pub(super) fn valid_cgroup_parent(value: &str) -> bool {
    let value = value.strip_prefix('/').unwrap_or(value);
    !value.is_empty()
        && value.split('/').all(|part| {
            !part.is_empty()
                && part != "."
                && part != ".."
                && part
                    .chars()
                    .all(|char| !char.is_control() && !char.is_whitespace())
        })
}

pub(super) fn shutdown_signal(value: Option<&str>) -> Result<&str, OrchestratorError> {
    let value = value.unwrap_or("SIGTERM");
    if valid_shutdown_signal(value) {
        return Ok(value);
    }
    Err(OrchestratorError::Process(format!(
        "invalid shutdown signal: {value}"
    )))
}

pub(super) fn stop_args(name: &str) -> [&str; 4] {
    ["stop", "--time", "5", name]
}

fn cpu_index(value: &str) -> Option<u32> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())
        .flatten()
}

fn valid_shutdown_signal(value: &str) -> bool {
    value
        .parse::<u8>()
        .is_ok_and(|signal| signal > 0 && signal <= 64 && signal != 9 && signal != 19)
        || matches!(
            value,
            "SIGHUP"
                | "SIGINT"
                | "SIGQUIT"
                | "SIGILL"
                | "SIGTRAP"
                | "SIGABRT"
                | "SIGBUS"
                | "SIGFPE"
                | "SIGUSR1"
                | "SIGSEGV"
                | "SIGUSR2"
                | "SIGPIPE"
                | "SIGALRM"
                | "SIGTERM"
                | "SIGCHLD"
                | "SIGCONT"
                | "SIGTSTP"
                | "SIGTTIN"
                | "SIGTTOU"
                | "SIGURG"
                | "SIGXCPU"
                | "SIGXFSZ"
                | "SIGVTALRM"
                | "SIGPROF"
                | "SIGWINCH"
                | "SIGIO"
                | "SIGPWR"
                | "SIGSYS"
        )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn isolation_values_accept_only_safe_forms() {
        assert!(valid_cpuset("0,2-4") && valid_cgroup_parent("benchmark.slice/workload"));
        assert!(!valid_cpuset("0,,2") && !valid_cpuset("4-2"));
        assert!(!valid_cgroup_parent("../escape"));
    }

    #[test]
    fn shutdown_signal_defaults_and_rejects_unknown_values() {
        assert_eq!(shutdown_signal(None).unwrap(), "SIGTERM");
        assert!(shutdown_signal(Some("SIGUNKNOWN")).is_err());
        assert!(shutdown_signal(Some("SIGKILL")).is_err());
    }

    #[test]
    fn docker_stop_arguments_include_a_graceful_timeout() {
        assert_eq!(stop_args("target"), ["stop", "--time", "5", "target"]);
    }
}
