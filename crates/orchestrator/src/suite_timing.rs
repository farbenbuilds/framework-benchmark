use crate::host_profile;

#[derive(Debug, Clone, Copy)]
pub struct SuiteTiming {
    pub warmup_secs: u64,
    pub cooldown_secs: u64,
}

impl SuiteTiming {
    pub const fn small() -> Self {
        Self {
            warmup_secs: 5,
            cooldown_secs: 3,
        }
    }

    pub fn for_environment() -> Self {
        if host_profile::small_host() {
            return Self::small();
        }
        Self::default()
    }
}

impl Default for SuiteTiming {
    fn default() -> Self {
        Self {
            warmup_secs: 15,
            cooldown_secs: 10,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SuiteTiming;

    #[test]
    fn small_profile_reduces_warmup_and_cooldown() {
        assert_eq!(SuiteTiming::small().warmup_secs, 5);
        assert_eq!(SuiteTiming::small().cooldown_secs, 3);
        assert!(SuiteTiming::small().warmup_secs < SuiteTiming::default().warmup_secs);
    }
}
