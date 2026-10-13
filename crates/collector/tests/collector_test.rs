use collector::aggregator::HarvestAggregator;
use collector::cgroups::{parse_cgroup_cpu_stat, parse_cgroup_memory_stat};
use collector::procfs::{parse_stat_times, parse_statm, parse_status_switches};
use collector::{
    EbpfMetrics, PerfMode, energy_delta_uj, page_cache_hit_ratio, parse_ebpf_metrics,
    parse_perf_event_paranoid, select_perf_mode,
};
use common::result::ResourceUsage;

#[test]
fn test_parse_procfs_statm() {
    let statm_sample = "1000 500 200 10 0 40 0\n";
    let rss = parse_statm(statm_sample, 4096).expect("parse statm failed");
    assert_eq!(rss, 500 * 4096);
}

#[test]
fn test_parse_procfs_status_switches() {
    let status_sample =
        "Name:\tsome_proc\nvoluntary_ctxt_switches:\t1234\nnonvoluntary_ctxt_switches:\t56\n";
    let (vol, invol) = parse_status_switches(status_sample);
    assert_eq!(vol, 1234);
    assert_eq!(invol, 56);
}

#[test]
fn test_parse_procfs_stat_times() {
    let stat = "42 (server worker) S 0 0 0 0 0 0 0 0 0 0 12 34 0 0 0\n";
    assert_eq!(parse_stat_times(stat), Some((12, 34)));
}

#[test]
fn test_parse_cgroup_v2_stats() {
    let memory_sample = "10485760\n";
    let bytes = parse_cgroup_memory_stat(memory_sample).expect("cgroup memory parse failed");
    assert_eq!(bytes, 10_485_760);

    let cpu_sample = "usage_usec 1000000\nuser_usec 800000\nsystem_usec 200000\n";
    let (user, sys) = parse_cgroup_cpu_stat(cpu_sample);
    assert_eq!(user, 800_000);
    assert_eq!(sys, 200_000);
}

#[test]
fn test_harvest_aggregator_metrics() {
    let mut agg = HarvestAggregator::default();
    agg.record_sample(10_000_000);
    agg.record_sample(20_000_000);
    agg.record_sample(30_000_000);
    agg.cpu_user_pct = 65.0;
    agg.cpu_kernel_pct = 15.0;
    agg.context_switches = 2000;

    let res = agg.finalize();
    assert_eq!(res.peak_rss_bytes, 30_000_000);
    assert_eq!(res.avg_rss_bytes, 20_000_000);
    assert!((res.cpu_user_pct - 65.0).abs() < f32::EPSILON);
    assert_eq!(res.context_switches, 2000);
}

#[test]
fn test_energy_counter_wrap() {
    assert_eq!(energy_delta_uj(90, 20, 100), 30);
    assert_eq!(energy_delta_uj(20, 90, 100), 70);
}

#[test]
fn test_perf_fallback_selection() {
    assert_eq!(parse_perf_event_paranoid("2\n"), Some(2));
    assert_eq!(select_perf_mode(Some(2), false), PerfMode::SoftwareFallback);
    assert_eq!(select_perf_mode(Some(3), true), PerfMode::Hardware);
}

#[test]
fn test_ebpf_metric_parsing() {
    let metrics = parse_ebpf_metrics("off_cpu_usec 10\npage_cache_hits 4\npage_cache_misses 1\n");
    assert_eq!(metrics.off_cpu_usec, 10);
    assert_eq!(page_cache_hit_ratio(metrics), Some(0.8));
    assert_eq!(page_cache_hit_ratio(EbpfMetrics::default()), None);
}

#[test]
fn test_runtime_counter_delta() {
    let mut usage = ResourceUsage {
        peak_rss_bytes: 0,
        avg_rss_bytes: 0,
        cpu_user_pct: 0.0,
        cpu_kernel_pct: 0.0,
        context_switches: 0,
        rapl_joules: None,
    };
    collector::metrics::apply_runtime_counters(
        &mut usage,
        collector::metrics::RuntimeCounters {
            user_usec: 10,
            kernel_usec: 20,
            context_switches: 30,
        },
        collector::metrics::RuntimeCounters {
            user_usec: 510,
            kernel_usec: 270,
            context_switches: 42,
        },
        1_000,
    );
    assert!((usage.cpu_user_pct - 50.0).abs() < f32::EPSILON);
    assert!((usage.cpu_kernel_pct - 25.0).abs() < f32::EPSILON);
    assert_eq!(usage.context_switches, 12);
}
