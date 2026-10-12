use crate::error::CommonError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LibcVariant {
    Glibc,
    Musl,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MachineInfo {
    pub hostname: String,
    pub os_release: String,
    pub kernel_version: String,
    pub cpu_model: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub total_memory_mb: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencyDistribution {
    pub min_us: u64,
    pub p50_us: u64,
    pub p75_us: u64,
    pub p90_us: u64,
    pub p99_us: u64,
    pub p99_9_us: u64,
    pub p99_99_us: u64,
    pub max_us: u64,
    pub std_dev_us: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub peak_rss_bytes: u64,
    pub avg_rss_bytes: u64,
    pub cpu_user_pct: f32,
    pub cpu_kernel_pct: f32,
    pub context_switches: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rapl_joules: Option<f64>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusCodeCounts {
    pub http_2xx: u64,
    pub http_4xx: u64,
    pub http_5xx: u64,
    pub errors: u64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TestMetrics {
    pub category: String,
    pub concurrency: u32,
    pub duration_secs: u32,
    pub requests_total: u64,
    pub requests_per_sec: f64,
    pub transfer_mb_per_sec: f64,
    pub latency: LatencyDistribution,
    pub resources: ResourceUsage,
    pub status_codes: StatusCodeCounts,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RunResult {
    pub run_id: String,
    pub framework_name: String,
    pub framework_version: String,
    pub libc: LibcVariant,
    pub timestamp_epoch_secs: u64,
    pub machine: MachineInfo,
    pub benchmarks: Vec<TestMetrics>,
}

pub fn serialize_result_pretty(result: &RunResult) -> Result<String, CommonError> {
    serde_json::to_string_pretty(result).map_err(CommonError::JsonParse)
}

pub fn deserialize_result(json_str: &str) -> Result<RunResult, CommonError> {
    serde_json::from_str(json_str).map_err(CommonError::JsonParse)
}
