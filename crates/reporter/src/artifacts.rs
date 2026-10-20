use crate::chart::generate_throughput_svg;
use crate::error::ReporterError;
use crate::logic::aggregate_leaderboard;
use crate::markdown::render_markdown_table;
use crate::safety::{safe_component, safe_file_name};
use common::result::RunResult;
use std::fs;
use std::path::Path;

pub fn write_leaderboard_artifacts(
    base: &Path,
    results: &[RunResult],
) -> Result<(), ReporterError> {
    let charts_dir = base.join("charts");
    fs::create_dir_all(&charts_dir)?;
    for category in categories(results) {
        let rows = aggregate_leaderboard(results, &category);
        let table = render_markdown_table(&rows, &category);
        let title = format!("{category} Throughput Benchmark");
        let svg = generate_throughput_svg(&rows, &title);
        let chart_name = safe_file_name(&category, "_throughput.svg")?;
        let table_name = format!("leaderboard_{}.md", safe_component(&category)?);
        fs::write(charts_dir.join(chart_name), svg)?;
        fs::write(base.join(table_name), table)?;
    }
    Ok(())
}

fn categories(results: &[RunResult]) -> Vec<String> {
    let mut categories: Vec<String> = results
        .iter()
        .flat_map(|result| result.benchmarks.iter())
        .map(|benchmark| benchmark.category.clone())
        .collect();
    categories.sort_unstable();
    categories.dedup();
    categories
}
