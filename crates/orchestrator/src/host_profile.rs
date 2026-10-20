use std::env;

pub const SMALL_HOST_ENV: &str = "BENCHMARK_SMALL_HOST";

pub fn small_host() -> bool {
    env_flag(SMALL_HOST_ENV)
}

pub fn env_flag(name: &str) -> bool {
    env::var(name).is_ok_and(|value| value == "1")
}

pub fn target_host() -> String {
    env::var("BENCHMARK_TARGET_HOST")
        .ok()
        .filter(|value| valid_host(value))
        .unwrap_or_else(|| "127.0.0.1".into())
}

pub fn target_addr(host: &str, port: u16) -> String {
    if host.contains(':') && !host.starts_with('[') {
        return format!("[{host}]:{port}");
    }
    format!("{host}:{port}")
}

pub fn target_is_remote(host: &str) -> bool {
    !matches!(host, "127.0.0.1" | "localhost" | "::1")
}

fn valid_host(value: &str) -> bool {
    !value.trim().is_empty()
        && value.len() <= 253
        && !value
            .bytes()
            .any(|byte| byte.is_ascii_control() || byte.is_ascii_whitespace())
}

pub fn target_workers(cpuset: Option<&str>) -> usize {
    if let Some(workers) = env::var("BENCHMARK_TARGET_WORKERS")
        .ok()
        .and_then(|value| value.parse().ok())
        .filter(|value: &usize| *value > 0)
    {
        return workers;
    }
    if small_host() {
        return 1;
    }
    if let Some(count) = cpuset.and_then(cpu_count) {
        return count.max(1);
    }
    std::thread::available_parallelism().map_or(1, usize::from)
}

pub fn cpu_count(cpuset: &str) -> Option<usize> {
    let mut total = 0_usize;
    for part in cpuset.split(',') {
        let (start, end) = part.split_once('-').unwrap_or((part, part));
        let start = start.parse::<usize>().ok()?;
        let end = end.parse::<usize>().ok()?;
        if start > end {
            return None;
        }
        total = total.checked_add(end - start + 1)?;
    }
    Some(total)
}

#[cfg(test)]
mod tests {
    use super::{cpu_count, target_addr, target_is_remote};

    #[test]
    fn cpu_sets_are_counted_without_overflows() {
        assert_eq!(cpu_count("0"), Some(1));
        assert_eq!(cpu_count("2-7"), Some(6));
        assert_eq!(cpu_count("0,2-4"), Some(4));
        assert_eq!(cpu_count("4-2"), None);
    }

    #[test]
    fn only_loopback_hosts_are_local() {
        assert!(!target_is_remote("127.0.0.1"));
        assert!(!target_is_remote("localhost"));
        assert!(target_is_remote("10.0.0.12"));
    }

    #[test]
    fn ipv6_hosts_are_bracketed_for_socket_addresses() {
        assert_eq!(target_addr("::1", 8080), "[::1]:8080");
        assert_eq!(target_addr("127.0.0.1", 8080), "127.0.0.1:8080");
    }
}
