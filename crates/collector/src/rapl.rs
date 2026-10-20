use crate::error::CollectorError;
use std::fs;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RaplZoneSnapshot {
    pub name: String,
    pub energy_uj: u64,
    pub max_energy_range_uj: u64,
}

pub fn read_rapl_snapshot(root: &Path) -> Result<Vec<RaplZoneSnapshot>, CollectorError> {
    let entries = match fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    let mut zones = Vec::new();
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        read_rapl_directory(&path, &mut zones)?;
    }
    zones.sort_unstable_by(|left, right| left.name.cmp(&right.name));
    Ok(zones)
}

fn read_rapl_directory(
    path: &Path,
    zones: &mut Vec<RaplZoneSnapshot>,
) -> Result<(), CollectorError> {
    if path.join("energy_uj").is_file() {
        zones.push(read_rapl_zone(path)?);
        return Ok(());
    }
    for entry in fs::read_dir(path)? {
        let path = entry?.path();
        if path.is_dir() {
            read_rapl_directory(&path, zones)?;
        }
    }
    Ok(())
}

pub fn energy_delta_uj(before: u64, after: u64, max_range: u64) -> u64 {
    if after >= before {
        return after - before;
    }
    max_range.saturating_sub(before).saturating_add(after)
}

pub fn energy_delta_joules(before: &[RaplZoneSnapshot], after: &[RaplZoneSnapshot]) -> f64 {
    after
        .iter()
        .filter_map(|end| {
            before
                .iter()
                .find(|start| start.name == end.name)
                .map(|start| {
                    energy_delta_uj(start.energy_uj, end.energy_uj, start.max_energy_range_uj)
                })
        })
        .fold(0_u64, u64::saturating_add) as f64
        / 1_000_000.0
}

fn read_rapl_zone(path: &Path) -> Result<RaplZoneSnapshot, CollectorError> {
    let label = fs::read_to_string(path.join("name"))?.trim().to_string();
    let id = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("unknown");
    let name = format!("{id}:{label}");
    let energy_uj = read_u64(path, "energy_uj")?;
    let max_energy_range_uj = read_u64(path, "max_energy_range_uj")?;
    Ok(RaplZoneSnapshot {
        name,
        energy_uj,
        max_energy_range_uj,
    })
}

fn read_u64(path: &Path, name: &str) -> Result<u64, CollectorError> {
    fs::read_to_string(path.join(name))?
        .trim()
        .parse()
        .map_err(|error| CollectorError::Parse(format!("invalid RAPL {name}: {error}")))
}

#[cfg(test)]
mod tests {
    use super::read_rapl_snapshot;
    use std::fs;
    use std::path::Path;

    fn write_zone(path: &Path, label: &str, energy: u64) {
        fs::create_dir_all(path).expect("create RAPL fixture");
        fs::write(path.join("name"), label).expect("write RAPL name");
        fs::write(path.join("energy_uj"), energy.to_string()).expect("write RAPL energy");
        fs::write(path.join("max_energy_range_uj"), "1000").expect("write RAPL range");
    }

    #[test]
    fn nested_zones_do_not_double_count_package_energy() {
        let root = std::env::temp_dir().join(format!("rapl-{}", std::process::id()));
        let package = root.join("intel-rapl:0");
        write_zone(&package, "package-0", 100);
        write_zone(&package.join("intel-rapl:0:0"), "core", 50);

        let zones = read_rapl_snapshot(&root).expect("read RAPL fixture");
        fs::remove_dir_all(&root).expect("remove RAPL fixture");

        assert_eq!(zones.len(), 1);
        assert_eq!(zones[0].energy_uj, 100);
    }
}
