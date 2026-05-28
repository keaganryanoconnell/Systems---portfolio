use std::path::Path;

use nix::mount::MntFlags;
use nix::unistd::pivot_root as nix_pivot_root;

use crate::util::errors::{ContainerError, ContainerResult};

const OLD_ROOT_DIR: &str = ".old_root";

/// Execute pivot_root to switch the process's root filesystem to `new_root`.
/// The previous root is moved to `new_root/.old_root` and then unmounted.
pub fn execute(new_root: &Path) -> ContainerResult<()> {
    // Create the old_root directory inside new_root
    let old_root_path = new_root.join(OLD_ROOT_DIR);

    std::fs::create_dir_all(&old_root_path).map_err(|e| ContainerError::FilesystemError {
        step: "create_old_root",
        detail: format!("mkdir {} failed", old_root_path.display()),
        source: None,
    })?;

    // Perform the pivot_root syscall
    nix_pivot_root(new_root, &old_root_path).map_err(|e| ContainerError::FilesystemError {
        step: "pivot_root",
        detail: format!(
            "pivot_root({}, {}) failed",
            new_root.display(),
            old_root_path.display()
        ),
        source: Some(e),
    })?;

    // At this point our root is new_root, and old_root is at /.old_root
    // Unmount the old root
    let old_root_after = Path::new("/").join(OLD_ROOT_DIR);

    // First try MNT_DETACH (lazy unmount), then try to remove the directory
    nix::mount::umount2(&old_root_after, MntFlags::MNT_DETACH).map_err(|e| {
        ContainerError::FilesystemError {
            step: "umount_old_root",
            detail: format!("umount2({}) failed", old_root_after.display()),
            source: Some(e),
        }
    })?;

    // Clean up the empty directory
    let _ = std::fs::remove_dir(&old_root_after);

    tracing::info!(new_root = %new_root.display(), "pivot_root completed");

    Ok(())
}
