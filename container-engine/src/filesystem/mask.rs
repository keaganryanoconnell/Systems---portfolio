use std::path::Path;

use nix::mount::{mount, MsFlags};

use crate::util::errors::{ContainerError, ContainerResult};

/// Mask sensitive kernel interface paths inside the container by
/// mounting empty tmpfs over them, or bind-mounting them from
/// /dev/null for files that must exist.
const MASKED_PATHS: &[&str] = &[
    "/sys/firmware",
    "/sys/kernel/security",
    "/sys/kernel/debug",
    "/sys/kernel/tracing",
    "/proc/acpi",
    "/proc/kcore",
    "/proc/keys",
    "/proc/latency_stats",
    "/proc/timer_list",
    "/proc/sched_debug",
    "/proc/scsi",
];

const READONLY_PATHS: &[&str] = &["/proc/sys", "/proc/sysrq-trigger", "/proc/irq", "/proc/bus"];

pub fn apply_masks() -> ContainerResult<()> {
    for path in MASKED_PATHS {
        let p = Path::new(path);
        if p.exists() {
            // Mount a minimal tmpfs to mask the directory
            mount(
                Some("tmpfs"),
                p,
                Some("tmpfs"),
                MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RDONLY,
                Some("mode=0,size=0"),
            )
            .map_err(|e| ContainerError::FilesystemError {
                step: "mask_path",
                detail: format!("mask tmpfs on {path} failed"),
                source: Some(e),
            })?;
        }
    }

    for path in READONLY_PATHS {
        let p = Path::new(path);
        if p.exists() {
            // Bind-mount the path onto itself as readonly
            mount(
                Some(p),
                p,
                None::<&str>,
                MsFlags::MS_BIND | MsFlags::MS_REC,
                None::<&str>,
            )
            .map_err(|e| ContainerError::FilesystemError {
                step: "readonly_bind",
                detail: format!("bind mount on {path} failed"),
                source: Some(e),
            })?;

            // Remount as readonly
            mount(
                Some(p),
                p,
                None::<&str>,
                MsFlags::MS_BIND
                    | MsFlags::MS_REMOUNT
                    | MsFlags::MS_RDONLY
                    | MsFlags::MS_NOSUID
                    | MsFlags::MS_NODEV
                    | MsFlags::MS_NOEXEC,
                None::<&str>,
            )
            .map_err(|e| ContainerError::FilesystemError {
                step: "readonly_remount",
                detail: format!("remount ro {path} failed"),
                source: Some(e),
            })?;
        }
    }

    tracing::info!(
        "Applied {} masked + {} readonly paths",
        MASKED_PATHS.len(),
        READONLY_PATHS.len()
    );
    Ok(())
}
