use crate::error::OrchestratorError;
use std::fs;
use std::path::{Path, PathBuf};

pub(crate) fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|character| {
            if character.is_ascii_lowercase() || character.is_ascii_digit() {
                return character;
            }
            if character.is_ascii_uppercase() {
                return character.to_ascii_lowercase();
            }
            '-'
        })
        .collect()
}

pub(crate) fn secure_dockerfile(root: &Path, relative: &str) -> Result<PathBuf, OrchestratorError> {
    let root = fs::canonicalize(root)?;
    let candidate = root.join(relative);
    let metadata = fs::symlink_metadata(&candidate)?;
    if metadata.file_type().is_symlink() {
        return Err(OrchestratorError::Process(
            "dockerfile must not be a symbolic link".into(),
        ));
    }
    let path = fs::canonicalize(&candidate)?;
    if !path.starts_with(&root) || !path.is_file() {
        return Err(OrchestratorError::Process(
            "dockerfile must remain inside the framework directory".into(),
        ));
    }
    Ok(path)
}

#[cfg(all(test, unix))]
mod tests {
    use super::{sanitize_name, secure_dockerfile};
    use std::fs;
    use std::os::unix::fs::symlink;

    #[test]
    fn dockerfile_cannot_escape_or_use_a_symlink() {
        let root = std::env::temp_dir().join(format!("dockerfile-root-{}", std::process::id()));
        let outside = std::env::temp_dir().join(format!("dockerfile-out-{}", std::process::id()));
        fs::create_dir_all(&root).expect("create root");
        fs::create_dir_all(&outside).expect("create outside");
        fs::write(root.join("Dockerfile.glibc"), "FROM scratch\n").expect("write dockerfile");
        fs::write(outside.join("Dockerfile.glibc"), "FROM scratch\n").expect("write outside");
        symlink(
            outside.join("Dockerfile.glibc"),
            root.join("Dockerfile.musl"),
        )
        .expect("create symlink");

        assert!(secure_dockerfile(&root, "Dockerfile.glibc").is_ok());
        assert!(secure_dockerfile(&root, "Dockerfile.musl").is_err());
        assert!(secure_dockerfile(&root, "../Dockerfile.glibc").is_err());

        let _ = fs::remove_dir_all(root);
        let _ = fs::remove_dir_all(outside);
    }

    #[test]
    fn container_names_are_lowercase_and_safe() {
        assert_eq!(sanitize_name("Rust/Axum"), "rust-axum");
    }
}
