use common::result::{
    LatencyDistribution, LibcVariant, MachineInfo, ResourceUsage, RunProvenance, RunResult,
    StatusCodeCounts, TestMetrics, TrafficSettings,
};
use reporter::write_leaderboard_artifacts;
use std::fs;

fn result() -> RunResult {
    RunResult {
        run_id: "artifact-test".into(),
        framework_name: "sample".into(),
        framework_version: "1.0".into(),
        libc: LibcVariant::Glibc,
        timestamp_epoch_secs: 1_700_000_000,
        machine: MachineInfo {
            hostname: "test".into(),
            os_release: "test".into(),
            kernel_version: "test".into(),
            cpu_model: "test".into(),
            physical_cores: 2,
            logical_cores: 2,
            total_memory_mb: 8192,
        },
        provenance: RunProvenance::default(),
        benchmarks: vec![TestMetrics {
            category: "plaintext".into(),
            concurrency: 8,
            duration_secs: 10,
            requests_total: 100,
            requests_per_sec: 10.0,
            transfer_mb_per_sec: 0.1,
            latency: LatencyDistribution {
                min_us: 1,
                p50_us: 2,
                p75_us: 3,
                p90_us: 4,
                p99_us: 5,
                p99_9_us: 6,
                p99_99_us: 7,
                max_us: 8,
                std_dev_us: 1.0,
            },
            resources: ResourceUsage::default(),
            status_codes: StatusCodeCounts::default(),
            traffic: TrafficSettings {
                pipeline_depth: 1,
                connection_mode: "keep_alive".into(),
                target_rps: None,
            },
        }],
    }
}

#[test]
fn leaderboard_artifacts_are_written_for_each_category() {
    let base = std::env::temp_dir().join(format!("reporter-artifacts-{}", std::process::id()));
    write_leaderboard_artifacts(&base, &[result()]).expect("write leaderboard artifacts");

    assert!(base.join("charts/plaintext_throughput.svg").is_file());
    assert!(base.join("leaderboard_plaintext.md").is_file());
    fs::remove_dir_all(&base).expect("remove artifact fixture");
}
