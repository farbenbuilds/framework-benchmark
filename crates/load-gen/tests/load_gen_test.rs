use load_gen::histogram::LatencyRecorder;
use load_gen::stats::WorkerStats;

#[test]
fn test_latency_recorder_quantiles() {
    let mut recorder = LatencyRecorder::new().expect("create latency histogram");
    recorder.record_micros(100);
    recorder.record_micros(200);
    recorder.record_micros(300);
    recorder.record_micros(400);
    recorder.record_micros(500);

    let dist = recorder.export_distribution();
    assert_eq!(dist.min_us, 100);
    assert_eq!(dist.max_us, 500);
    assert!(dist.p50_us >= 200);
}

#[test]
fn test_latency_recorder_coordinated_omission() {
    let mut recorder = LatencyRecorder::new().expect("create latency histogram");
    recorder.record_micros_corrected(10_000, 1_000);

    let dist = recorder.export_distribution();
    assert!(dist.min_us <= 1_000);
    assert!(dist.max_us >= 10_000);
}

#[test]
fn test_worker_stats_aggregation() {
    let mut stats1 = WorkerStats::default();
    stats1.record_response(200, 100);
    stats1.record_response(404, 50);

    let mut stats2 = WorkerStats::default();
    stats2.record_response(500, 120);
    stats2.record_error();

    stats1.merge(&stats2);
    assert_eq!(stats1.requests_completed, 3);
    assert_eq!(stats1.status_2xx, 1);
    assert_eq!(stats1.status_4xx, 1);
    assert_eq!(stats1.status_5xx, 1);
    assert_eq!(stats1.errors, 1);
    assert_eq!(stats1.bytes_transferred, 270);
}
