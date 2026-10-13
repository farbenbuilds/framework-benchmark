use crate::LoadGenError;
use crate::io::{read_until, write_until};
use std::time::Instant;
use tokio::net::TcpStream;

const ACCEPT: &[u8] = b"s3pPLMBiTxaQ9kYGzzhZRbK+xOo=";

pub(crate) async fn perform_handshake(
    stream: &mut TcpStream,
    addr: &str,
    path: &str,
    deadline: Instant,
) -> Result<(), LoadGenError> {
    let request = format!(
        "GET {path} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
    );
    write_until(stream, request.as_bytes(), deadline).await?;
    let mut response = [0_u8; 2048];
    let mut used = 0;
    loop {
        if let Some(end) = header_end(&response[..used]) {
            return valid_handshake(&response[..end])
                .then_some(())
                .ok_or_else(|| LoadGenError::Protocol("invalid websocket handshake".into()));
        }
        if used == response.len() {
            return Err(LoadGenError::Protocol(
                "websocket handshake is too large".into(),
            ));
        }
        let read = read_until(stream, &mut response[used..], deadline).await?;
        if read == 0 {
            return Err(LoadGenError::Protocol("websocket handshake closed".into()));
        }
        used += read;
    }
}

fn valid_handshake(response: &[u8]) -> bool {
    let Some(status_end) = response.iter().position(|byte| *byte == b'\n') else {
        return false;
    };
    let status = &response[..status_end];
    let headers = &response[status_end + 1..];
    if !status.trim_ascii().starts_with(b"HTTP/1.1 101 ") {
        return false;
    }
    header_has_token(headers, b"upgrade", b"websocket")
        && header_has_token(headers, b"connection", b"upgrade")
        && header_value(headers, b"sec-websocket-accept") == Some(ACCEPT)
}

fn header_end(bytes: &[u8]) -> Option<usize> {
    bytes
        .windows(4)
        .position(|part| part == b"\r\n\r\n")
        .map(|end| end + 4)
}

fn header_has_token(headers: &[u8], name: &[u8], token: &[u8]) -> bool {
    header_value(headers, name).is_some_and(|value| {
        value
            .split(|byte| *byte == b',')
            .any(|part| part.trim_ascii().eq_ignore_ascii_case(token))
    })
}

fn header_value<'a>(headers: &'a [u8], name: &[u8]) -> Option<&'a [u8]> {
    headers.split(|byte| *byte == b'\n').find_map(|line| {
        let colon = line.iter().position(|byte| *byte == b':')?;
        let key = &line[..colon];
        let value = &line[colon + 1..];
        key.trim_ascii()
            .eq_ignore_ascii_case(name)
            .then_some(value.trim_ascii())
    })
}

#[cfg(test)]
mod tests {
    use super::valid_handshake;

    #[test]
    fn handshake_requires_all_websocket_upgrade_headers() {
        let valid = b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=\r\n\r\n";
        let missing_accept =
            b"HTTP/1.1 101 Switching Protocols\r\nUpgrade: websocket\r\nConnection: Upgrade\r\n\r\n";
        assert!(valid_handshake(valid));
        assert!(!valid_handshake(missing_accept));
    }
}
