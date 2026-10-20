use crate::error::ReporterError;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn write_new_file(path: &Path, contents: &[u8]) -> Result<(), ReporterError> {
    let parent = parent_path(path)?;
    fs::create_dir_all(parent)?;
    let (mut file, temporary_path) = create_temporary_file(path, parent)?;
    let result = write_and_publish_new(&mut file, &temporary_path, path, parent, contents);
    let cleanup = cleanup_temporary_file(&temporary_path);
    result?;
    cleanup?;
    Ok(())
}

pub(crate) fn replace_file(path: &Path, contents: &[u8]) -> Result<(), ReporterError> {
    let parent = parent_path(path)?;
    fs::create_dir_all(parent)?;
    let (mut file, temporary_path) = create_temporary_file(path, parent)?;
    let result = write_and_replace(&mut file, &temporary_path, path, parent, contents);
    let cleanup = cleanup_temporary_file(&temporary_path);
    result?;
    cleanup?;
    Ok(())
}

fn write_and_publish_new(
    file: &mut File,
    temporary_path: &Path,
    path: &Path,
    parent: &Path,
    contents: &[u8],
) -> Result<(), ReporterError> {
    file.write_all(contents)?;
    file.sync_all()?;
    fs::hard_link(temporary_path, path)?;
    sync_directory(parent)
}

fn write_and_replace(
    file: &mut File,
    temporary_path: &Path,
    path: &Path,
    parent: &Path,
    contents: &[u8],
) -> Result<(), ReporterError> {
    file.write_all(contents)?;
    file.sync_all()?;
    fs::rename(temporary_path, path)?;
    sync_directory(parent)
}

fn parent_path(path: &Path) -> Result<&Path, ReporterError> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .ok_or_else(|| ReporterError::Formatting("output path must have a parent directory".into()))
}

fn create_temporary_file(path: &Path, parent: &Path) -> Result<(File, PathBuf), ReporterError> {
    let file_name = path.file_name().and_then(OsStr::to_str).ok_or_else(|| {
        ReporterError::Formatting("output path must have a UTF-8 file name".into())
    })?;

    for attempt in 0..64 {
        let temporary_path = parent.join(format!(
            ".{file_name}.{}.{}.tmp",
            std::process::id(),
            attempt
        ));
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&temporary_path)
        {
            Ok(file) => return Ok((file, temporary_path)),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error.into()),
        }
    }

    Err(ReporterError::Formatting(
        "could not reserve a temporary output path".into(),
    ))
}

fn sync_directory(path: &Path) -> Result<(), ReporterError> {
    File::open(path)?.sync_all()?;
    Ok(())
}

fn cleanup_temporary_file(path: &Path) -> Result<(), ReporterError> {
    match fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}
