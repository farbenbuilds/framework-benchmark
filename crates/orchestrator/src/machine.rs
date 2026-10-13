use common::result::MachineInfo;
use std::collections::BTreeSet;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn current_machine_info() -> MachineInfo {
    let cpuinfo = fs::read_to_string("/proc/cpuinfo").unwrap_or_default();
    MachineInfo {
        hostname: hostname(),
        os_release: os_release(),
        kernel_version: read_trimmed("/proc/sys/kernel/osrelease")
            .unwrap_or_else(|| "unknown".into()),
        cpu_model: cpu_model(&cpuinfo).unwrap_or_else(|| "unknown".into()),
        physical_cores: physical_core_count(&cpuinfo),
        logical_cores: logical_core_count(&cpuinfo),
        total_memory_mb: memory_mb(),
    }
}

pub fn current_epoch_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_secs())
}

fn hostname() -> String {
    std::env::var("HOSTNAME")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| read_trimmed("/etc/hostname"))
        .unwrap_or_else(|| "unknown".into())
}

fn os_release() -> String {
    let content = fs::read_to_string("/etc/os-release").unwrap_or_default();
    content
        .lines()
        .find_map(|line| line.strip_prefix("PRETTY_NAME="))
        .map_or_else(|| std::env::consts::OS.into(), unquote)
}

fn read_trimmed(path: &str) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn unquote(value: &str) -> String {
    value.trim_matches('"').to_string()
}

fn cpu_model(cpuinfo: &str) -> Option<String> {
    cpuinfo
        .lines()
        .find_map(|line| {
            line.split_once(':')
                .filter(|(key, _)| key.trim() == "model name")
        })
        .map(|(_, value)| value.trim().to_string())
}

fn logical_core_count(cpuinfo: &str) -> usize {
    let count = cpuinfo
        .lines()
        .filter(|line| line.starts_with("processor"))
        .count();
    count.max(1)
}

fn physical_core_count(cpuinfo: &str) -> usize {
    let mut cores = BTreeSet::new();
    let mut physical_id: Option<u32> = None;
    let mut core_id: Option<u32> = None;

    for line in cpuinfo.lines().chain(std::iter::once("")) {
        if line.is_empty() {
            if let (Some(package), Some(core)) = (physical_id.take(), core_id.take()) {
                cores.insert((package, core));
            }
            continue;
        }
        let Some((key, value)) = line.split_once(':') else {
            continue;
        };
        match key.trim() {
            "physical id" => physical_id = value.trim().parse().ok(),
            "core id" => core_id = value.trim().parse().ok(),
            _ => {}
        }
    }
    if cores.is_empty() {
        logical_core_count(cpuinfo)
    } else {
        cores.len()
    }
}

fn memory_mb() -> u64 {
    fs::read_to_string("/proc/meminfo")
        .ok()
        .and_then(|content| {
            content.lines().find_map(|line| {
                line.strip_prefix("MemTotal:")?
                    .split_whitespace()
                    .next()?
                    .parse::<u64>()
                    .ok()
            })
        })
        .map_or(0, |kb| kb / 1024)
}

#[cfg(test)]
mod tests {
    use super::{cpu_model, physical_core_count};

    #[test]
    fn parses_cpu_layout() {
        let cpuinfo = "processor : 0\nphysical id : 0\ncore id : 0\n\nprocessor : 1\nphysical id : 0\ncore id : 0\n\nprocessor : 2\nphysical id : 0\ncore id : 1\nmodel name : test cpu\n";
        assert_eq!(physical_core_count(cpuinfo), 2);
        assert_eq!(cpu_model(cpuinfo).as_deref(), Some("test cpu"));
    }
}
