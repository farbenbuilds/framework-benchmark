use crate::error::OrchestratorError;
use common::result::RunResult;
use reporter::archive::{
    timestamped_result_path, timestamped_result_relative_path, write_result_json,
};
use reporter::index::update_historical_index;
use std::path::Path;

pub(crate) fn persist_session_results(
    result: &RunResult,
    base_dir: &str,
) -> Result<(), OrchestratorError> {
    let base = Path::new(base_dir);
    let json_path = timestamped_result_path(base, result)?;
    let relative_path = timestamped_result_relative_path(result)?;
    let index_path = base.join("index.json");

    write_result_json(&json_path, result)?;
    update_historical_index(
        &index_path,
        result,
        relative_path
            .to_str()
            .ok_or_else(|| OrchestratorError::Process("result path must be valid UTF-8".into()))?,
    )?;
    Ok(())
}
