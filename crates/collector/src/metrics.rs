use common::result::ResourceUsage;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct RuntimeCounters {
    pub user_usec: u64,
    pub kernel_usec: u64,
    pub context_switches: u64,
}

pub fn apply_runtime_counters(
    usage: &mut ResourceUsage,
    start: RuntimeCounters,
    end: RuntimeCounters,
    elapsed_usec: u64,
) {
    if elapsed_usec == 0 {
        return;
    }
    let user = end.user_usec.saturating_sub(start.user_usec);
    let kernel = end.kernel_usec.saturating_sub(start.kernel_usec);
    usage.cpu_user_pct = percentage(user, elapsed_usec);
    usage.cpu_kernel_pct = percentage(kernel, elapsed_usec);
    usage.context_switches = end.context_switches.saturating_sub(start.context_switches);
}

fn percentage(delta_usec: u64, elapsed_usec: u64) -> f32 {
    ((delta_usec as f64 * 100.0) / elapsed_usec as f64) as f32
}
