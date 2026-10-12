use thiserror::Error;

#[derive(Debug, Error)]
pub enum CommonError {
    #[error("failed to parse TOML manifest: {0}")]
    ManifestTomlParse(#[from] toml::de::Error),

    #[error("failed to parse JSON: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("invalid framework manifest: {0}")]
    InvalidManifest(String),

    #[error("validation error: {0}")]
    Validation(String),
}
