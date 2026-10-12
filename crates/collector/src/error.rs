use thiserror::Error;

#[derive(Debug, Error)]
pub enum CollectorError {
    #[error("I/O error reading system telemetry: {0}")]
    Io(#[from] std::io::Error),

    #[error("failed to parse /proc or cgroup field: {0}")]
    Parse(String),

    #[error("process {0} not found")]
    ProcessNotFound(u32),
}
