use crate::autobahn::builder::FrameBuilder;
use crate::autobahn::frame::{FrameOpcode, parse_frame_header};
use crate::error::LoadGenError;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct AutobahnSuiteRunner;

impl AutobahnSuiteRunner {
    pub async fn perform_handshake(
        stream: &mut TcpStream,
        addr: &str,
        path: &str,
    ) -> Result<(), LoadGenError> {
        let req = format!(
            "GET {path} HTTP/1.1\r\nHost: {addr}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n"
        );
        stream.write_all(req.as_bytes()).await?;

        let mut buf = vec![0u8; 1024];
        let n = stream.read(&mut buf).await?;
        if n == 0 || !buf[..n].starts_with(b"HTTP/1.1 101") {
            return Err(LoadGenError::Protocol("handshake upgrade rejected".into()));
        }
        Ok(())
    }

    pub async fn test_echo_message(
        stream: &mut TcpStream,
        text: &str,
    ) -> Result<bool, LoadGenError> {
        let mask = [1, 2, 3, 4];
        let frame = FrameBuilder::build_text_message(text, mask);
        stream.write_all(&frame).await?;

        let mut buf = vec![0u8; 4096];
        let n = stream.read(&mut buf).await?;
        if let Some((header, offset)) = parse_frame_header(&buf[..n])? {
            if header.opcode == FrameOpcode::Text {
                let payload = &buf[offset..offset + header.payload_len];
                return Ok(payload == text.as_bytes());
            }
        }
        Ok(false)
    }

    pub async fn test_fuzz_rejection(
        stream: &mut TcpStream,
        fuzz_frame: &[u8],
    ) -> Result<bool, LoadGenError> {
        stream.write_all(fuzz_frame).await?;
        let mut buf = vec![0u8; 1024];

        let timeout = tokio::time::sleep(Duration::from_millis(500));
        tokio::pin!(timeout);

        tokio::select! {
            res = stream.read(&mut buf) => {
                match res {
                    Ok(0) | Err(_) => Ok(true),
                    Ok(n) => {
                        if let Some((header, _)) = parse_frame_header(&buf[..n])? {
                            Ok(header.opcode == FrameOpcode::Close)
                        } else {
                            Ok(false)
                        }
                    }
                }
            }
            () = &mut timeout => Ok(false),
        }
    }
}
