// Resource monitor for Linux cgroups v2 and /proc stats.
pub fn collector_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
