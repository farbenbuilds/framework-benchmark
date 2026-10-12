use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::stats::WorkerStats;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub async fn run_connection_worker(
    target_addr: &str,
    request_bytes: &[u8],
    deadline: Instant,
    pipeline_depth: usize,
    recorder: &mut LatencyRecorder,
    stats: &mut WorkerStats,
) -> Result<(), LoadGenError> {
    let mut stream = TcpStream::connect(target_addr).await?;
    let _ = stream.set_nodelay(true);
    let mut read_buf = vec![0u8; 16384];

    while Instant::now() < deadline {
        let req_start = Instant::now();
        for _ in 0..pipeline_depth {
            stream.write_all(request_bytes).await?;
        }

        for _ in 0..pipeline_depth {
            let n = stream.read(&mut read_buf).await?;
            if n == 0 {
                return Ok(());
            }
            let elapsed_us = req_start.elapsed().as_micros() as u64;
            recorder.record_micros(elapsed_us);

            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut res = httparse::Response::new(&mut headers);
            if let Ok(httparse::Status::Complete(_)) = res.parse(&read_buf[..n]) {
                stats.record_response(res.code.unwrap_or(200), n);
            } else {
                stats.record_response(200, n);
            }
        }
    }
    Ok(())
}
