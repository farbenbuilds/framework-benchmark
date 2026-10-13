use crate::error::OrchestratorError;
use common::manifest::{FrameworkManifest, parse_manifest};
use std::fs;
use std::path::{Path, PathBuf};

pub fn discover_frameworks(
    root: &Path,
) -> Result<Vec<(PathBuf, FrameworkManifest)>, OrchestratorError> {
    let mut discovered = Vec::new();
    find_manifests_recursive(root, &mut discovered)?;
    Ok(discovered)
}

pub fn select_frameworks(
    frameworks: Vec<(PathBuf, FrameworkManifest)>,
    filter: &str,
) -> Result<Vec<(PathBuf, FrameworkManifest)>, OrchestratorError> {
    let filter = filter.trim();
    if filter.is_empty() || filter == "all" {
        return Ok(frameworks);
    }
    let selected: Vec<_> = frameworks
        .into_iter()
        .filter(|(_, manifest)| manifest.framework.name == filter)
        .collect();
    if selected.is_empty() {
        return Err(OrchestratorError::Process(format!(
            "no framework matches BENCHMARK_FRAMEWORK_FILTER={filter}"
        )));
    }
    Ok(selected)
}

fn find_manifests_recursive(
    dir: &Path,
    results: &mut Vec<(PathBuf, FrameworkManifest)>,
) -> Result<(), OrchestratorError> {
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let manifest_file = path.join("framework.toml");
            if manifest_file.is_file() {
                let content = fs::read_to_string(&manifest_file)?;
                let manifest = parse_manifest(&content)?;
                results.push((path, manifest));
                continue;
            }
            find_manifests_recursive(&path, results)?;
        }
    }
    Ok(())
}
