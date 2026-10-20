use crate::error::OrchestratorError;
use common::manifest::{FrameworkManifest, parse_manifest};
use std::fs;
use std::path::{Path, PathBuf};

const MAX_DISCOVERY_DEPTH: usize = 16;

pub fn discover_frameworks(
    root: &Path,
) -> Result<Vec<(PathBuf, FrameworkManifest)>, OrchestratorError> {
    let mut discovered = Vec::new();
    find_manifests_recursive(root, &mut discovered, 0)?;
    discovered.sort_unstable_by(|left, right| left.0.cmp(&right.0));
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
    depth: usize,
) -> Result<(), OrchestratorError> {
    if depth > MAX_DISCOVERY_DEPTH {
        return Err(OrchestratorError::Process(
            "framework discovery exceeded the maximum directory depth".into(),
        ));
    }
    if !dir.is_dir() {
        return Ok(());
    }

    let entries = fs::read_dir(dir)?;
    for entry in entries {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            let path = entry.path();
            let manifest_file = path.join("framework.toml");
            if let Ok(metadata) = fs::symlink_metadata(&manifest_file) {
                if metadata.file_type().is_symlink() {
                    continue;
                }
                if metadata.is_file() {
                    let content = fs::read_to_string(&manifest_file)?;
                    let manifest = parse_manifest(&content)?;
                    results.push((path, manifest));
                    continue;
                }
            }
            find_manifests_recursive(&path, results, depth + 1)?;
        }
    }
    Ok(())
}
