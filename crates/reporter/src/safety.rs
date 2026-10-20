use crate::error::ReporterError;
use std::ffi::OsStr;
use std::path::{Component, Path};

const UNSAFE_COMPONENT: &str =
    "value must contain only ASCII letters, digits, hyphens, and underscores";
const UNSAFE_PATH: &str = "path must be a normal relative path";

pub fn safe_component(value: &str) -> Result<&str, ReporterError> {
    if value.is_empty()
        || value == "."
        || value == ".."
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
    {
        return Err(ReporterError::Formatting(UNSAFE_COMPONENT.into()));
    }
    Ok(value)
}

pub fn safe_file_name(value: &str, suffix: &str) -> Result<String, ReporterError> {
    Ok(format!("{}{suffix}", safe_component(value)?))
}

pub fn safe_relative_path(path: &Path) -> Result<&Path, ReporterError> {
    let mut found_component = false;
    for component in path.components() {
        let Component::Normal(name) = component else {
            return Err(ReporterError::Formatting(UNSAFE_PATH.into()));
        };
        if !safe_relative_name(name) {
            return Err(ReporterError::Formatting(UNSAFE_PATH.into()));
        }
        found_component = true;
    }

    if !found_component {
        return Err(ReporterError::Formatting(UNSAFE_PATH.into()));
    }
    Ok(path)
}

fn safe_relative_name(name: &OsStr) -> bool {
    name.to_str().is_some_and(|name| {
        !name.is_empty()
            && name != "."
            && name != ".."
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    })
}

#[cfg(test)]
mod tests {
    use super::{safe_component, safe_file_name, safe_relative_path};
    use std::path::Path;

    #[test]
    fn safe_component_accepts_ascii_identifiers() {
        assert_eq!(
            safe_component("plaintext-2_ws").ok(),
            Some("plaintext-2_ws")
        );
    }

    #[test]
    fn safe_component_rejects_empty_dots_and_separators() {
        for value in [
            "",
            ".",
            "..",
            "a/b",
            "a\\b",
            "../x",
            "plain text",
            "pl@in",
            "caf\u{e9}",
        ] {
            assert!(safe_component(value).is_err(), "accepted {value:?}");
        }
    }

    #[test]
    fn safe_file_name_keeps_safe_category_names() {
        assert_eq!(
            safe_file_name("plaintext", "_throughput.svg")
                .ok()
                .as_deref(),
            Some("plaintext_throughput.svg")
        );
        assert!(safe_file_name("../../etc/passwd", "_throughput.svg").is_err());
    }

    #[test]
    fn safe_relative_path_accepts_existing_index_entries() {
        for value in [
            "results/data/2026/10/1700000000-test-run.json",
            "data/2026/10/1700000000-test-run.json",
        ] {
            assert_eq!(
                safe_relative_path(Path::new(value)).ok(),
                Some(Path::new(value))
            );
        }
    }

    #[test]
    fn safe_relative_path_rejects_non_normal_paths() {
        for value in [
            "",
            "/etc/passwd",
            "../outside.json",
            "data/../../outside.json",
            "./data/run.json",
            "data\\..\\outside.json",
            "data/run id.json",
            "data/\u{e9}.json",
        ] {
            assert!(
                safe_relative_path(Path::new(value)).is_err(),
                "accepted {value:?}"
            );
        }
    }
}
