use common::result::{
    LatencyDistribution, LibcVariant, MachineInfo, ResourceUsage, RunResult, StatusCodeCounts,
    TestMetrics, deserialize_result, serialize_result_pretty,
};

#[test]
fn test_roundtrip_result_serialization() {
    let result = RunResult {
        run_id: "run-20261012-001".into(),
        framework_name: "axum".into(),
        framework_version: "0.8".into(),
        libc: LibcVariant::Glibc,
        timestamp_epoch_secs: 1_791_848_000,
        machine: MachineInfo {
            hostname: "bench-linux-01".into(),
            os_release: "Ubuntu 24.04 LTS".into(),
            kernel_version: "6.8.0-45-generic".into(),
            cpu_model: "AMD EPYC 9654".into(),
            physical_cores: 96,
            logical_cores: 192,
            total_memory_mb: 262_144,
        },
        benchmarks: vec![TestMetrics {
            category: "plaintext".into(),
            concurrency: 512,
            duration_secs: 30,
            requests_total: 3_000_000,
            requests_per_sec: 100_000.0,
            transfer_mb_per_sec: 12.5,
            latency: LatencyDistribution {
                min_us: 100,
                p50_us: 250,
                p75_us: 400,
                p90_us: 700,
                p99_us: 1500,
                p99_9_us: 3000,
                p99_99_us: 5000,
                max_us: 8000,
                std_dev_us: 150.0,
            },
            resources: ResourceUsage {
                peak_rss_bytes: 45_000_000,
                avg_rss_bytes: 42_000_000,
                cpu_user_pct: 75.5,
                cpu_kernel_pct: 12.3,
                context_switches: 45_000,
            },
            status_codes: StatusCodeCounts {
                http_2xx: 3_000_000,
                http_4xx: 0,
                http_5xx: 0,
                errors: 0,
            },
        }],
    };

    let serialized = serialize_result_pretty(&result).expect("failed to serialize");
    assert!(serialized.contains("run-20261012-001"));
    assert!(serialized.contains("glibc"));

    let deserialized = deserialize_result(&serialized).expect("failed to deserialize");
    assert_eq!(deserialized, result);
}
