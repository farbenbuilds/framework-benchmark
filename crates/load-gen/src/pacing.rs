use std::time::{Duration, Instant};

pub(crate) fn next_issue_at(
    interval: Option<Duration>,
    current: Instant,
    deadline: Instant,
) -> Instant {
    interval
        .and_then(|value| current.checked_add(value))
        .unwrap_or(current)
        .min(deadline)
}

#[cfg(test)]
mod tests {
    use super::next_issue_at;
    use std::time::{Duration, Instant};

    #[test]
    fn pacing_never_passes_deadline() {
        let now = Instant::now();
        let deadline = now + Duration::from_micros(10);
        assert_eq!(
            next_issue_at(Some(Duration::from_micros(20)), now, deadline),
            deadline
        );
    }

    #[test]
    fn pacing_without_limit_keeps_current_time() {
        let now = Instant::now();
        assert_eq!(next_issue_at(None, now, now + Duration::from_secs(1)), now);
    }
}
