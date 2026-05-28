pub mod mask;
pub mod mounts;
pub mod overlay;
pub mod pivot_root;

use std::path::Path;

use crate::container::config::ContainerConfig;
use crate::util::errors::{ContainerError, ContainerResult};

/// Perform full filesystem isolation for a container:
/// 1. Make root mount private
/// 2. If readonly, set up OverlayFS
/// 3. Bind-mount rootfs
/// 4. pivot_root into the container root
/// 5. Unmount old root
pub fn isolate_root(config: &ContainerConfig) -> ContainerResult<()> {
    // Step 1: Make root mount propagation private
    nix::mount::mount(
        None::<&str>,
        "/",
        None::<&str>,
        nix::mount::MsFlags::MS_PRIVATE | nix::mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "make_root_private",
        detail: "failed to set MS_PRIVATE|MS_REC on /".into(),
        source: Some(e),
    })?;

    let rootfs = &config.rootfs_path;

    // Step 2: If readonly, use OverlayFS
    if config.readonly_rootfs {
        overlay::setup_overlay(rootfs)?;
    }

    // Step 3: Bind-mount the rootfs onto itself so it becomes a mount point
    nix::mount::mount(
        Some(rootfs),
        rootfs,
        None::<&str>,
        nix::mount::MsFlags::MS_BIND | nix::mount::MsFlags::MS_REC,
        None::<&str>,
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "bind_mount_rootfs",
        detail: format!("MS_BIND failed on {}", rootfs.display()),
        source: Some(e),
    })?;

    // Step 4: pivot_root into the rootfs
    pivot_root::execute(rootfs)?;

    // Step 5: Change directory to new root
    std::env::set_current_dir("/").map_err(|e| ContainerError::FilesystemError {
        step: "chdir_root",
        detail: "chdir(/) after pivot_root failed".into(),
        source: None,
    })?;

    Ok(())
}

/// Mount virtual filesystems inside the container's new root.
pub fn mount_virtual_fs(_config: &ContainerConfig) -> ContainerResult<()> {
    mounts::mount_all()?;
    mask::apply_masks()?;
    Ok(())
}
