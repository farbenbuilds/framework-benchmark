use crate::autobahn::builder::FrameBuilder;
use crate::autobahn::echo::validate_echo_frame;
use crate::autobahn::frame::{
    FrameOpcode, MAX_FRAME_HEADER_BYTES, MAX_FRAME_PAYLOAD_BYTES, parse_frame_header,
};
use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::io::{read_until, write_until};
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

        let mut buf = vec![0; MAX_FRAME_PAYLOAD_BYTES + MAX_FRAME_HEADER_BYTES];
        match read_echo_frame(
            stream,
            text.as_bytes(),
            &mut buf,
            Instant::now() + Duration::from_secs(1),
        )
        .await
        {
            Ok(_) => Ok(true),
            Err(LoadGenError::Protocol(_)) => Ok(false),
            Err(error) => Err(error),
        }
    }

    pub async fn run_echo_benchmark(
        addr: &str,
        path: &str,
        deadline: Instant,
        recorder: &mut LatencyRecorder,
        stats: &mut WorkerStats,
    ) -> Result<(), LoadGenError> {
        let mut stream = match TcpStream::connect(addr).await {
            Ok(stream) => stream,
            Err(error) => {
                stats.record_error();
                return Err(error.into());
            }
        };
        let _ = stream.set_nodelay(true);
        if let Err(error) = Self::perform_handshake(&mut stream, addr, path).await {
            stats.record_error();
            return Err(error);
        }

        let msg = "framework-benchmark-echo";
        let mask = [0x12, 0x34, 0x56, 0x78];
        let frame = FrameBuilder::build_text_message(msg, mask);
        let mut buf = vec![0; MAX_FRAME_PAYLOAD_BYTES + MAX_FRAME_HEADER_BYTES];

        while Instant::now() < deadline {
            let start = Instant::now();
            if let Err(error) = write_until(&mut stream, &frame, deadline).await {
                stats.record_error();
                return Err(error);
            }
            let frame_len =
                match read_echo_frame(&mut stream, msg.as_bytes(), &mut buf, deadline).await {
                    Ok(frame_len) => frame_len,
                    Err(error) => {
                        stats.record_error();
                        return Err(error);
                    }
                };
            let elapsed_us = start.elapsed().as_micros() as u64;
            recorder.record_micros(elapsed_us);
            stats.record_response(200, frame_len);
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

async fn read_echo_frame(
    stream: &mut TcpStream,
    expected: &[u8],
    buf: &mut [u8],
    deadline: Instant,
) -> Result<usize, LoadGenError> {
    let mut used = 0;
    loop {
        if let Some(frame_len) = validate_echo_frame(&buf[..used], expected)? {
            return Ok(frame_len);
        }
        if used == buf.len() {
            return Err(LoadGenError::Protocol(
                "websocket echo exceeds read buffer".into(),
            ));
        }
        let read = read_until(stream, &mut buf[used..], deadline).await?;
        if read == 0 {
            return Err(LoadGenError::Protocol(
                "connection closed before websocket echo".into(),
            ));
        }
        used += read;
    }
}
