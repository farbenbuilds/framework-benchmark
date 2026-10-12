use crate::error::ReporterError;
use common::result::RunResult;
use std::fs;
use std::path::Path;

pub fn write_result_json(output_path: &Path, result: &RunResult) -> Result<(), ReporterError> {
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = common::result::serialize_result_pretty(result)?;
    fs::write(output_path, json)?;
    Ok(())
}
