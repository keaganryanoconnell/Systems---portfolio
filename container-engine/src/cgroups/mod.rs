pub mod cpu;
pub mod device;
pub mod io;
pub mod memory;
pub mod pids;

use std::path::PathBuf;

use nix::unistd::Pid;

use crate::container::config::ContainerConfig;
use crate::util::errors::{ContainerError, ContainerResult};

const CGROUP_ROOT: &str = "/sys/fs/cgroup";

/// RAII guard for a cgroup directory. On Drop, removes the cgroup.
pub struct CgroupGuard {
    path: PathBuf,
    cleaned_up: bool,
}

impl CgroupGuard {
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    pub fn mark_cleaned(&mut self) {
        self.cleaned_up = true;
    }
}

impl Drop for CgroupGuard {
    fn drop(&mut self) {
        if !self.cleaned_up {
            // Best-effort cleanup: kill remaining processes and remove
            let _ = std::fs::write(self.path.join("cgroup.kill"), "1");
            let _ = std::fs::remove_dir(&self.path);
            tracing::warn!(path = %self.path.display(), "Cgroup guard cleanup on drop");
        }
    }
}

/// Sets up all cgroup controllers for a container.
/// Returns a CgroupGuard that cleans up on Drop.
pub fn setup_cgroups(config: &ContainerConfig, child_pid: Pid) -> ContainerResult<CgroupGuard> {
    let cgroup_name = format!("container-{}", config.id.short_id());
    let cgroup_path = PathBuf::from(CGROUP_ROOT).join(&cgroup_name);

    // Create the cgroup directory
    std::fs::create_dir_all(&cgroup_path).map_err(|e| ContainerError::CgroupError {
        controller: "cgroup_root",
        detail: format!("mkdir {} failed", cgroup_path.display()),
        source: Some(e),
    })?;

    let mut guard = CgroupGuard {
        path: cgroup_path.clone(),
        cleaned_up: false,
    };

    // Enable required controllers at the cgroup level by writing to cgroup.subtree_control
    // (For leaf cgroups, this is not needed; we write directly to controller files.)

    // Apply each controller
    if let Some(limit) = config.memory_limit_bytes {
        memory::apply_memory_limit(&cgroup_path, limit, config.memory_swap_bytes)?;
    }

    if let Some(weight) = config.cpu_weight {
        cpu::apply_cpu_weight(&cgroup_path, weight)?;
    }

    if let Some((quota, period)) = config.cpu_max {
        cpu::apply_cpu_max(&cgroup_path, quota, period)?;
    }

    if let Some(weight) = config.io_weight {
        io::apply_io_weight(&cgroup_path, weight)?;
    }

    if let Some(rate) = config.io_max_bps {
        io::apply_io_max_bps(&cgroup_path, &rate)?;
    }

    if let Some(rate) = config.io_max_iops {
        io::apply_io_max_iops(&cgroup_path, &rate)?;
    }

    if let Some(max) = config.pids_max {
        pids::apply_pids_max(&cgroup_path, max)?;
    }

    device::apply_device_whitelist(&cgroup_path)?;

    // Attach the child PID to the cgroup
    let procs_path = cgroup_path.join("cgroup.procs");
    std::fs::write(&procs_path, child_pid.as_raw().to_string()).map_err(|e| {
        ContainerError::CgroupError {
            controller: "cgroup_procs",
            detail: format!("failed to attach PID {} to cgroup", child_pid),
            source: Some(e),
        }
    })?;

    tracing::info!(
        cgroup = %cgroup_name,
        pid = %child_pid,
        memory = ?config.memory_limit_bytes,
        cpu_weight = ?config.cpu_weight,
        pids_max = ?config.pids_max,
        "Cgroups configured"
    );

    Ok(guard)
}

/// Clean up cgroup directory (removes all processes and the directory).
pub fn cleanup_cgroup(path: &PathBuf) -> ContainerResult<()> {
    // Kill any remaining processes
    let _ = std::fs::write(path.join("cgroup.kill"), "1");
    // Remove the cgroup directory
    std::fs::remove_dir(path).map_err(|e| ContainerError::CgroupError {
        controller: "cleanup",
        detail: format!("remove_dir({}) failed", path.display()),
        source: Some(e),
    })?;
    Ok(())
}
