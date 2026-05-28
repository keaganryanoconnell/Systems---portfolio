use crate::util::errors::{ContainerError, ContainerResult};

/// Bring up the loopback interface inside the container's network namespace.
/// This is called AFTER pivot_root, so /proc is available inside the new root.
pub fn setup_loopback() -> ContainerResult<()> {
    // Write to /sys/class/net/lo/ifstate or use sysfs
    // Simplified: we try to bring lo up via writing to sysfs
    let lo_ifstate = std::path::Path::new("/sys/class/net/lo/device/operstate");
    if lo_ifstate.exists() {
        // Loopback exists, try to set it up by writing flags
        let operstate_path = std::path::Path::new("/sys/class/net/lo/operstate");
        if operstate_path.exists() {
            // Already up if operstate is "up"
            if let Ok(state) = std::fs::read_to_string(operstate_path) {
                if state.trim() == "up" {
                    return Ok(());
                }
            }
        }
    }

    // Fallback: use ip command (should be available via busybox)
    match std::process::Command::new("ip")
        .args(["link", "set", "lo", "up"])
        .output()
    {
        Ok(output) if output.status.success() => {
            tracing::debug!("Loopback interface brought up");
            Ok(())
        }
        Ok(output) => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            tracing::warn!(stderr = %stderr, "Failed to bring up loopback (non-fatal)");
            Ok(()) // Non-fatal
        }
        Err(e) => {
            tracing::warn!(error = %e, "Failed to bring up loopback (ip command not found, non-fatal)");
            Ok(()) // Non-fatal
        }
    }
}
