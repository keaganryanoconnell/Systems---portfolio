use std::path::Path;

use crate::util::errors::{ContainerError, ContainerResult};

/// Apply memory limits to a cgroup.
///
/// Files written:
///   memory.max         — Hard limit in bytes (OOM-kill if exceeded)
///   memory.high        — Soft limit in bytes (throttle reclaim above this)
///   memory.swap.max    — Swap limit (0 disables swap)
///   memory.oom.group   — Kill all processes in the cgroup on OOM
pub fn apply_memory_limit(
    cgroup_path: &Path,
    limit_bytes: u64,
    swap_bytes: Option<u64>,
) -> ContainerResult<()> {
    // memory.max — hard limit
    write_cgroup_file(cgroup_path, "memory.max", &limit_bytes.to_string())?;

    // memory.high — soft limit at 80% of hard limit
    let high = (limit_bytes as f64 * 0.8) as u64;
    write_cgroup_file(cgroup_path, "memory.high", &high.to_string())?;

    // memory.swap.max — disable swap for deterministic memory behavior
    let swap_value = swap_bytes.unwrap_or(0);
    write_cgroup_file(cgroup_path, "memory.swap.max", &swap_value.to_string())?;

    // memory.oom.group — kill entire cgroup on OOM, not just the faulting thread
    write_cgroup_file(cgroup_path, "memory.oom.group", "1")?;

    // memory.zswap.max — disable zswap for latency predictability
    let _ = write_cgroup_file(cgroup_path, "memory.zswap.max", "0");

    Ok(())
}

fn write_cgroup_file(cgroup_path: &Path, filename: &str, value: &str) -> ContainerResult<()> {
    let file_path = cgroup_path.join(filename);
    std::fs::write(&file_path, value).map_err(|e| ContainerError::CgroupError {
        controller: "memory",
        detail: format!(
            "write to {}/{} failed: {value}",
            cgroup_path.display(),
            filename
        ),
        source: Some(e),
    })
}
