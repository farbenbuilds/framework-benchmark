use collector::BackgroundSampler;
use std::time::Duration;

#[tokio::test]
async fn test_background_sampler_lifecycle() {
    let sampler = BackgroundSampler::spawn_simulated(Duration::from_millis(10), 1024 * 1024);
    tokio::time::sleep(Duration::from_millis(50)).await;
    let usage = sampler.stop().await;

    assert_eq!(usage.peak_rss_bytes, 1024 * 1024);
    assert_eq!(usage.avg_rss_bytes, 1024 * 1024);
}
