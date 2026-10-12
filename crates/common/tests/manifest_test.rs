use common::manifest::parse_manifest;

const VALID_TOML: &str = r#"
[framework]
name = "axum"
version = "0.8"
language = "rust"
language_version = "1.85"
website = "https://github.com/tokio-rs/axum"
category = "web-server"
tier = "micro"

[capabilities]
plaintext = true
json = true
db_single = true
db_multi = false
fortunes = false
updates = false
websocket = false
http2 = true
http3 = false

[build]
engine = "docker"

[build.docker]
dockerfile_glibc = "Dockerfile.glibc"
dockerfile_musl = "Dockerfile.musl"

[runtime]
port = 8080
healthcheck_path = "/healthz"
startup_timeout_seconds = 30
shutdown_signal = "SIGTERM"
"#;

#[test]
fn test_parse_valid_manifest() {
    let manifest = parse_manifest(VALID_TOML).expect("failed to parse valid manifest");
    assert_eq!(manifest.framework.name, "axum");
    assert_eq!(manifest.framework.language, "rust");
    assert!(manifest.capabilities.plaintext);
    assert!(manifest.capabilities.json);
    assert!(!manifest.capabilities.websocket);
    assert_eq!(manifest.runtime.port, 8080);
    assert_eq!(manifest.runtime.healthcheck_path, "/healthz");
}

#[test]
fn test_reject_invalid_port() {
    let invalid_toml = VALID_TOML.replace("port = 8080", "port = 0");
    let err = parse_manifest(&invalid_toml).unwrap_err();
    assert!(err.to_string().contains("runtime port cannot be 0"));
}

#[test]
fn test_reject_empty_name() {
    let invalid_toml = VALID_TOML.replace("name = \"axum\"", "name = \"\"");
    let err = parse_manifest(&invalid_toml).unwrap_err();
    assert!(err.to_string().contains("framework name cannot be empty"));
}

#[test]
fn test_reject_invalid_engine() {
    let invalid_toml = VALID_TOML.replace("engine = \"docker\"", "engine = \"kubernetes\"");
    let err = parse_manifest(&invalid_toml).unwrap_err();
    assert!(
        err.to_string()
            .contains("build engine must be 'docker' or 'nix'")
    );
}
