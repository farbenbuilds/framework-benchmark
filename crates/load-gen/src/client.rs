use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::stats::WorkerStats;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

struct WorkerTask<'a> {
    deadline: Instant,
    pipeline_depth: usize,
    expected_interval: Option<Duration>,
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
        expected_interval: (expected_interval_us > 0)
            .then(|| Duration::from_micros(expected_interval_us)),
        recorder,
        stats,
    };
    let mut read_buf = Vec::with_capacity(32_768);

    while Instant::now() < task.deadline {
        let Ok(mut stream) = TcpStream::connect(target_addr).await else {
            task.stats.record_error();
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            continue;
        };

        let _ = stream.set_nodelay(true);

        if process_connection_loop(&mut stream, request_bytes, &mut read_buf, &mut task)
            .await
            .is_err()
        {
            task.stats.record_error();
        }
    }
    Ok(())
}

async fn process_connection_loop(
    stream: &mut TcpStream,
    request_bytes: &[u8],
    read_buf: &mut Vec<u8>,
    task: &mut WorkerTask<'_>,
) -> Result<(), LoadGenError> {
    let depth = task.pipeline_depth.max(1);
    let mut next_issue = Instant::now();
    let mut starts = Vec::with_capacity(depth);

    while Instant::now() < task.deadline {
        wait_until(next_issue).await;
        if Instant::now() >= task.deadline {
            return Ok(());
        }

        starts.clear();
        for _ in 0..depth {
            let req_start = Instant::now();
            stream.write_all(request_bytes).await?;
            starts.push(req_start);
            if let Some(interval) = task.expected_interval {
                next_issue += interval;
            }
        }

        let mut completed = 0;
        while completed < depth {
            let before = read_buf.len();
            read_buf.resize(before + 16_384, 0);
            let n = stream.read(&mut read_buf[before..]).await?;
            read_buf.truncate(before + n);
            if n == 0 {
                return Ok(());
            }
            completed += record_responses(read_buf, &starts[completed..], task)?;
        }
    }
    Ok(())
}

async fn wait_until(when: Instant) {
    if when > Instant::now() {
        tokio::time::sleep_until(tokio::time::Instant::from_std(when)).await;
    }
}

fn record_responses(
    bytes: &mut Vec<u8>,
    starts: &[Instant],
    task: &mut WorkerTask<'_>,
) -> Result<usize, LoadGenError> {
    let mut offset = 0;
    let mut completed = 0;

    while completed < starts.len() {
        let Some((status, response_len)) = parse_response(&bytes[offset..])? else {
            break;
        };
        let elapsed = starts[completed].elapsed().as_micros() as u64;
        let interval = task
            .expected_interval
            .map_or(0, |value| value.as_micros() as u64);
        task.recorder.record_micros_corrected(elapsed, interval);
        task.stats.record_response(status, response_len);
        offset += response_len;
        completed += 1;
    }
    if offset > 0 {
        bytes.drain(..offset);
    }
    Ok(completed)
}

fn parse_response(bytes: &[u8]) -> Result<Option<(u16, usize)>, LoadGenError> {
    let mut headers = [httparse::EMPTY_HEADER; 16];
    let (header_len, status) = {
        let mut response = httparse::Response::new(&mut headers);
        let httparse::Status::Complete(header_len) = response
            .parse(bytes)
            .map_err(|error| LoadGenError::Protocol(error.to_string()))?
        else {
            return Ok(None);
        };
        (header_len, response.code.unwrap_or(200))
    };
    let body_len = headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case("content-length"))
        .map_or(Ok(0), |header| {
            std::str::from_utf8(header.value)
                .map_err(|error| LoadGenError::Protocol(error.to_string()))?
                .trim()
                .parse::<usize>()
                .map_err(|error| LoadGenError::Protocol(error.to_string()))
        })?;
    let response_len = header_len + body_len;
    if bytes.len() < response_len {
        return Ok(None);
    }
    Ok(Some((status, response_len)))
}
