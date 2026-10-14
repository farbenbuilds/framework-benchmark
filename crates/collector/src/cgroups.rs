use crate::error::CollectorError;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CgroupSample {
    pub current_bytes: u64,
    pub peak_bytes: u64,
    pub user_usec: u64,
    pub system_usec: u64,
}

#[derive(Debug)]
pub struct CgroupFiles {
    memory_current: Option<File>,
    memory_peak: Option<File>,
    cpu_stat: Option<File>,
}

pub fn parse_cgroup_memory_stat(content: &str) -> Result<u64, CollectorError> {
    content
        .trim()
        .parse::<u64>()
        .map_err(|error| CollectorError::Parse(format!("invalid cgroup memory stat: {error}")))
}

pub fn parse_cgroup_cpu_stat(content: &str) -> (u64, u64) {
    let mut user_usec = 0;
    let mut system_usec = 0;
    for line in content.lines() {
        if let Some(value) = line.strip_prefix("user_usec ") {
            user_usec = value.trim().parse().unwrap_or(0);
        } else if let Some(value) = line.strip_prefix("system_usec ") {
            system_usec = value.trim().parse().unwrap_or(0);
        }
    }
    (user_usec, system_usec)
}

pub fn open_cgroup_files(path: &Path) -> CgroupFiles {
    CgroupFiles {
        memory_current: File::open(path.join("memory.current")).ok(),
        memory_peak: File::open(path.join("memory.peak")).ok(),
        cpu_stat: File::open(path.join("cpu.stat")).ok(),
    }
}

pub fn sample_cgroup_files(files: &mut CgroupFiles) -> CgroupSample {
    let current_bytes = read_number(&mut files.memory_current);
    let peak_bytes = read_number(&mut files.memory_peak);
    let (user_usec, system_usec) = read_cpu_stat(&mut files.cpu_stat);
    CgroupSample {
        current_bytes,
        peak_bytes,
        user_usec,
        system_usec,
    }
}

fn read_number(file: &mut Option<File>) -> u64 {
    let Some(file) = file.as_mut() else {
        return 0;
    };
    let mut buffer = [0; 32];
    if file.seek(SeekFrom::Start(0)).is_err() {
        return 0;
    }
    let Ok(count) = file.read(&mut buffer) else {
        return 0;
    };
    parse_number_bytes(&buffer[..count])
}

fn read_cpu_stat(file: &mut Option<File>) -> (u64, u64) {
    let Some(file) = file.as_mut() else {
        return (0, 0);
    };
    let mut buffer = [0; 256];
    if file.seek(SeekFrom::Start(0)).is_err() {
        return (0, 0);
    }
    let Ok(count) = file.read(&mut buffer) else {
        return (0, 0);
    };
    let Ok(content) = std::str::from_utf8(&buffer[..count]) else {
        return (0, 0);
    };
    parse_cgroup_cpu_stat(content)
}

fn parse_number_bytes(content: &[u8]) -> u64 {
    let mut value = 0_u64;
    for byte in content {
        if !byte.is_ascii_digit() {
            break;
        }
        value = value
            .saturating_mul(10)
            .saturating_add(u64::from(byte - b'0'));
    }
    value
}

#[cfg(test)]
mod tests {
    use super::{CgroupSample, open_cgroup_files, sample_cgroup_files};
    use std::fs;

    #[test]
    fn cgroup_files_should_sample_all_controllers() {
        let path = std::env::temp_dir().join(format!("collector-cgroups-{}", std::process::id()));
        fs::create_dir_all(&path).expect("create cgroup fixture directory");
        fs::write(path.join("memory.current"), "4096\n").expect("write current memory fixture");
        fs::write(path.join("memory.peak"), "8192\n").expect("write peak memory fixture");
        fs::write(
            path.join("cpu.stat"),
            "usage_usec 10\nuser_usec 7\nsystem_usec 3\n",
        )
        .expect("write cpu fixture");

        let mut files = open_cgroup_files(&path);
        let sample = sample_cgroup_files(&mut files);
        fs::remove_dir_all(&path).expect("remove cgroup fixture directory");

        assert_eq!(
            sample,
            CgroupSample {
                current_bytes: 4096,
                peak_bytes: 8192,
                user_usec: 7,
                system_usec: 3,
            }
        );
    }
}
