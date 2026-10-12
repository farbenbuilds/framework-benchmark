use collector::aggregator::HarvestAggregator;
use collector::cgroups::{parse_cgroup_cpu_stat, parse_cgroup_memory_stat};
use collector::procfs::{parse_statm, parse_status_switches};

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
