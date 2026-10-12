use crate::logic::LeaderboardRow;
use std::fmt::Write;

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
