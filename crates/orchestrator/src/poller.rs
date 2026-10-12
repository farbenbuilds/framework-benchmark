use crate::error::OrchestratorError;
use std::time::{Duration, Instant};
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;

pub async fn wait_for_ready(
    target_addr: &str,
    healthcheck_path: &str,
    timeout_secs: u64,
) -> Result<(), OrchestratorError> {
    let deadline = Instant::now() + Duration::from_secs(timeout_secs);
    let mut backoff_ms = 50;

    let probe_req = format!(
        "GET {healthcheck_path} HTTP/1.1\r\nHost: {target_addr}\r\nConnection: close\r\n\r\n"
    );

    while Instant::now() < deadline {
        if let Ok(mut stream) = TcpStream::connect(target_addr).await {
            if stream.write_all(probe_req.as_bytes()).await.is_ok() {
                return Ok(());
            }
        }
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
        backoff_ms = (backoff_ms * 2).min(1000);
    }

    Err(OrchestratorError::ReadinessTimeout(timeout_secs))
}
