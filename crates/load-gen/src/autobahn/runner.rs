use crate::autobahn::builder::FrameBuilder;
use crate::autobahn::echo::validate_echo_frame;
use crate::autobahn::frame::{
    FrameOpcode, MAX_FRAME_HEADER_BYTES, MAX_FRAME_PAYLOAD_BYTES, parse_frame_header,
};
use crate::autobahn::handshake::perform_handshake;
use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::io::{connect_until, read_until, write_until};
use crate::stats::WorkerStats;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

#[derive(Debug)]
pub struct AutobahnSuiteRunner;

impl AutobahnSuiteRunner {
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
        let mut stream = match connect_until(addr, deadline).await {
            Ok(stream) => stream,
            Err(error) => {
                stats.record_error();
                return Err(error);
            }
        };
        let _ = stream.set_nodelay(true);
        if let Err(error) = perform_handshake(&mut stream, addr, path, deadline).await {
            return finish_or_error(error, deadline, stats);
        }

        let msg = "framework-benchmark-echo";
        let mask = [0x12, 0x34, 0x56, 0x78];
        let frame = FrameBuilder::build_text_message(msg, mask);
        let mut buf = vec![0; MAX_FRAME_PAYLOAD_BYTES + MAX_FRAME_HEADER_BYTES];

        while Instant::now() < deadline {
            let start = Instant::now();
            if let Err(error) = write_until(&mut stream, &frame, deadline).await {
                return finish_or_error(error, deadline, stats);
            }
            let frame_len =
                match read_echo_frame(&mut stream, msg.as_bytes(), &mut buf, deadline).await {
                    Ok(frame_len) => frame_len,
                    Err(error) => return finish_or_error(error, deadline, stats),
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

fn finish_or_error(
    error: LoadGenError,
    deadline: Instant,
    stats: &mut WorkerStats,
) -> Result<(), LoadGenError> {
    if Instant::now() >= deadline {
        return Ok(());
    }
    stats.record_error();
    Err(error)
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

#[cfg(test)]
mod tests {
    use super::finish_or_error;
    use crate::error::LoadGenError;
    use crate::stats::WorkerStats;
    use std::time::{Duration, Instant};

    #[test]
    fn expiry_is_not_a_server_error() {
        let mut stats = WorkerStats::default();
        let deadline = Instant::now()
            .checked_sub(Duration::from_millis(1))
            .expect("test deadline");
        assert!(finish_or_error(LoadGenError::Timeout, deadline, &mut stats).is_ok());
        assert_eq!(stats.errors, 0);
    }

    #[test]
    fn early_failure_is_recorded() {
        let mut stats = WorkerStats::default();
        let deadline = Instant::now() + Duration::from_secs(1);
        assert!(finish_or_error(LoadGenError::Timeout, deadline, &mut stats).is_err());
        assert_eq!(stats.errors, 1);
    }
}
