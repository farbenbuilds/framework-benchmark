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
use crate::LoadGenError;

impl BenchmarkPlan {
    pub fn validate(&self) -> Result<(), LoadGenError> {
        if self.target_addr.trim().is_empty() {
            return Err(LoadGenError::InvalidPlan("target address is empty".into()));
        }
        if !self.path.starts_with('/') {
            return Err(LoadGenError::InvalidPlan("path must start with '/'".into()));
        }
        if self.concurrency == 0 {
            return Err(LoadGenError::InvalidPlan(
                "concurrency must be positive".into(),
            ));
        }
        if self.duration_secs == 0 {
            return Err(LoadGenError::InvalidPlan(
                "duration must be positive".into(),
            ));
        }
        if self.pipeline_depth == 0 || self.pipeline_depth > 1024 {
            return Err(LoadGenError::InvalidPlan(
                "pipeline depth must be between 1 and 1024".into(),
            ));
        }
        if self.target_rps == Some(0) {
            return Err(LoadGenError::InvalidPlan(
                "target RPS must be positive".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BenchmarkPlan;

    #[test]
    fn validate_rejects_invalid_measurement_bounds() {
        let plan = BenchmarkPlan {
            duration_secs: 0,
            ..BenchmarkPlan::default()
        };
        assert!(plan.validate().is_err());
    }

    #[test]
    fn validate_accepts_default_plan() {
        assert!(BenchmarkPlan::default().validate().is_ok());
    }
}
