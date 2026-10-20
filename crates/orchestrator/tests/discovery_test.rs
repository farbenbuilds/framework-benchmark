use orchestrator::discovery::{discover_frameworks, select_frameworks};
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
fn select_frameworks_rejects_an_unknown_name() {
    let temp_root = std::env::temp_dir().join("bench_discovery_filter_test");
    let target_dir = temp_root.join("rust").join("sample");
    fs::create_dir_all(&target_dir).expect("create test directory failed");
    fs::write(target_dir.join("framework.toml"), SAMPLE_MANIFEST).expect("write manifest failed");

    let frameworks = discover_frameworks(&temp_root).expect("discover framework registry failed");
    assert_eq!(select_frameworks(frameworks, "sample").unwrap().len(), 1);
    let frameworks = discover_frameworks(&temp_root).expect("discover framework registry failed");
    assert!(select_frameworks(frameworks, "missing").is_err());

    let _ = fs::remove_dir_all(temp_root);
}

#[cfg(unix)]
#[test]
fn test_discovery_does_not_follow_symlinked_directories() {
    use std::os::unix::fs::symlink;

    let root = std::env::temp_dir().join(format!("bench_disc_root_{}", std::process::id()));
    let outside = std::env::temp_dir().join(format!("bench_disc_out_{}", std::process::id()));
    let target = outside.join("linked");
    fs::create_dir_all(&target).expect("create outside target");
    fs::write(target.join("framework.toml"), SAMPLE_MANIFEST).expect("write manifest");
    fs::create_dir_all(&root).expect("create root");
    symlink(&outside, root.join("escape")).expect("create symlink");

    let discovered = discover_frameworks(&root).expect("discovery failed");
    assert!(discovered.is_empty());

    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(outside);
}

#[test]
fn test_registry_manifests_are_valid() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../frameworks");
    let discovered = discover_frameworks(&root).expect("discover framework registry failed");
    assert!(discovered.len() >= 31);
}
