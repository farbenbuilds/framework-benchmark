use crate::error::CollectorError;
use std::fs;

const DEFAULT_PAGE_SIZE: u64 = 4096;
const SMAPS_ROLLUP_PATH: &str = "/proc/self/smaps_rollup";
const SMAPS_PATH: &str = "/proc/self/smaps";

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ProcessSample {
    pub rss_bytes: u64,
    pub user_time_ticks: u64,
    pub kernel_time_ticks: u64,
    pub voluntary_ctxt_switches: u64,
    pub involuntary_ctxt_switches: u64,
}

pub fn parse_statm(content: &str, page_size: u64) -> Result<u64, CollectorError> {
    let mut parts = content.split_whitespace();
    let _size = parts.next();
    let resident = parts
        .next()
        .ok_or_else(|| CollectorError::Parse("missing resident field in statm".into()))?;
    let pages: u64 = resident
        .parse()
        .map_err(|e| CollectorError::Parse(format!("invalid pages in statm: {e}")))?;
    Ok(pages.saturating_mul(page_size))
}

pub fn parse_kernel_page_size(content: &str) -> Option<u64> {
    content
        .lines()
        .filter_map(|line| line.strip_prefix("KernelPageSize:"))
        .filter_map(|value| value.split_whitespace().next()?.parse::<u64>().ok())
        .find(|size| *size > 0)
        .and_then(|kib| kib.checked_mul(1024))
}

pub fn select_page_size(rollup: Option<&str>, smaps: Option<&str>) -> u64 {
    rollup
        .and_then(parse_kernel_page_size)
        .or_else(|| smaps.and_then(parse_kernel_page_size))
        .unwrap_or(DEFAULT_PAGE_SIZE)
}

pub fn resolve_page_size() -> u64 {
    let rollup = fs::read_to_string(SMAPS_ROLLUP_PATH).ok();
    let smaps = if rollup.as_deref().and_then(parse_kernel_page_size).is_some() {
        None
    } else {
        fs::read_to_string(SMAPS_PATH).ok()
    };
    select_page_size(rollup.as_deref(), smaps.as_deref())
}

pub fn parse_status_switches(content: &str) -> (u64, u64) {
    let mut vol = 0;
    let mut invol = 0;
    for line in content.lines() {
        if let Some(rest) = line.strip_prefix("voluntary_ctxt_switches:") {
            vol = rest.trim().parse().unwrap_or(0);
        } else if let Some(rest) = line.strip_prefix("nonvoluntary_ctxt_switches:") {
            invol = rest.trim().parse().unwrap_or(0);
        }
    }
    (vol, invol)
}

pub fn parse_stat_times(content: &str) -> Option<(u64, u64)> {
    let (_, fields) = content.rsplit_once(") ")?;
    let mut fields = fields.split_whitespace();
    let user = fields.nth(11)?.parse().ok()?;
    let kernel = fields.next()?.parse().ok()?;
    Some((user, kernel))
}

#[cfg(test)]
mod tests {
    use super::{parse_kernel_page_size, parse_statm, select_page_size};

    #[test]
    fn parse_kernel_page_size_reads_smaps_field() {
        let content = "Rss: 1024 kB\nKernelPageSize: 16 kB\nMMUPageSize: 16 kB\n";
        assert_eq!(parse_kernel_page_size(content), Some(16_384));
    }

    #[test]
    fn parse_kernel_page_size_rejects_missing_or_invalid_values() {
        assert_eq!(parse_kernel_page_size("Rss: 1024 kB\n"), None);
        assert_eq!(parse_kernel_page_size("KernelPageSize: invalid kB\n"), None);
        assert_eq!(parse_kernel_page_size("KernelPageSize: 0 kB\n"), None);
        let mixed = "KernelPageSize: 0 kB\nKernelPageSize: 4 kB\n";
        assert_eq!(parse_kernel_page_size(mixed), Some(4096));
    }

    #[test]
    fn select_page_size_prefers_rollup_then_smaps() {
        let rollup = "KernelPageSize: 16 kB\n";
        let smaps = "KernelPageSize: 64 kB\n";
        assert_eq!(select_page_size(Some(rollup), Some(smaps)), 16_384);
        assert_eq!(select_page_size(None, Some(smaps)), 65_536);
        assert_eq!(select_page_size(Some("invalid"), Some(smaps)), 65_536);
        assert_eq!(select_page_size(None, None), 4096);
    }

    #[test]
    fn parse_statm_saturates_page_product() {
        let content = "1 18446744073709551615 0 0 0 0 0\n";
        assert_eq!(parse_statm(content, 4096).ok(), Some(u64::MAX));
    }
}
