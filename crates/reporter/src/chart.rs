use crate::logic::LeaderboardRow;
use std::fmt::Write;

const CHART_WIDTH: usize = 800;
const LEFT_MARGIN: usize = 180;
const MAX_BAR_WIDTH: f64 = 500.0;
const BAR_HEIGHT: usize = 24;
const GAP: usize = 12;
const PANEL_MARGIN: usize = 42;

pub fn generate_throughput_svg(rows: &[LeaderboardRow], title: &str) -> String {
    let panel_height = PANEL_MARGIN + rows.len() * (BAR_HEIGHT + GAP) + GAP;
    let height = 42 + panel_height * 2;
    let mut svg = String::new();
    let _ = writeln!(
        svg,
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 {CHART_WIDTH} {height}" width="100%" height="{height}">"#
    );
    let _ = writeln!(
        svg,
        r##"<rect width="100%" height="100%" fill="#0f172a" rx="6"/>"##
    );
    let _ = writeln!(
        svg,
        r##"<text x="24" y="26" fill="#f8fafc" font-size="16" font-weight="600" font-family="sans-serif">{}</text>"##,
        xml_text(title)
    );
    render_bars(
        &mut svg,
        rows,
        42,
        "Throughput (requests/sec)",
        throughput,
        "req/s",
    );
    render_bars(
        &mut svg,
        rows,
        42 + panel_height,
        "P99 latency (ms)",
        p99_latency,
        "ms",
    );
    svg.push_str("</svg>\n");
    svg
}

fn render_bars(
    svg: &mut String,
    rows: &[LeaderboardRow],
    top: usize,
    label: &str,
    value: fn(&LeaderboardRow) -> f64,
    unit: &str,
) {
    let max_value = rows.iter().map(value).fold(1.0_f64, f64::max);
    let _ = writeln!(
        svg,
        r##"<text x="24" y="{}" fill="#cbd5e1" font-size="13" font-weight="600" font-family="sans-serif">{label}</text>"##,
        top + 16
    );
    for (index, row) in rows.iter().enumerate() {
        let y = top + PANEL_MARGIN + index * (BAR_HEIGHT + GAP);
        let bar_width = ((value(row) / max_value) * MAX_BAR_WIDTH).max(4.0);
        let color = if row.libc == "glibc" {
            "#38bdf8"
        } else {
            "#34d399"
        };
        let _ = writeln!(
            svg,
            r##"<text x="168" y="{}" fill="#94a3b8" font-size="12" font-family="sans-serif" text-anchor="end">{}-{}</text>"##,
            y + 16,
            xml_text(&row.framework),
            xml_text(&row.libc)
        );
        let _ = writeln!(
            svg,
            r#"<rect x="{LEFT_MARGIN}" y="{y}" width="{bar_width:.1}" height="{BAR_HEIGHT}" fill="{color}" rx="4"/>"#
        );
        let _ = writeln!(
            svg,
            r##"<text x="{}" y="{}" fill="#f8fafc" font-size="12" font-family="sans-serif">{:.3} {unit}</text>"##,
            LEFT_MARGIN as f64 + bar_width + 8.0,
            y + 16,
            value(row)
        );
    }
}

fn throughput(row: &LeaderboardRow) -> f64 {
    row.requests_per_sec
}

fn p99_latency(row: &LeaderboardRow) -> f64 {
    row.p99_latency_ms
}

fn xml_text(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

#[cfg(test)]
mod tests {
    use super::xml_text;

    #[test]
    fn xml_text_escapes_element_and_attribute_delimiters() {
        assert_eq!(xml_text("<&>\"'"), "&lt;&amp;&gt;&quot;&apos;");
    }
}
