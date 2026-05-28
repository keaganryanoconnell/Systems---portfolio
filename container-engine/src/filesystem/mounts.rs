use nix::mount::{mount, MsFlags};
use nix::unistd::mkdir;

use crate::util::errors::{ContainerError, ContainerResult};

/// Mount all required virtual filesystems inside the container.
/// Must be called AFTER pivot_root so the target paths exist in the new root.
pub fn mount_all() -> ContainerResult<()> {
    // /proc — process information
    ensure_dir("/proc")?;
    mount_proc()?;

    // /sys — kernel information (will be remounted ro+masked)
    ensure_dir("/sys")?;
    mount_sysfs()?;

    // /dev — device nodes
    ensure_dir("/dev")?;
    mount_devtmpfs()?;

    // /dev/pts — pseudo-terminals
    ensure_dir("/dev/pts")?;
    mount_devpts()?;

    // /dev/mqueue — POSIX message queues
    ensure_dir("/dev/mqueue")?;
    mount_mqueue()?;

    // /run — volatile runtime data
    ensure_dir("/run")?;
    mount_tmpfs("/run", "tmpfs-run", 0o755)?;

    // /tmp — scratch space
    ensure_dir("/tmp")?;
    mount_tmpfs("/tmp", "tmpfs-tmp", 0o1777)?;

    tracing::info!("All virtual filesystems mounted");
    Ok(())
}

fn ensure_dir(path: &str) -> ContainerResult<()> {
    let p = std::path::Path::new(path);
    if !p.exists() {
        mkdir(
            p,
            nix::sys::stat::Mode::S_IRWXU
                | nix::sys::stat::Mode::S_IRGRP
                | nix::sys::stat::Mode::S_IXGRP
                | nix::sys::stat::Mode::S_IROTH
                | nix::sys::stat::Mode::S_IXOTH,
        )
        .map_err(|e| ContainerError::FilesystemError {
            step: "ensure_dir",
            detail: format!("mkdir({path}) failed"),
            source: Some(e),
        })?;
    }
    Ok(())
}

fn mount_proc() -> ContainerResult<()> {
    mount(
        Some("proc"),
        "/proc",
        Some("proc"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME,
        None::<&str>,
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_proc",
        detail: "mount(/proc) failed".into(),
        source: Some(e),
    })
}

fn mount_sysfs() -> ContainerResult<()> {
    mount(
        Some("sysfs"),
        "/sys",
        Some("sysfs"),
        MsFlags::MS_NOSUID
            | MsFlags::MS_NODEV
            | MsFlags::MS_NOEXEC
            | MsFlags::MS_RDONLY
            | MsFlags::MS_RELATIME,
        None::<&str>,
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_sysfs",
        detail: "mount(/sys) failed".into(),
        source: Some(e),
    })
}

fn mount_devtmpfs() -> ContainerResult<()> {
    mount(
        Some("devtmpfs"),
        "/dev",
        Some("devtmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_STRICTATIME,
        Some("mode=755"),
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_devtmpfs",
        detail: "mount(/dev) failed".into(),
        source: Some(e),
    })
}

fn mount_devpts() -> ContainerResult<()> {
    mount(
        Some("devpts"),
        "/dev/pts",
        Some("devpts"),
        MsFlags::MS_NOSUID | MsFlags::MS_NOEXEC,
        Some("newinstance,ptmxmode=0666,mode=620"),
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_devpts",
        detail: "mount(/dev/pts) failed".into(),
        source: Some(e),
    })
}

fn mount_mqueue() -> ContainerResult<()> {
    mount(
        Some("mqueue"),
        "/dev/mqueue",
        Some("mqueue"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC,
        None::<&str>,
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_mqueue",
        detail: "mount(/dev/mqueue) failed".into(),
        source: Some(e),
    })
}

fn mount_tmpfs(path: &str, name: &str, mode: u32) -> ContainerResult<()> {
    let data = format!("mode={mode:o},size=64M");
    mount(
        Some(name),
        path,
        Some("tmpfs"),
        MsFlags::MS_NOSUID | MsFlags::MS_NODEV | MsFlags::MS_NOEXEC | MsFlags::MS_RELATIME,
        Some(data.as_str()),
    )
    .map_err(|e| ContainerError::FilesystemError {
        step: "mount_tmpfs",
        detail: format!("mount(tmpfs, {path}) failed"),
        source: Some(e),
    })
}
