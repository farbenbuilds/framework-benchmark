use common::result::{
    LatencyDistribution, LibcVariant, MachineInfo, ResourceUsage, RunProvenance, RunResult,
    StatusCodeCounts, TestMetrics, TrafficSettings, deserialize_result,
};
use reporter::archive::{
    timestamped_result_path, timestamped_result_relative_path, write_result_json,
};
use reporter::chart::generate_throughput_svg;
use reporter::index::{HistoricalIndex, update_historical_index};
use reporter::logic::aggregate_leaderboard;
use reporter::markdown::render_markdown_table;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

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
        provenance: RunProvenance::default(),
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
            traffic: TrafficSettings {
                pipeline_depth: 1,
                connection_mode: "keep_alive".into(),
                target_rps: None,
            },
        }],
    }
}

fn temporary_dir(label: &str) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after unix epoch")
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "reporter-{label}-{}-{timestamp}",
        std::process::id()
    ));
    fs::create_dir_all(&path).expect("create test directory failed");
    path
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
    let temp_dir = temporary_dir("index");
    let index_file = temp_dir.join("index.json");

    let r1 = create_dummy_run("axum", LibcVariant::Glibc, 500_000.0, 500);
    update_historical_index(&index_file, &r1, "results/data/2026/10/test-run.json")
        .expect("update index failed");
    update_historical_index(
        &index_file,
        &r1,
        "results/data/2026/10/retried-test-run.json",
    )
    .expect("repeat index update failed");

    let content = fs::read_to_string(&index_file).expect("read index failed");
    let index: HistoricalIndex = serde_json::from_str(&content).expect("parse index failed");
    assert_eq!(
        index.runs.len(),
        1,
        "the same run id must not duplicate history"
    );
    assert_eq!(index.runs[0].framework_name, "axum");
    assert_eq!(index.runs[0].libc, "glibc");
    assert_eq!(
        index.runs[0].relative_json_path,
        "results/data/2026/10/retried-test-run.json"
    );

    fs::remove_dir_all(temp_dir).expect("remove test directory failed");
}

#[test]
fn test_timestamped_artifact_path_rejects_collision() {
    let temp_dir = temporary_dir("archive");
    let run = create_dummy_run("axum", LibcVariant::Glibc, 500_000.0, 500);
    let path = timestamped_result_path(&temp_dir, &run).expect("build artifact path failed");

    assert!(path.ends_with("data/2023/11/1700000000-test-run.json"));
    let mut later_run = run.clone();
    later_run.timestamp_epoch_secs += 1;
    let later_path =
        timestamped_result_path(&temp_dir, &later_run).expect("build later artifact path failed");
    assert_ne!(path, later_path);
    write_result_json(&path, &run).expect("write artifact failed");
    let persisted = fs::read_to_string(&path).expect("read artifact failed");
    assert_eq!(
        deserialize_result(&persisted).expect("parse artifact failed"),
        run
    );
    let mut conflicting_run = run.clone();
    conflicting_run.framework_name = "changed".into();
    let error =
        write_result_json(&path, &conflicting_run).expect_err("artifact collision must fail");

    assert!(matches!(
        error,
        reporter::ReporterError::Io(error) if error.kind() == std::io::ErrorKind::AlreadyExists
    ));
    let persisted = fs::read_to_string(&path).expect("read immutable artifact failed");
    assert_eq!(
        deserialize_result(&persisted).expect("parse artifact failed"),
        run
    );
    fs::remove_dir_all(temp_dir).expect("remove test directory failed");
}

#[test]
fn test_timestamped_artifact_path_rejects_unsafe_run_id() {
    let mut run = create_dummy_run("axum", LibcVariant::Glibc, 500_000.0, 500);
    run.run_id = "../outside".into();

    let error = timestamped_result_relative_path(&run).expect_err("unsafe run id must fail");

    assert!(error.to_string().contains("run id"));
}
