use crate::error::OrchestratorError;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};

pub fn framework_input_sha256(root: &Path) -> Result<String, OrchestratorError> {
    let mut files = Vec::new();
    collect_files(root, root, &mut files)?;
    files.sort_unstable();

    let mut digest = Sha256::new();
    for path in files {
        let relative = path.strip_prefix(root).map_err(|error| {
            OrchestratorError::Process(format!("framework input path error: {error}"))
        })?;
        digest.update(relative.as_os_str().as_encoded_bytes());
        digest.update([0]);
        digest.update(fs::read(path)?);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn collect_files(
    root: &Path,
    directory: &Path,
    files: &mut Vec<PathBuf>,
) -> Result<(), OrchestratorError> {
    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        let path = entry.path();
        if excluded(root, &path) {
            continue;
        }
        if entry.file_type()?.is_dir() {
            collect_files(root, &path, files)?;
        } else if entry.file_type()?.is_file() {
            files.push(path);
        }
    }
    Ok(())
}

fn excluded(root: &Path, path: &Path) -> bool {
    path.strip_prefix(root)
        .ok()
        .and_then(|relative| relative.components().next())
        .is_some_and(|component| matches!(component.as_os_str().to_str(), Some(".git" | "target")))
}

#[cfg(test)]
mod tests {
    use super::framework_input_sha256;
    use std::fs;

    #[test]
    fn framework_digest_is_stable_and_excludes_build_outputs() {
        let root = std::env::temp_dir().join(format!("provenance-{}", std::process::id()));
        fs::create_dir_all(root.join("target")).unwrap();
        fs::write(root.join("framework.toml"), "name = 'test'").unwrap();
        fs::write(root.join("target/output"), "first").unwrap();
        let first = framework_input_sha256(&root).unwrap();
        fs::write(root.join("target/output"), "second").unwrap();
        assert_eq!(framework_input_sha256(&root).unwrap(), first);
        fs::write(root.join("framework.toml"), "name = 'changed'").unwrap();
        assert_ne!(framework_input_sha256(&root).unwrap(), first);
        fs::remove_dir_all(root).unwrap();
    }
}
