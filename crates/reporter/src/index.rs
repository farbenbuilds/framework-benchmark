use crate::error::ReporterError;
use crate::storage::replace_file;
use common::result::RunResult;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HistoricalIndex {
    pub runs: Vec<IndexEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexEntry {
    pub run_id: String,
    pub framework_name: String,
    pub libc: String,
    pub timestamp_epoch_secs: u64,
    pub relative_json_path: String,
}

fn historical_index_entry(result: &RunResult, relative_path: &str) -> IndexEntry {
    IndexEntry {
        run_id: result.run_id.clone(),
        framework_name: result.framework_name.clone(),
        libc: libc_name(result.libc).into(),
        timestamp_epoch_secs: result.timestamp_epoch_secs,
        relative_json_path: relative_path.into(),
    }
}

fn append_historical_index(index: &mut HistoricalIndex, entry: IndexEntry) {
    index.runs.push(entry);
}

pub fn update_historical_index(
    index_file_path: &Path,
    result: &RunResult,
    relative_path: &str,
) -> Result<(), ReporterError> {
    let mut index: HistoricalIndex = if index_file_path.exists() {
        let content = fs::read_to_string(index_file_path)?;
        serde_json::from_str(&content)
            .map_err(|error| ReporterError::Formatting(format!("failed to parse index: {error}")))?
    } else {
        if let Some(parent) = index_file_path.parent() {
            fs::create_dir_all(parent)?;
        }
        HistoricalIndex::default()
    };

    append_historical_index(&mut index, historical_index_entry(result, relative_path));

    let serialized = serde_json::to_string_pretty(&index)
        .map_err(|e| ReporterError::Formatting(format!("failed to serialize index: {e}")))?;
    replace_file(index_file_path, serialized.as_bytes())
}

fn libc_name(libc: common::result::LibcVariant) -> &'static str {
    match libc {
        common::result::LibcVariant::Glibc => "glibc",
        common::result::LibcVariant::Musl => "musl",
    }
}
