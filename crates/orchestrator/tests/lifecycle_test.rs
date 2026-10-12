use common::manifest::parse_manifest;
use common::result::LibcVariant;
use orchestrator::lifecycle::run_framework_suite;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_lifecycle_suite_execution() {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();

    let server_handle = tokio::spawn(async move {
        while let Ok((mut socket, _)) = listener.accept().await {
            tokio::spawn(async move {
                let mut buf = [0u8; 1024];
                while let Ok(n) = socket.read(&mut buf).await {
                    if n == 0 {
                        break;
                    }
                    let resp = b"HTTP/1.1 200 OK\r\nContent-Length: 13\r\nConnection: keep-alive\r\n\r\nHello, World!";
                    if socket.write_all(resp).await.is_err() {
                        break;
                    }
                }
            });
        }
    });

    let manifest_toml = format!(
        r#"
[framework]
name = "mock"
version = "0.1"
language = "rust"
language_version = "1.85"
category = "web-server"
tier = "micro"

[capabilities]
plaintext = true
json = false
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
port = {}
healthcheck_path = "/healthz"
startup_timeout_seconds = 2
"#,
        addr.port()
    );

    let manifest = parse_manifest(&manifest_toml).unwrap();
    let result = run_framework_suite(&manifest, &addr.to_string(), 4, 1, LibcVariant::Glibc)
        .await
        .unwrap();

    assert_eq!(result.framework_name, "mock");
    assert_eq!(result.benchmarks.len(), 1);
    assert_eq!(result.benchmarks[0].category, "plaintext");
    assert!(result.benchmarks[0].requests_per_sec > 0.0);

    server_handle.abort();
}
