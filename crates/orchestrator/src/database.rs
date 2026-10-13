use crate::error::OrchestratorError;
use std::path::{Path, PathBuf};
use std::process::Command;

pub const DEFAULT_DATABASE_URL: &str =
    "postgres://benchmarkdbuser:benchmarkdbpass@127.0.0.1:5432/hello_world";

#[derive(Debug)]
pub struct DatabaseSession {
    runtime: String,
    compose_file: PathBuf,
    started: bool,
}

pub fn start_database(repo_root: &Path) -> Result<Option<DatabaseSession>, OrchestratorError> {
    if !database_management_enabled()? {
        return Ok(None);
    }
    let session = DatabaseSession {
        runtime: std::env::var("CONTAINER_RUNTIME").unwrap_or_else(|_| "docker".into()),
        compose_file: repo_root.join("deploy/postgres/compose.yaml"),
        started: false,
    };
    let was_running = !compose(&session, &["ps", "--quiet", "postgres"])?
        .trim()
        .is_empty();
    if was_running {
        return Ok(Some(session));
    }
    compose(&session, &["up", "--detach", "--wait"])?;
    Ok(Some(DatabaseSession {
        started: true,
        ..session
    }))
}

pub fn stop_database(session: &DatabaseSession) -> Result<(), OrchestratorError> {
    if !session.started {
        return Ok(());
    }
    compose(session, &["down", "--remove-orphans"]).map(|_| ())
}

fn database_management_enabled() -> Result<bool, OrchestratorError> {
    match std::env::var("BENCHMARK_DATABASE_MANAGED").as_deref() {
        Ok("1") | Err(std::env::VarError::NotPresent) => Ok(true),
        Ok("0") => Ok(false),
        Ok(value) => Err(OrchestratorError::Process(format!(
            "BENCHMARK_DATABASE_MANAGED must be 0 or 1, got {value}"
        ))),
        Err(error) => Err(OrchestratorError::Process(error.to_string())),
    }
}

fn compose(session: &DatabaseSession, args: &[&str]) -> Result<String, OrchestratorError> {
    let output = Command::new(&session.runtime)
        .args(["compose", "--file"])
        .arg(&session.compose_file)
        .args(args)
        .output()?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }
    Err(OrchestratorError::Process(
        String::from_utf8_lossy(&output.stderr).trim().into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::DEFAULT_DATABASE_URL;

    #[test]
    fn default_database_url_uses_the_seeded_database() {
        assert!(DEFAULT_DATABASE_URL.ends_with("/hello_world"));
    }
}
