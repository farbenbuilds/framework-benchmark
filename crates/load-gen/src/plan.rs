#[derive(Debug, Clone)]
pub struct BenchmarkPlan {
    pub target_addr: String,
    pub path: String,
    pub concurrency: usize,
    pub duration_secs: u64,
    pub pipeline_depth: usize,
    pub target_rps: Option<u64>,
}

impl Default for BenchmarkPlan {
    fn default() -> Self {
        Self {
            target_addr: "127.0.0.1:8080".into(),
            path: "/plaintext".into(),
            concurrency: 64,
            duration_secs: 10,
            pipeline_depth: 1,
            target_rps: None,
        }
    }
}
