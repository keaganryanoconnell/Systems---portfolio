use std::path::Path;

use crate::util::errors::{ContainerError, ContainerResult};

/// Apply PID limit to a cgroup to prevent fork bombs.
/// pids.max controls the maximum number of processes and threads.
pub fn apply_pids_max(cgroup_path: &Path, max: u32) -> ContainerResult<()> {
    let file_path = cgroup_path.join("pids.max");
    std::fs::write(&file_path, max.to_string()).map_err(|e| ContainerError::CgroupError {
        controller: "pids",
        detail: format!("write to pids.max failed: {max}"),
        source: Some(e),
    })
}
