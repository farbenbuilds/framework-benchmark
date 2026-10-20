use crate::error::LoadGenError;
use crate::histogram::LatencyRecorder;
use crate::io::{connect_until, read_until, retry_after_error, wait_until, write_until};
use crate::pacing::next_issue_at;
use crate::plan::ConnectionMode;
use crate::response::{MAX_RESPONSE_BYTES, parse_response};
use crate::stats::WorkerStats;
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
struct WorkerTask<'a> {
    deadline: Instant,
    pipeline_depth: usize,
    connection_mode: ConnectionMode,
    expected_interval: Option<Duration>,
    next_issue: Instant,
    recorder: &'a mut LatencyRecorder,
    stats: &'a mut WorkerStats,
}
struct ResponseProgress {
    completed: usize,
    consumed: usize,
    closes_connection: bool,
}
pub(crate) struct WorkerBuffers {
    read_buf: Vec<u8>,
    starts: Vec<Instant>,
}
pub(crate) fn preallocate_worker_buffers(pipeline_depth: usize) -> WorkerBuffers {
    WorkerBuffers {
        read_buf: vec![0; MAX_RESPONSE_BYTES],
        starts: Vec::with_capacity(pipeline_depth),
    }
}
#[expect(
    clippy::too_many_arguments,
    reason = "hot path passes borrowed state explicitly"
)]
pub(crate) async fn run_connection_worker_with_buffers(
    target_addr: &str,
    request_bytes: &[u8],
    deadline: Instant,
    pipeline_depth: usize,
    connection_mode: ConnectionMode,
    expected_interval_us: u64,
    recorder: &mut LatencyRecorder,
    stats: &mut WorkerStats,
    mut buffers: WorkerBuffers,
) -> Result<(), LoadGenError> {
    let mut task = WorkerTask {
        deadline,
        pipeline_depth,
        connection_mode,
        expected_interval: (expected_interval_us > 0)
            .then(|| Duration::from_micros(expected_interval_us)),
        next_issue: Instant::now(),
        recorder,
        stats,
    };
    while Instant::now() < task.deadline {
        let stream = connect_until(target_addr, task.deadline).await;
        let Ok(mut stream) = stream else {
            if Instant::now() >= task.deadline {
                return Ok(());
            }
            task.stats.record_error();
            retry_after_error(task.deadline).await;
            continue;
        };
        let _ = stream.set_nodelay(true);
        let result = process_connection_loop(
            &mut stream,
            request_bytes,
            &mut buffers.read_buf,
            &mut buffers.starts,
            &mut task,
        )
        .await;
        if result.is_err() && Instant::now() < task.deadline {
            task.stats.record_error();
        }
    }
    Ok(())
}
async fn process_connection_loop(
    stream: &mut TcpStream,
    request_bytes: &[u8],
    read_buf: &mut [u8],
    starts: &mut Vec<Instant>,
    task: &mut WorkerTask<'_>,
) -> Result<(), LoadGenError> {
    let mut used = 0;
    while Instant::now() < task.deadline {
        if used != 0 {
            return Err(LoadGenError::Protocol(
                "unexpected bytes after response batch".into(),
            ));
        }
        starts.clear();
        for _ in 0..task.pipeline_depth {
            if !wait_until(task.next_issue, task.deadline).await {
                return Ok(());
            }
            let start = Instant::now();
            write_until(stream, request_bytes, task.deadline).await?;
            starts.push(start);
            task.next_issue = next_issue_at(task.expected_interval, task.next_issue, task.deadline);
        }
        used = read_batch(stream, read_buf, used, starts, task).await?;
        if task.connection_mode == ConnectionMode::Churn {
            return Ok(());
        }
    }
    Ok(())
}

async fn read_batch(
    stream: &mut TcpStream,
    read_buf: &mut [u8],
    mut used: usize,
    starts: &[Instant],
    task: &mut WorkerTask<'_>,
) -> Result<usize, LoadGenError> {
    let mut completed = 0;
    while completed < starts.len() {
        let progress = record_responses(&read_buf[..used], &starts[completed..], task)?;
        completed += progress.completed;
        if progress.consumed > 0 {
            read_buf.copy_within(progress.consumed..used, 0);
            used -= progress.consumed;
        }
        if progress.closes_connection && completed == starts.len() {
            return Ok(0);
        }
        if progress.closes_connection {
            return Err(LoadGenError::Protocol(
                "connection closed before pipeline drained".into(),
            ));
        }
        if completed == starts.len() {
            return Ok(used);
        }
        if used == read_buf.len() {
            return Err(LoadGenError::Protocol("response too large".into()));
        }
        let read = read_until(stream, &mut read_buf[used..], task.deadline).await?;
        if read == 0 {
            return Err(LoadGenError::Protocol("connection closed".into()));
        }
        used += read;
    }
    Ok(used)
}

fn record_responses(
    bytes: &[u8],
    starts: &[Instant],
    task: &mut WorkerTask<'_>,
) -> Result<ResponseProgress, LoadGenError> {
    let mut completed = 0;
    let mut consumed = 0;
    let mut closes_connection = false;
    while completed < starts.len() {
        let Some(response) = parse_response(&bytes[consumed..])? else {
            break;
        };
        consumed += response.len;
        if (100..200).contains(&response.status) {
            continue;
        }
        let elapsed = starts[completed].elapsed().as_micros() as u64;
        let interval = task
            .expected_interval
            .map_or(0, |value| value.as_micros() as u64);
        task.recorder.record_micros_corrected(elapsed, interval);
        task.stats.record_response(response.status, response.len);
        completed += 1;
        closes_connection = response.closes_connection;
        if closes_connection {
            break;
        }
    }
    Ok(ResponseProgress {
        completed,
        consumed,
        closes_connection,
    })
}
