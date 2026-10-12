use crate::error::ReporterError;
use crate::logic::{LeaderboardRow, aggregate_leaderboard};
use common::result::RunResult;
use std::fmt::Write;
use std::fs;
use std::path::Path;

const START_MARKER: &str = "<!-- BENCHMARK_RESULTS_START -->";
const END_MARKER: &str = "<!-- BENCHMARK_RESULTS_END -->";

pub fn render_markdown_table(rows: &[LeaderboardRow], category: &str) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "### Benchmark Leaderboard: {category}\n");
    out.push_str("| Rank | Framework | Libc | Requests/sec | P99 Latency (ms) | Peak RSS (MB) |\n");
    out.push_str("| :--- | :-------- | :--- | -----------: | ---------------: | ------------: |\n");

    for row in rows {
        let _ = writeln!(
            out,
            "| {} | {} | {} | {:.2} | {:.3} | {:.2} |",
            row.rank,
            row.framework,
            row.libc,
            row.requests_per_sec,
            row.p99_latency_ms,
            row.peak_rss_mb
        );
    }
    out
}

pub fn update_readme_leaderboard(
    readme_path: &Path,
    results: &[RunResult],
) -> Result<(), ReporterError> {
    if !readme_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(readme_path)?;
    let Some(raw_start) = content.find(START_MARKER) else {
        return Ok(());
    };
    let start_pos = raw_start + START_MARKER.len();

    let Some(end_pos) = content.find(END_MARKER) else {
        return Ok(());
    };

    let categories = [
        "plaintext",
        "json",
        "db_single",
        "db_multi",
        "fortunes",
        "updates",
        "websocket",
    ];
    let mut section = String::from("\n\n");

    for cat in categories {
        let rows = aggregate_leaderboard(results, cat);
        if rows.is_empty() {
            continue;
        }
        section.push_str(&render_markdown_table(&rows, cat));
        section.push('\n');
    }

    if section.trim().is_empty() {
        section = "\n*No benchmark results recorded yet.*\n".to_string();
    }

    let mut new_content = String::with_capacity(content.len() + section.len());
    new_content.push_str(&content[..start_pos]);
    new_content.push_str(&section);
    new_content.push_str(&content[end_pos..]);

    fs::write(readme_path, new_content)?;
    Ok(())
}
