// Asynchronous load generator core module.
pub fn load_gen_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
