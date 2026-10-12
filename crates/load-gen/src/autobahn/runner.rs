use crate::autobahn::builder::FrameBuilder;
use crate::autobahn::frame::{FrameOpcode, parse_frame_header};
use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::stats::WorkerStats;
use std::time::{Duration, Instant};
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

    pub async fn run_echo_benchmark(
        addr: &str,
        path: &str,
        deadline: Instant,
        recorder: &mut LatencyRecorder,
        stats: &mut WorkerStats,
    ) -> Result<(), LoadGenError> {
        let mut stream = TcpStream::connect(addr).await?;
        let _ = stream.set_nodelay(true);
        Self::perform_handshake(&mut stream, addr, path).await?;

        let msg = "framework-benchmark-echo";
        let mask = [0x12, 0x34, 0x56, 0x78];
        let frame = FrameBuilder::build_text_message(msg, mask);
        let mut buf = vec![0u8; 4096];

        while Instant::now() < deadline {
            let start = Instant::now();
            stream.write_all(&frame).await?;
            let n = stream.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            let elapsed_us = start.elapsed().as_micros() as u64;
            recorder.record_micros(elapsed_us);
            stats.record_response(200, n);
        }
        Ok(())
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
