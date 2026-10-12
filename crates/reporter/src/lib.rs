// Metrics reporting, Markdown leaderboard generation, and JSON archiving.
pub fn reporter_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
