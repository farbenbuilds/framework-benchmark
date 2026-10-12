#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PerfMode {
    Hardware,
    SoftwareFallback,
    Unavailable,
}

pub fn parse_perf_event_paranoid(content: &str) -> Option<i32> {
    content.trim().parse().ok()
}

pub fn select_perf_mode(paranoid: Option<i32>, has_perfmon_cap: bool) -> PerfMode {
    if has_perfmon_cap || paranoid.is_some_and(|value| value <= 1) {
        return PerfMode::Hardware;
    }
    if paranoid.is_some() {
        return PerfMode::SoftwareFallback;
    }
    PerfMode::Unavailable
}

pub fn parse_perf_stat_value(content: &str) -> Option<u64> {
    content
        .trim()
        .split(',')
        .next()
        .map(str::trim)
        .filter(|value| *value != "<not supported>" && *value != "<not counted>")?
        .parse()
        .ok()
}
