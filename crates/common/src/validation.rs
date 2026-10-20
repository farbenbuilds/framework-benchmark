use crate::error::CommonError;
use crate::manifest::FrameworkManifest;

const MAX_NAME_BYTES: usize = 64;
const MAX_VERSION_BYTES: usize = 64;
const MAX_PATH_BYTES: usize = 256;
const MAX_ENV_BYTES: usize = 1024;

pub fn validate_manifest(manifest: &FrameworkManifest) -> Result<(), CommonError> {
    validate_metadata(manifest)?;
    validate_runtime(manifest)?;
    crate::validation_build::validate_build(manifest)
}

fn validate_metadata(manifest: &FrameworkManifest) -> Result<(), CommonError> {
    let framework = &manifest.framework;
    if framework.name.trim().is_empty() {
        return Err(CommonError::Validation(
            "framework name cannot be empty".into(),
        ));
    }
    if !valid_name(&framework.name) {
        return Err(CommonError::Validation(
            "framework name must contain only lowercase ASCII letters, digits, '-' or '_'".into(),
        ));
    }
    if !valid_version(&framework.version) {
        return Err(CommonError::Validation(
            "framework version contains invalid characters".into(),
        ));
    }
    if framework.language.trim().is_empty() {
        return Err(CommonError::Validation(
            "framework language cannot be empty".into(),
        ));
    }
    if !valid_language(&framework.language) {
        return Err(CommonError::Validation(
            "framework language contains invalid characters".into(),
        ));
    }
    if !valid_text(&framework.language_version, 64) {
        return Err(CommonError::Validation(
            "language_version contains invalid characters".into(),
        ));
    }
    if !valid_name(&framework.category) {
        return Err(CommonError::Validation(
            "category must contain only lowercase ASCII letters, digits, '-' or '_'".into(),
        ));
    }
    if framework
        .tier
        .as_deref()
        .is_some_and(|tier| !valid_name(tier))
    {
        return Err(CommonError::Validation(
            "tier contains invalid characters".into(),
        ));
    }
    if let Some(website) = framework.website.as_deref()
        && (!website.starts_with("https://") || !valid_text(website, MAX_PATH_BYTES))
    {
        return Err(CommonError::Validation(
            "website must be an https URL without whitespace".into(),
        ));
    }
    Ok(())
}

fn validate_runtime(manifest: &FrameworkManifest) -> Result<(), CommonError> {
    let runtime = &manifest.runtime;
    if runtime.port == 0 {
        return Err(CommonError::Validation("runtime port cannot be 0".into()));
    }
    if !valid_endpoint_path(&runtime.healthcheck_path) {
        return Err(CommonError::Validation(
            "healthcheck_path must start with '/' and contain no whitespace".into(),
        ));
    }
    if runtime
        .websocket_path
        .as_deref()
        .is_some_and(|path| !valid_endpoint_path(path))
    {
        return Err(CommonError::Validation(
            "websocket_path must start with '/' and contain no whitespace".into(),
        ));
    }
    if !(1..=600).contains(&runtime.startup_timeout_seconds) {
        return Err(CommonError::Validation(
            "startup_timeout_seconds must be between 1 and 600".into(),
        ));
    }
    if runtime
        .shutdown_signal
        .as_deref()
        .is_some_and(|signal| !valid_text(signal, 32))
    {
        return Err(CommonError::Validation(
            "shutdown_signal contains invalid characters".into(),
        ));
    }
    for (key, value) in &runtime.environment {
        if !valid_env_key(key) || !valid_text(value, MAX_ENV_BYTES) {
            return Err(CommonError::Validation(format!(
                "runtime environment entry is invalid: {key}"
            )));
        }
    }
    Ok(())
}

pub(crate) fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && !value.bytes().any(|byte| byte.is_ascii_control())
}

fn valid_name(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_NAME_BYTES
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_version(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= MAX_VERSION_BYTES
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'+'))
}

fn valid_language(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 32
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'+' | b'#' | b'-')
        })
}

fn valid_endpoint_path(value: &str) -> bool {
    value.starts_with('/')
        && valid_text(value, MAX_PATH_BYTES)
        && !value.bytes().any(|byte| byte.is_ascii_whitespace())
}

pub(crate) fn valid_env_key(value: &str) -> bool {
    let mut bytes = value.bytes();
    let Some(first) = bytes.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == b'_')
        && value.len() <= 128
        && bytes.all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}
