use common::result::{
    LatencyDistribution, LibcVariant, MachineInfo, ResourceUsage, RunResult, StatusCodeCounts,
    TestMetrics,
};
use reporter::chart::generate_throughput_svg;
use reporter::index::{HistoricalIndex, update_historical_index};
use reporter::logic::aggregate_leaderboard;
use reporter::markdown::render_markdown_table;
use std::fs;

fn create_dummy_run(name: &str, libc: LibcVariant, rps: f64, p99_us: u64) -> RunResult {
    RunResult {
        run_id: "test-run".into(),
        framework_name: name.into(),
        framework_version: "1.0".into(),
        libc,
        timestamp_epoch_secs: 1_700_000_000,
        machine: MachineInfo {
            hostname: "benchmark-box".into(),
            os_release: "linux".into(),
            kernel_version: "6.6".into(),
            cpu_model: "synthetic".into(),
            physical_cores: 8,
            logical_cores: 16,
            total_memory_mb: 16384,
        },
        benchmarks: vec![TestMetrics {
            category: "plaintext".into(),
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
                p99_us,
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
fn test_leaderboard_ranking_and_markdown() {
    let r1 = create_dummy_run("axum", LibcVariant::Glibc, 500_000.0, 500);
    let r2 = create_dummy_run("nethttp", LibcVariant::Musl, 400_000.0, 800);

    let rows = aggregate_leaderboard(&[r2, r1], "plaintext");
    assert_eq!(rows.len(), 2);
    assert_eq!(rows[0].framework, "axum");
    assert_eq!(rows[0].rank, 1);
    assert_eq!(rows[1].framework, "nethttp");
    assert_eq!(rows[1].rank, 2);

    let markdown = render_markdown_table(&rows, "plaintext");
    assert!(markdown.contains("axum"));
    assert!(markdown.contains("500000.00"));
}

#[test]
fn test_svg_chart_generation() {
    let r1 = create_dummy_run("axum", LibcVariant::Glibc, 500_000.0, 500);
    let rows = aggregate_leaderboard(&[r1], "plaintext");
    let svg = generate_throughput_svg(&rows, "Plaintext Throughput");

    assert!(svg.contains("<svg"));
    assert!(svg.contains("axum-glibc"));
    assert!(svg.contains("500000 req/s"));
    assert!(svg.ends_with("</svg>\n"));
}

#[test]
fn test_historical_index_update() {
    let temp_dir = std::env::temp_dir().join("bench_index_test");
    let index_file = temp_dir.join("index.json");
    if index_file.exists() {
        let _ = fs::remove_file(&index_file);
    }

    let r1 = create_dummy_run("axum", LibcVariant::Glibc, 500_000.0, 500);
    update_historical_index(&index_file, &r1, "results/data/2026/10/test-run.json")
        .expect("update index failed");

    let content = fs::read_to_string(&index_file).expect("read index failed");
    let index: HistoricalIndex = serde_json::from_str(&content).expect("parse index failed");
    assert_eq!(index.runs.len(), 1);
    assert_eq!(index.runs[0].framework_name, "axum");
    assert_eq!(index.runs[0].libc, "glibc");

    let _ = fs::remove_file(index_file);
    let _ = fs::remove_dir(temp_dir);
}
