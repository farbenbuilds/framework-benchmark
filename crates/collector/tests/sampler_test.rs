use collector::BackgroundSampler;
use std::fs;
use std::time::Duration;

#[tokio::test]
async fn test_background_sampler_lifecycle() {
    let sampler = BackgroundSampler::spawn_simulated(Duration::from_millis(10), 1024 * 1024);
    tokio::time::sleep(Duration::from_millis(50)).await;
    let usage = sampler.stop().await;

    assert_eq!(usage.peak_rss_bytes, 1024 * 1024);
    assert_eq!(usage.avg_rss_bytes, 1024 * 1024);
}

#[tokio::test]
async fn target_sampler_should_combine_process_and_cgroup_memory() {
    let path = std::env::temp_dir().join(format!("collector-sampler-{}", std::process::id()));
    let peak = 17_179_869_184_u64;
    fs::create_dir_all(&path).expect("create cgroup fixture directory");
    fs::write(path.join("memory.current"), "1024\n").expect("write current memory fixture");
    fs::write(path.join("memory.peak"), format!("{peak}\n")).expect("write peak memory fixture");
    fs::write(
        path.join("cpu.stat"),
        "usage_usec 10\nuser_usec 7\nsystem_usec 3\n",
    )
    .expect("write cpu fixture");

    let sampler = BackgroundSampler::spawn_target(
        Some(std::process::id()),
        Some(&path),
        Duration::from_millis(1),
    );
    tokio::time::sleep(Duration::from_millis(10)).await;
    let usage = sampler.stop().await;
    fs::remove_dir_all(&path).expect("remove cgroup fixture directory");

    assert!(usage.peak_rss_bytes < peak && usage.avg_rss_bytes > 1024);
}
