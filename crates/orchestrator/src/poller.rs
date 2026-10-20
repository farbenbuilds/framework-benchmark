use crate::error::OrchestratorError;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

const MAX_READY_RESPONSE_BYTES: usize = 2048;

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
    let mut response = [0_u8; MAX_READY_RESPONSE_BYTES];
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
            return is_success_response(&response[..used]);
        }
    }
    false
}

fn is_success_response(response: &[u8]) -> bool {
    let Some(line_end) = response.iter().position(|byte| *byte == b'\n') else {
        return false;
    };
    let mut parts = response[..line_end].split(|byte| *byte == b' ');
    let Some(version) = parts.next() else {
        return false;
    };
    if !version.starts_with(b"HTTP/") {
        return false;
    }
    let Some(status) = parts.next() else {
        return false;
    };
    let Ok(status) = std::str::from_utf8(status) else {
        return false;
    };
    status
        .parse::<u16>()
        .is_ok_and(|code| (200..300).contains(&code))
}

#[cfg(test)]
mod tests {
    use super::is_success_response;

    #[test]
    fn any_2xx_status_is_ready() {
        assert!(is_success_response(b"HTTP/1.1 200 OK\r\n\r\n"));
        assert!(is_success_response(b"HTTP/1.0 204 No Content\r\n\r\n"));
        assert!(!is_success_response(b"HTTP/1.1 503 Busy\r\n\r\n"));
        assert!(!is_success_response(b"garbage\r\n\r\n"));
    }
}
