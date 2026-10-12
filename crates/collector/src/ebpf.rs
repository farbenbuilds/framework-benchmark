#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EbpfMetrics {
    pub off_cpu_usec: u64,
    pub page_cache_hits: u64,
    pub page_cache_misses: u64,
    pub context_switches: u64,
}

pub fn parse_ebpf_metrics(content: &str) -> EbpfMetrics {
    let mut metrics = EbpfMetrics::default();
    for line in content.lines() {
        let Some((name, value)) = line.split_once(' ') else {
            continue;
        };
        let Ok(value) = value.trim().parse() else {
            continue;
        };
        match name {
            "off_cpu_usec" => metrics.off_cpu_usec = value,
            "page_cache_hits" => metrics.page_cache_hits = value,
            "page_cache_misses" => metrics.page_cache_misses = value,
            "context_switches" => metrics.context_switches = value,
            _ => (),
        }
    }
    metrics
}

pub fn page_cache_hit_ratio(metrics: EbpfMetrics) -> Option<f64> {
    let total = metrics.page_cache_hits + metrics.page_cache_misses;
    (total > 0).then(|| metrics.page_cache_hits as f64 / total as f64)
}
