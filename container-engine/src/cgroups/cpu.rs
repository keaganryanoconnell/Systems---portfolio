use std::path::Path;

use crate::util::errors::{ContainerError, ContainerResult};

/// Apply CPU weight (CFS shares) to a cgroup.
/// cpu.weight range: 1-10000, default 100.
/// Higher weight = more CPU time relative to other cgroups.
pub fn apply_cpu_weight(cgroup_path: &Path, weight: u16) -> ContainerResult<()> {
    let clamped = weight.clamp(1, 10000);
    write_cgroup_file(cgroup_path, "cpu.weight", &clamped.to_string())
}

/// Apply CPU max (CFS quota/period) to limit CPU usage.
/// quota: maximum microseconds of CPU time per period.
/// period: scheduling period in microseconds (default 100000 = 100ms).
/// Example: quota=150000, period=100000 => 1.5 CPU cores max.
pub fn apply_cpu_max(cgroup_path: &Path, quota_us: u64, period_us: u64) -> ContainerResult<()> {
    let value = format!("{quota_us} {period_us}");
    write_cgroup_file(cgroup_path, "cpu.max", &value)
}

fn write_cgroup_file(cgroup_path: &Path, filename: &str, value: &str) -> ContainerResult<()> {
    let file_path = cgroup_path.join(filename);
    std::fs::write(&file_path, value).map_err(|e| ContainerError::CgroupError {
        controller: "cpu",
        detail: format!(
            "write to {}/{} failed: {value}",
            cgroup_path.display(),
            filename
        ),
        source: Some(e),
    })
}
