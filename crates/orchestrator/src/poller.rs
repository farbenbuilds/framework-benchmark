use crate::error::OrchestratorError;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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
        if probe(target_addr, &probe_req, deadline).await {
            return Ok(());
        }
        tokio::time::sleep(Duration::from_millis(backoff_ms)).await;
        backoff_ms = (backoff_ms * 2).min(1000);
    }

    Err(OrchestratorError::ReadinessTimeout(timeout_secs))
}

async fn probe(target_addr: &str, request: &str, deadline: Instant) -> bool {
    let timeout = tokio::time::Instant::from_std(deadline);
    let Ok(Ok(mut stream)) =
        tokio::time::timeout_at(timeout, TcpStream::connect(target_addr)).await
    else {
        return false;
    };
    if stream.write_all(request.as_bytes()).await.is_err() {
        return false;
    }
    let mut response = [0_u8; 512];
    let mut used = 0;
    while used < response.len() {
        let Ok(Ok(read)) =
            tokio::time::timeout_at(timeout, stream.read(&mut response[used..])).await
        else {
            return false;
        };
        if read == 0 {
            return false;
        }
        used += read;
        if response[..used]
            .windows(4)
            .any(|bytes| bytes == b"\r\n\r\n")
        {
            return response[..used].starts_with(b"HTTP/1.1 200 ");
        }
    }
    false
}
