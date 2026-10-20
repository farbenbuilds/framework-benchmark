use crate::cgroups::{CgroupFiles, open_cgroup_files, parse_cgroup_cpu_stat, sample_cgroup_files};
use crate::metrics::RuntimeCounters;
use crate::procfs::{parse_stat_times, parse_statm, parse_status_switches, resolve_page_size};
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub(super) struct Sources {
    cgroups: Option<CgroupFiles>,
    statm: Option<File>,
    page_size: u64,
}

#[derive(Debug, Default)]
pub(super) struct CounterPaths {
    cpu_stat: Option<PathBuf>,
    stat: Option<PathBuf>,
    status: Option<PathBuf>,
}

pub(super) fn counter_paths(target_pid: Option<u32>, cgroup_dir: Option<&Path>) -> CounterPaths {
    CounterPaths {
        cpu_stat: cgroup_dir.map(|path| path.join("cpu.stat")),
        stat: target_pid.map(|pid| PathBuf::from(format!("/proc/{pid}/stat"))),
        status: target_pid.map(|pid| PathBuf::from(format!("/proc/{pid}/status"))),
    }
}

pub(super) fn read_counters(paths: &CounterPaths) -> RuntimeCounters {
    let (user_usec, kernel_usec) = if let Some(content) = paths
        .cpu_stat
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
    {
        parse_cgroup_cpu_stat(&content)
    } else {
        paths
            .stat
            .as_ref()
            .and_then(|path| std::fs::read_to_string(path).ok())
            .and_then(|content| parse_stat_times(&content))
            // Linux USER_HZ is 100 on the supported x86_64 and aarch64 hosts.
            .map(|(user, kernel)| (user.saturating_mul(10_000), kernel.saturating_mul(10_000)))
            .unwrap_or_default()
    };
    let context_switches = paths
        .status
        .as_ref()
        .and_then(|path| std::fs::read_to_string(path).ok())
        .map(|content| parse_status_switches(&content))
        .map(|(voluntary, involuntary)| voluntary.saturating_add(involuntary))
        .unwrap_or_default();
    RuntimeCounters {
        user_usec,
        kernel_usec,
        context_switches,
    }
}

pub(super) fn open_sources(target_pid: Option<u32>, cgroup_dir: Option<&Path>) -> Sources {
    let pid = target_pid.map_or_else(|| "self".to_owned(), |pid| pid.to_string());
    Sources {
        cgroups: cgroup_dir.map(open_cgroup_files),
        statm: File::open(format!("/proc/{pid}/statm")).ok(),
        page_size: resolve_page_size(),
    }
}

pub(super) fn sample_memory(sources: &mut Sources) -> (u64, u64) {
    let process_rss = read_statm(&mut sources.statm, sources.page_size);
    let cgroup = sources.cgroups.as_mut().map(sample_cgroup_files);
    let current = cgroup.map_or(0, |sample| sample.current_bytes);
    let peak = cgroup.map_or(0, |sample| sample.peak_bytes);
    (process_rss.max(current), process_rss.max(peak))
}

fn read_statm(file: &mut Option<File>, page_size: u64) -> u64 {
    let Some(file) = file.as_mut() else {
        return 0;
    };
    let mut buffer = [0; 128];
    if file.seek(SeekFrom::Start(0)).is_err() {
        return 0;
    }
    let Ok(count) = file.read(&mut buffer) else {
        return 0;
    };
    let Ok(content) = std::str::from_utf8(&buffer[..count]) else {
        return 0;
    };
    parse_statm(content, page_size).unwrap_or(0)
}
