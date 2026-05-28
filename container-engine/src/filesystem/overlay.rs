use std::path::{Path, PathBuf};

use nix::mount::{mount, MsFlags};
use nix::unistd::mkdir;

use crate::util::errors::{ContainerError, ContainerResult};

/// Set up OverlayFS to provide a read-only root filesystem with a
/// writable upper layer stored on a tmpfs.
///
/// OverlayFS layout:
///   /tmp/container-overlay/{id}/
///     upper/    — writable diff layer (tmpfs)
///     work/     — overlay workdir (required by kernel)
///     merged/   — the final merged view
///
/// The original rootfs path becomes the lowerdir (read-only).
/// After setup, the config rootfs_path should point to merged/.
pub fn setup_overlay(rootfs: &Path) -> ContainerResult<PathBuf> {
    let overlay_base = PathBuf::from("/tmp/container-overlay");

    // Create overlay directory structure
    let upper = overlay_base.join("upper");
    let work = overlay_base.join("work");
    let merged = overlay_base.join("merged");

    for dir in [&upper, &work, &merged] {
        if !dir.exists() {
            mkdir(dir, nix::sys::stat::Mode::S_IRWXU).map_err(|e| {
                ContainerError::FilesystemError {
                    step: "overlay_mkdir",
                    detail: format!("mkdir({}) failed", dir.display()),
                    source: Some(e),
                }
            })?;
        }
    }

    // Build the overlay mount options
    let options = format!(
        "lowerdir={},upperdir={},workdir={},redirect_dir=off,metacopy=off",
        rootfs.display(),
        upper.display(),
        work.display(),
    );

    mount(
        Some("overlay"),
        &merged,
        Some("overlay"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_RELATIME,
        Some(options.as_str()),
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_overlay",
        detail: format!(
            "OverlayFS mount failed: lowerdir={}, upperdir={}",
            rootfs.display(),
            upper.display()
        ),
        source: Some(e),
    })?;

    tracing::info!(
        lowerdir = %rootfs.display(),
        upper = %upper.display(),
        merged = %merged.display(),
        "OverlayFS mounted"
    );

    Ok(merged)
}
