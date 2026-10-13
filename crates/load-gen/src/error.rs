use thiserror::Error;

#[derive(Debug, Error)]
pub enum LoadGenError {
    #[error("invalid benchmark plan: {0}")]
    InvalidPlan(String),

    #[error("network I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("histogram error: {0}")]
    Histogram(#[from] hdrhistogram::RecordError),

    #[error("protocol error: {0}")]
    Protocol(String),

    #[error("timeout waiting for response")]
    Timeout,
}
