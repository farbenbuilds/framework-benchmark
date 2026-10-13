use crate::error::ReporterError;
use crate::logic::{DualLibcDelta, LeaderboardRow, aggregate_leaderboard, dual_libc_deltas};
use crate::storage::replace_file;
use common::result::RunResult;
use std::fmt::Write;
use std::fs;
use std::path::Path;

const START_MARKER: &str = "<!-- BENCHMARK_RESULTS_START -->";
const END_MARKER: &str = "<!-- BENCHMARK_RESULTS_END -->";
pub fn render_markdown_table(rows: &[LeaderboardRow], category: &str) -> String {
    render_markdown_table_with_deltas(rows, &[], category)
}

pub fn render_markdown_table_with_deltas(
    rows: &[LeaderboardRow],
    deltas: &[DualLibcDelta],
    category: &str,
) -> String {
    let mut output = String::new();
    let category = markdown_cell(category);
    let _ = writeln!(output, "### Benchmark Leaderboard: {category}\n");
    let _ = writeln!(
        output,
        "![{category} throughput and P99 latency](results/charts/{category}_throughput.svg)\n"
    );
    render_throughput_table(&mut output, rows);
    output.push('\n');
    render_latency_table(&mut output, rows);
    output.push('\n');
    render_resources_table(&mut output, rows);
    if !deltas.is_empty() {
        output.push('\n');
        render_delta_table(&mut output, deltas);
    }
    output
}

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

fn render_throughput_table(output: &mut String, rows: &[LeaderboardRow]) {
    output.push_str("#### Throughput\n\n");
    output.push_str("| Rank | Framework | Version | Libc | Concurrency | Duration (s) | Requests | Requests/sec | Transfer (MB/s) |\n");
    output.push_str("| ---: | :-------- | :------ | :--- | ----------: | -----------: | -------: | -----------: | --------------: |\n");
    for row in rows {
        let _ = writeln!(
            output,
            "| {} | {} | {} | {} | {} | {} | {} | {:.2} | {:.2} |",
            row.rank,
            markdown_cell(&row.framework),
            markdown_cell(&row.framework_version),
            markdown_cell(&row.libc),
            row.concurrency,
            row.duration_secs,
            row.requests_total,
            row.requests_per_sec,
            row.transfer_mb_per_sec
        );
    }
}

fn render_latency_table(output: &mut String, rows: &[LeaderboardRow]) {
    output.push_str("#### Latency distribution (ms)\n\n");
    output.push_str("| Rank | Framework | Libc | Min | P50 | P75 | P90 | P99 | P99.9 | P99.99 | Max | Std. dev. |\n");
    output.push_str("| ---: | :-------- | :--- | --: | --: | --: | --: | --: | ----: | ------: | --: | --------: |\n");
    for row in rows {
        let _ = writeln!(
            output,
            "| {} | {} | {} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} | {:.3} |",
            row.rank,
            markdown_cell(&row.framework),
            markdown_cell(&row.libc),
            row.min_latency_ms,
            row.p50_latency_ms,
            row.p75_latency_ms,
            row.p90_latency_ms,
            row.p99_latency_ms,
            row.p99_9_latency_ms,
            row.p99_99_latency_ms,
            row.max_latency_ms,
            row.latency_std_dev_ms
        );
    }
}

fn render_resources_table(output: &mut String, rows: &[LeaderboardRow]) {
    output.push_str("#### Resources and reliability\n\n");
    output.push_str("| Rank | Framework | Libc | Avg RSS (MB) | Peak RSS (MB) | User CPU (%) | Kernel CPU (%) | Context switches | Energy (J) | 2xx | 4xx | 5xx | Errors |\n");
    output.push_str("| ---: | :-------- | :--- | -----------: | ------------: | -----------: | -------------: | ---------------: | ---------: | --: | --: | --: | -----: |\n");
    for row in rows {
        let energy = row
            .rapl_joules
            .map_or_else(|| "n/a".into(), |value| format!("{value:.3}"));
        let _ = writeln!(
            output,
            "| {} | {} | {} | {:.2} | {:.2} | {:.2} | {:.2} | {} | {} | {} | {} | {} | {} |",
            row.rank,
            markdown_cell(&row.framework),
            markdown_cell(&row.libc),
            row.avg_rss_mb,
            row.peak_rss_mb,
            row.cpu_user_pct,
            row.cpu_kernel_pct,
            row.context_switches,
            energy,
            row.http_2xx,
            row.http_4xx,
            row.http_5xx,
            row.errors
        );
    }
}

fn render_delta_table(output: &mut String, deltas: &[DualLibcDelta]) {
    output.push_str("#### musl vs glibc deltas\n\n");
    output.push_str("| Framework | Version | Concurrency | RPS delta | P99 delta |\n");
    output.push_str("| :-------- | :------ | ----------: | --------: | --------: |\n");
    for delta in deltas {
        let _ = writeln!(
            output,
            "| {} | {} | {} | {} | {} |",
            markdown_cell(&delta.framework),
            markdown_cell(&delta.framework_version),
            delta.concurrency,
            percentage_cell(delta.requests_per_sec_change_pct),
            percentage_cell(delta.p99_latency_change_pct),
        );
    }
}

fn percentage_cell(value: Option<f64>) -> String {
    value.map_or_else(|| "n/a".into(), |value| format!("{value:+.2}%"))
}

fn markdown_cell(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('|', "\\|")
        .replace(['\r', '\n'], " ")
}

#[cfg(test)]
mod tests {
    use super::markdown_cell;

    #[test]
    fn markdown_cell_escapes_table_delimiters_and_line_breaks() {
        assert_eq!(markdown_cell("a|b\\c\nd"), "a\\|b\\\\c d");
    }
}
