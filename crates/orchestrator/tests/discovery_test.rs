use orchestrator::discovery::discover_frameworks;
use std::fs;

const SAMPLE_MANIFEST: &str = r#"
[framework]
name = "sample"
version = "0.1"
language = "rust"
language_version = "1.85"
category = "web-server"
tier = "micro"

[capabilities]
plaintext = true
json = true
db_single = false
db_multi = false
fortunes = false
updates = false
websocket = false
http2 = false
http3 = false

[build]
engine = "docker"

[build.docker]
dockerfile_glibc = "Dockerfile.glibc"
dockerfile_musl = "Dockerfile.musl"

[runtime]
port = 8080
healthcheck_path = "/healthz"
startup_timeout_seconds = 5
"#;

#[test]
fn test_discover_frameworks_in_directory() {
    let temp_root = std::env::temp_dir().join("bench_disc_test");
    let target_dir = temp_root.join("rust").join("sample");
    fs::create_dir_all(&target_dir).expect("create test dir failed");

    fs::write(target_dir.join("framework.toml"), SAMPLE_MANIFEST).expect("write manifest failed");

    let discovered = discover_frameworks(&temp_root).expect("discovery failed");
    assert_eq!(discovered.len(), 1);
    assert_eq!(discovered[0].1.framework.name, "sample");

    let _ = fs::remove_dir_all(temp_root);
}

#[test]
fn test_discover_empty_directory() {
    let temp_empty = std::env::temp_dir().join("bench_disc_empty");
    fs::create_dir_all(&temp_empty).expect("create empty dir failed");

    let discovered = discover_frameworks(&temp_empty).expect("discovery on empty failed");
    assert_eq!(discovered.len(), 0);

    let _ = fs::remove_dir_all(temp_empty);
}

#[test]
fn test_registry_manifests_are_valid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frameworks");
    let discovered = discover_frameworks(&root).expect("discover framework registry failed");
    assert_eq!(discovered.len(), 32);
}
