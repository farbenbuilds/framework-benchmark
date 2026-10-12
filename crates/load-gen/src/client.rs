use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::stats::WorkerStats;
use std::time::Instant;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct WorkerTask<'a> {
    deadline: Instant,
    pipeline_depth: usize,
    expected_interval_us: u64,
    recorder: &'a mut LatencyRecorder,
    stats: &'a mut WorkerStats,
}

pub async fn run_connection_worker(
    target_addr: &str,
    request_bytes: &[u8],
    deadline: Instant,
    pipeline_depth: usize,
    expected_interval_us: u64,
    recorder: &mut LatencyRecorder,
    stats: &mut WorkerStats,
) -> Result<(), LoadGenError> {
    let mut task = WorkerTask {
        deadline,
        pipeline_depth,
        expected_interval_us,
        recorder,
        stats,
    };
    let mut read_buf = vec![0u8; 16384];

    while Instant::now() < task.deadline {
        let Ok(stream) = TcpStream::connect(target_addr).await else {
            task.stats.record_error();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            continue;
        };

        let _ = stream.set_nodelay(true);
        let mut stream = stream;

        let res =
            process_connection_loop(&mut stream, request_bytes, &mut read_buf, &mut task).await;

        if res.is_err() {
            task.stats.record_error();
        }
    }
    Ok(())
}

async fn process_connection_loop(
    stream: &mut TcpStream,
    request_bytes: &[u8],
    read_buf: &mut [u8],
    task: &mut WorkerTask<'_>,
) -> Result<(), LoadGenError> {
    let depth = task.pipeline_depth.max(1);

    while Instant::now() < task.deadline {
        let req_start = Instant::now();
        for _ in 0..depth {
            stream.write_all(request_bytes).await?;
        }

        for _ in 0..depth {
            let n = stream.read(read_buf).await?;
            if n == 0 {
                return Ok(());
            }

            let elapsed_us = req_start.elapsed().as_micros() as u64;
            task.recorder
                .record_micros_corrected(elapsed_us, task.expected_interval_us);

            let mut headers = [httparse::EMPTY_HEADER; 16];
            let mut res = httparse::Response::new(&mut headers);
            let code = match res.parse(&read_buf[..n]) {
                Ok(httparse::Status::Complete(_)) => res.code.unwrap_or(200),
                _ => 200,
            };

            task.stats.record_response(code, n);

            let mut should_close = false;
            for h in &headers {
                if h.name.eq_ignore_ascii_case("connection")
                    && h.value.eq_ignore_ascii_case(b"close")
                {
                    should_close = true;
                    break;
                }
            }
            if should_close {
                return Ok(());
            }
        }
    }
    Ok(())
}
