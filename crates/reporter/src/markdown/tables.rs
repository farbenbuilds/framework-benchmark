use crate::logic::{DualLibcDelta, LeaderboardRow};
use crate::safety::safe_file_name;
use std::fmt::Write;

pub fn render_markdown_table(rows: &[LeaderboardRow], category: &str) -> String {
    render_markdown_table_with_deltas(rows, &[], category)
}

pub fn render_markdown_table_with_deltas(
    rows: &[LeaderboardRow],
    deltas: &[DualLibcDelta],
    category: &str,
) -> String {
    let mut output = String::new();
    let category_cell = markdown_cell(category);
    let _ = writeln!(output, "### Benchmark Leaderboard: {category_cell}\n");
    if let Ok(file_name) = safe_file_name(category, "_throughput.svg") {
        let _ = writeln!(
            output,
            "![{category_cell} throughput and P99 latency](results/charts/{file_name})\n"
        );
    }
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
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

#[cfg(test)]
mod tests {
    use super::{markdown_cell, render_markdown_table};

    #[test]
    fn markdown_cell_escapes_delimiters_breaks_and_angle_brackets() {
        assert_eq!(markdown_cell("a|b\\c\nd<e>f"), "a\\|b\\\\c d&lt;e&gt;f");
    }

    #[test]
    fn markdown_category_cannot_traverse_chart_path() {
        let markdown = render_markdown_table(&[], "../../etc/passwd");

        assert!(!markdown.contains("passwd_throughput.svg"));
        assert!(!markdown.contains("results/charts/"));
    }

    #[test]
    fn markdown_category_still_renders_safe_chart_path() {
        let markdown = render_markdown_table(&[], "plaintext");

        assert!(markdown.contains("results/charts/plaintext_throughput.svg"));
    }
}
