use std::path::Path;

use crate::container::config::DeviceRate;
use crate::util::errors::{ContainerError, ContainerResult};

/// Apply IO weight for a cgroup.
/// io.weight range: 1-10000, default 100.
pub fn apply_io_weight(cgroup_path: &Path, weight: u16) -> ContainerResult<()> {
    let clamped = weight.clamp(1, 10000);
    write_cgroup_file(cgroup_path, "io.weight", &clamped.to_string())
}

/// Apply IO bandwidth limits (bytes per second) per device.
/// Format: "major:minor rbps=<rate> wbps=<rate>"
pub fn apply_io_max_bps(cgroup_path: &Path, rate: &DeviceRate) -> ContainerResult<()> {
    let mut parts = vec![format!("{}:{}", rate.major, rate.minor)];
    if let Some(rbps) = rate.read_bps {
        parts.push(format!("rbps={rbps}"));
    }
    if let Some(wbps) = rate.write_bps {
        parts.push(format!("wbps={wbps}"));
    }
    if parts.len() > 1 {
        let value = parts.join(" ");
        write_cgroup_file(cgroup_path, "io.max", &value)?;
    }
    Ok(())
}

/// Apply IOPS limits per device.
pub fn apply_io_max_iops(cgroup_path: &Path, rate: &DeviceRate) -> ContainerResult<()> {
    let mut parts = vec![format!("{}:{}", rate.major, rate.minor)];
    if let Some(riops) = rate.read_iops {
        parts.push(format!("riops={riops}"));
    }
    if let Some(wiops) = rate.write_iops {
        parts.push(format!("wiops={wiops}"));
    }
    if parts.len() > 1 {
        let value = parts.join(" ");
        write_cgroup_file(cgroup_path, "io.max", &value)?;
    }
    Ok(())
}

fn write_cgroup_file(cgroup_path: &Path, filename: &str, value: &str) -> ContainerResult<()> {
    let file_path = cgroup_path.join(filename);
    std::fs::write(&file_path, value).map_err(|e| ContainerError::CgroupError {
        controller: "io",
        detail: format!(
            "write to {}/{} failed: {value}",
            cgroup_path.display(),
            filename
        ),
        source: Some(e),
    })
}
