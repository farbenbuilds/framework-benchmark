use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReporterError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("common crate error: {0}")]
    Common(#[from] common::CommonError),

    #[error("formatting error: {0}")]
    Formatting(String),
}
