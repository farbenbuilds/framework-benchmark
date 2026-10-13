use crate::LoadGenError;

pub const MAX_CONCURRENCY: usize = 256;
pub const MAX_PIPELINE_DEPTH: usize = 64;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionMode {
    #[default]
    KeepAlive,
    Churn,
}

#[derive(Debug, Clone)]
pub struct BenchmarkPlan {
    pub target_addr: String,
    pub path: String,
    pub concurrency: usize,
    pub duration_secs: u64,
    pub pipeline_depth: usize,
    pub connection_mode: ConnectionMode,
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
            connection_mode: ConnectionMode::KeepAlive,
            target_rps: None,
        }
    }
}

impl BenchmarkPlan {
    pub fn validate(&self) -> Result<(), LoadGenError> {
        if self.target_addr.trim().is_empty() {
            return Err(LoadGenError::InvalidPlan("target address is empty".into()));
        }
        if !self.path.starts_with('/') {
            return Err(LoadGenError::InvalidPlan("path must start with '/'".into()));
        }
        if !(1..=MAX_CONCURRENCY).contains(&self.concurrency) {
            return Err(LoadGenError::InvalidPlan(
                "concurrency must be between 1 and 256".into(),
            ));
        }
        if self.duration_secs == 0 {
            return Err(LoadGenError::InvalidPlan(
                "duration must be positive".into(),
            ));
        }
        if !(1..=MAX_PIPELINE_DEPTH).contains(&self.pipeline_depth) {
            return Err(LoadGenError::InvalidPlan(
                "pipeline depth must be between 1 and 64".into(),
            ));
        }
        if self.connection_mode == ConnectionMode::Churn && self.pipeline_depth != 1 {
            return Err(LoadGenError::InvalidPlan(
                "churn mode requires a pipeline depth of 1".into(),
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
    use super::{BenchmarkPlan, ConnectionMode, MAX_CONCURRENCY, MAX_PIPELINE_DEPTH};

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

    #[test]
    fn default_plan_keeps_connections_open() {
        assert_eq!(
            BenchmarkPlan::default().connection_mode,
            ConnectionMode::KeepAlive
        );
    }

    #[test]
    fn validate_rejects_excessive_worker_resources() {
        let concurrency = BenchmarkPlan {
            concurrency: MAX_CONCURRENCY + 1,
            ..BenchmarkPlan::default()
        };
        let pipeline = BenchmarkPlan {
            pipeline_depth: MAX_PIPELINE_DEPTH + 1,
            ..BenchmarkPlan::default()
        };
        assert!(concurrency.validate().is_err());
        assert!(pipeline.validate().is_err());
    }

    #[test]
    fn validate_rejects_pipelined_connection_churn() {
        let plan = BenchmarkPlan {
            connection_mode: ConnectionMode::Churn,
            pipeline_depth: 2,
            ..BenchmarkPlan::default()
        };
        assert!(plan.validate().is_err());
    }
}
