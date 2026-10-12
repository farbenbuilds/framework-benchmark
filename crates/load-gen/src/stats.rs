use common::result::StatusCodeCounts;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkerStats {
    pub requests_completed: u64,
    pub bytes_transferred: u64,
    pub status_2xx: u64,
    pub status_4xx: u64,
    pub status_5xx: u64,
    pub errors: u64,
}

impl WorkerStats {
    #[inline]
    pub fn record_response(&mut self, status_code: u16, bytes: usize) {
        self.requests_completed += 1;
        self.bytes_transferred += bytes as u64;
        match status_code {
            200..=299 => self.status_2xx += 1,
            400..=499 => self.status_4xx += 1,
            500..=599 => self.status_5xx += 1,
            _ => self.errors += 1,
        }
    }

    #[inline]
    pub fn record_error(&mut self) {
        self.errors += 1;
    }

    pub fn merge(&mut self, other: &Self) {
        self.requests_completed += other.requests_completed;
        self.bytes_transferred += other.bytes_transferred;
        self.status_2xx += other.status_2xx;
        self.status_4xx += other.status_4xx;
        self.status_5xx += other.status_5xx;
        self.errors += other.errors;
    }

    pub fn to_status_code_counts(&self) -> StatusCodeCounts {
        StatusCodeCounts {
            http_2xx: self.status_2xx,
            http_4xx: self.status_4xx,
            http_5xx: self.status_5xx,
            errors: self.errors,
        }
    }
}
