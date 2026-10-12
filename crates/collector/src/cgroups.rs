use crate::error::CollectorError;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgroupSample {
    pub current_bytes: u64,
    pub peak_bytes: u64,
    pub user_usec: u64,
    pub system_usec: u64,
}

pub fn parse_cgroup_memory_stat(content: &str) -> Result<u64, CollectorError> {
    content
        .trim()
        .parse::<u64>()
        .map_err(|e| CollectorError::Parse(format!("invalid cgroup memory stat: {e}")))
}

pub fn parse_cgroup_cpu_stat(content: &str) -> (u64, u64) {
    let mut user = 0;
    let mut system = 0;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("user_usec ") {
            user = rest.trim().parse().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("system_usec ") {
            system = rest.trim().parse().unwrap_or(0);
        }
    }
    (user, system)
}
