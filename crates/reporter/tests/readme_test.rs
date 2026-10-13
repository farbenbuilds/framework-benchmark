use common::result::{
    LatencyDistribution, LibcVariant, MachineInfo, ResourceUsage, RunResult, StatusCodeCounts,
    TestMetrics,
};
use reporter::{render_readme_leaderboards, update_readme_leaderboard};
use std::fs;

fn create_sample_run(name: &str, category: &str, rps: f64) -> RunResult {
    RunResult {
        run_id: "test-run".into(),
        framework_name: name.into(),
        framework_version: "1.0".into(),
        libc: LibcVariant::Glibc,
        timestamp_epoch_secs: 1_700_000_000,
        machine: MachineInfo {
            hostname: "benchmark-host".into(),
            os_release: "linux".into(),
            kernel_version: "6.6".into(),
            cpu_model: "synthetic".into(),
            physical_cores: 8,
            logical_cores: 16,
            total_memory_mb: 16384,
        },
        benchmarks: vec![TestMetrics {
            category: category.into(),
            concurrency: 64,
            duration_secs: 10,
            requests_total: 100_000,
            requests_per_sec: rps,
            transfer_mb_per_sec: 50.0,
            latency: LatencyDistribution {
                min_us: 100,
                p50_us: 200,
                p75_us: 300,
                p90_us: 400,
                p99_us: 500,
                p99_9_us: 1000,
                p99_99_us: 2000,
                max_us: 5000,
                std_dev_us: 50.0,
            },
            resources: ResourceUsage {
                peak_rss_bytes: 32 * 1024 * 1024,
                avg_rss_bytes: 28 * 1024 * 1024,
                cpu_user_pct: 70.0,
                cpu_kernel_pct: 10.0,
                context_switches: 1000,
                rapl_joules: None,
            },
            status_codes: StatusCodeCounts::default(),
        }],
    }
}

#[test]
fn test_update_readme_leaderboard() {
    let temp_dir = std::env::temp_dir().join("bench_readme_test");
    fs::create_dir_all(&temp_dir).expect("create test dir failed");
    let readme_path = temp_dir.join("README.md");

    let initial = r"# Framework Benchmark
<!-- BENCHMARK_RESULTS_START -->
*Empty*
<!-- BENCHMARK_RESULTS_END -->
";
    fs::write(&readme_path, initial).expect("write initial readme failed");

    let run = create_sample_run("axum", "plaintext", 450_000.0);
    update_readme_leaderboard(&readme_path, &[run]).expect("update readme failed");

    let updated = fs::read_to_string(&readme_path).expect("read updated readme failed");
    assert!(updated.contains("### Benchmark Leaderboard: plaintext"));
    assert!(updated.contains("axum"));
    assert!(updated.contains("450000.00"));

    let _ = fs::remove_dir_all(temp_dir);
}

#[test]
fn test_render_readme_leaderboards_includes_complete_batch() {
    let first = create_sample_run("axum", "plaintext", 450_000.0);
    let second = create_sample_run("actix", "websocket", 400_000.0);

    let rendered = render_readme_leaderboards(&[first, second]);

    assert!(rendered.contains("axum") && rendered.contains("actix"));
}

#[test]
fn test_update_readme_leaderboard_preserves_existing_output_for_empty_batch() {
    let temp_dir = std::env::temp_dir().join("bench_readme_empty_batch_test");
    fs::create_dir_all(&temp_dir).expect("create test dir failed");
    let readme_path = temp_dir.join("README.md");
    let initial = "<!-- BENCHMARK_RESULTS_START -->\nold\n<!-- BENCHMARK_RESULTS_END -->\n";
    fs::write(&readme_path, initial).expect("write initial readme failed");

    update_readme_leaderboard(&readme_path, &[]).expect("empty batch must not fail");

    assert_eq!(
        fs::read_to_string(&readme_path).expect("read updated readme failed"),
        initial
    );
    let _ = fs::remove_dir_all(temp_dir);
}
