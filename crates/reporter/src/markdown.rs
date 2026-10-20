use crate::error::ReporterError;
use crate::logic::{aggregate_leaderboard, dual_libc_deltas};
use crate::storage::replace_file;
use common::result::RunResult;
use std::fs;
use std::path::Path;

mod tables;

pub use tables::{render_markdown_table, render_markdown_table_with_deltas};

const START_MARKER: &str = "<!-- BENCHMARK_RESULTS_START -->";
const END_MARKER: &str = "<!-- BENCHMARK_RESULTS_END -->";

pub fn render_readme_leaderboards(results: &[RunResult]) -> String {
    let mut categories: Vec<&str> = results
        .iter()
        .flat_map(|result| result.benchmarks.iter().map(|test| test.category.as_str()))
        .collect();
    categories.sort_unstable();
    categories.dedup();

    if categories.is_empty() {
        return "\n*No benchmark results recorded yet.*\n".into();
    }

    let mut section = String::from("\n\n");
    for category in categories {
        let rows = aggregate_leaderboard(results, category);
        let deltas = dual_libc_deltas(results, category);
        section.push_str(&render_markdown_table_with_deltas(&rows, &deltas, category));
        section.push('\n');
    }
    section
}

pub fn update_readme_leaderboard(
    readme_path: &Path,
    results: &[RunResult],
) -> Result<(), ReporterError> {
    if results.is_empty() || !readme_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(readme_path)?;
    let Some(raw_start) = content.find(START_MARKER) else {
        return Ok(());
    };
    let Some(end_pos) = content.find(END_MARKER) else {
        return Ok(());
    };
    let start_pos = raw_start + START_MARKER.len();
    if end_pos < start_pos {
        return Err(ReporterError::Formatting(
            "README benchmark markers are in reverse order".into(),
        ));
    }

    let section = render_readme_leaderboards(results);
    let mut new_content = String::with_capacity(content.len() + section.len());
    new_content.push_str(&content[..start_pos]);
    new_content.push_str(&section);
    new_content.push_str(&content[end_pos..]);
    replace_file(readme_path, new_content.as_bytes())
}
