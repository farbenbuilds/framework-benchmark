use crate::error::CollectorError;

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
    Ok(pages * page_size)
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
