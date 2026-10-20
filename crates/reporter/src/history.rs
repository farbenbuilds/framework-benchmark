use crate::error::ReporterError;
use crate::index::HistoricalIndex;
use crate::safety::safe_relative_path;
use common::result::{RunResult, deserialize_result};
use std::fs;
use std::path::Path;

pub fn load_historical_results(results_dir: &Path) -> Result<Vec<RunResult>, ReporterError> {
    let index_path = results_dir.join("index.json");
    if !index_path.exists() {
        return Ok(Vec::new());
    }
    let index: HistoricalIndex = serde_json::from_str(&fs::read_to_string(index_path)?)
        .map_err(|error| ReporterError::Formatting(format!("failed to parse index: {error}")))?;
    index
        .runs
        .iter()
        .map(|entry| load_result(results_dir, &entry.relative_json_path))
        .collect()
}

fn load_result(results_dir: &Path, relative_path: &str) -> Result<RunResult, ReporterError> {
    let path = safe_relative_path(Path::new(relative_path))?;
    let contents = fs::read_to_string(results_dir.join(path))?;
    deserialize_result(&contents).map_err(ReporterError::from)
}

#[cfg(test)]
mod tests {
    use super::load_result;
    use std::path::Path;

    #[test]
    fn historical_paths_cannot_escape_results_directory() {
        for path in [
            "../outside.json",
            "/etc/passwd",
            "data/../../outside.json",
            "./outside.json",
            "",
        ] {
            assert!(
                load_result(Path::new("/tmp"), path).is_err(),
                "accepted {path:?}"
            );
        }
    }

    #[test]
    fn historical_paths_accept_existing_layout() {
        let path = "results/data/2026/10/1700000000-test-run.json";
        let error = load_result(Path::new("/nonexistent-results"), path)
            .expect_err("missing artifact must fail");
        assert!(matches!(error, crate::ReporterError::Io(_)));
    }
}
