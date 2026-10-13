use crate::LoadGenError;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;

pub(crate) async fn connect_until(
    target_addr: &str,
    deadline: Instant,
) -> Result<TcpStream, LoadGenError> {
    tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        TcpStream::connect(target_addr),
    )
    .await
    .map_err(|_| LoadGenError::Timeout)?
    .map_err(LoadGenError::from)
}

pub(crate) async fn retry_after_error(deadline: Instant) {
    let retry_at = (Instant::now() + Duration::from_millis(10)).min(deadline);
    tokio::time::sleep_until(tokio::time::Instant::from_std(retry_at)).await;
}

pub(crate) async fn write_until(
    stream: &mut TcpStream,
    bytes: &[u8],
    deadline: Instant,
) -> Result<(), LoadGenError> {
    tokio::time::timeout_at(
        tokio::time::Instant::from_std(deadline),
        stream.write_all(bytes),
    )
    .await
    .map_err(|_| LoadGenError::Timeout)??;
    Ok(())
}

pub(crate) async fn read_until(
    stream: &mut TcpStream,
    bytes: &mut [u8],
    deadline: Instant,
) -> Result<usize, LoadGenError> {
    tokio::time::timeout_at(tokio::time::Instant::from_std(deadline), stream.read(bytes))
        .await
        .map_err(|_| LoadGenError::Timeout)?
        .map_err(LoadGenError::from)
}

pub(crate) async fn wait_until(when: Instant, deadline: Instant) -> bool {
    let target = when.min(deadline);
    if target > Instant::now() {
        tokio::time::sleep_until(tokio::time::Instant::from_std(target)).await;
    }
    Instant::now() < deadline
}
