use crate::logic::LeaderboardRow;
use std::fmt::Write;

pub fn generate_throughput_svg(rows: &[LeaderboardRow], title: &str) -> String {
    let bar_height = 28;
    let gap = 16;
    let top_margin = 50;
    let left_margin = 160;
    let chart_width = 800;
    let max_bar_width = 540.0;

    let height = top_margin + (rows.len() * (bar_height + gap)) + 30;
    let max_rps = rows
        .iter()
        .map(|r| r.requests_per_sec)
        .fold(1.0_f64, f64::max);

    let mut svg = String::new();
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {chart_width} {height}" width="100%" height="{height}">"#
    );
    let _ = writeln!(
        svg,
        r##"  <rect width="100%" height="100%" fill="#0f172a" rx="6"/>"##
    );
    let _ = writeln!(
        svg,
        r##"  <text x="24" y="32" fill="#f8fafc" font-size="16" font-weight="600" font-family="sans-serif">{title}</text>"##
    );

    for (i, row) in rows.iter().enumerate() {
        let y = top_margin + i * (bar_height + gap);
        let bar_w = ((row.requests_per_sec / max_rps) * max_bar_width).max(4.0);
        let label = format!("{}-{}", row.framework, row.libc);
        let val_text = format!("{:.0} req/s", row.requests_per_sec);

        let color = if row.libc == "glibc" {
            "#38bdf8"
        } else {
            "#34d399"
        };

        let _ = writeln!(
            svg,
            r##"  <text x="148" y="{}" fill="#94a3b8" font-size="13" font-family="sans-serif" text-anchor="end">{}</text>"##,
            y + 19,
            label
        );
        let _ = writeln!(
            svg,
            r#"  <rect x="{left_margin}" y="{y}" width="{bar_w:.1}" height="{bar_height}" fill="{color}" rx="4"/>"#
        );
        let _ = writeln!(
            svg,
            r##"  <text x="{}" y="{}" fill="#f8fafc" font-size="12" font-family="sans-serif">{}</text>"##,
            f64::from(left_margin) + bar_w + 10.0,
            y + 19,
            val_text
        );
    }

    svg.push_str("</svg>\n");
    svg
}
