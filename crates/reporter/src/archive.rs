use crate::error::ReporterError;
use crate::storage::write_new_file;
use common::result::RunResult;
use std::path::{Path, PathBuf};

pub fn timestamped_result_path(
    results_dir: &Path,
    result: &RunResult,
) -> Result<PathBuf, ReporterError> {
    Ok(results_dir.join(timestamped_result_relative_path(result)?))
}

pub fn timestamped_result_relative_path(result: &RunResult) -> Result<PathBuf, ReporterError> {
    if !valid_run_id(&result.run_id) {
        return Err(ReporterError::Formatting(
            "run id must contain only ASCII letters, digits, hyphens, and underscores".into(),
        ));
    }

    let (year, month) = year_month(result.timestamp_epoch_secs);
    Ok(PathBuf::from("data")
        .join(format!("{year:04}"))
        .join(format!("{month:02}"))
        .join(format!(
            "{}-{}.json",
            result.timestamp_epoch_secs, result.run_id
        )))
}

pub fn write_result_json(output_path: &Path, result: &RunResult) -> Result<(), ReporterError> {
    let json = common::result::serialize_result_pretty(result)?;
    write_new_file(output_path, json.as_bytes())
}

fn valid_run_id(run_id: &str) -> bool {
    !run_id.is_empty()
        && run_id
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_')
}

fn year_month(timestamp_epoch_secs: u64) -> (u64, u64) {
    let days = timestamp_epoch_secs / 86_400 + 719_468;
    let era = days / 146_097;
    let day_of_era = days - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    (year, month)
}
