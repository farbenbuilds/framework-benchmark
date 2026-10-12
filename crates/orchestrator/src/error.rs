use thiserror::Error;

#[derive(Debug, Error)]
pub enum OrchestratorError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("common crate error: {0}")]
    Common(#[from] common::CommonError),

    #[error("load generator error: {0}")]
    LoadGen(#[from] load_gen::LoadGenError),

    #[error("readiness timeout: service failed to respond within {0} seconds")]
    ReadinessTimeout(u64),

    #[error("process exit error: {0}")]
    Process(String),
}
