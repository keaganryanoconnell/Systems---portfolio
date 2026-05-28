use std::process::Command;

use crate::util::errors::{ContainerError, ContainerResult};

/// Ensure the Linux bridge exists. Creates it with `ip` and `iptables`
/// commands if it doesn't already exist.
pub fn ensure_bridge(name: &str) -> ContainerResult<()> {
    // Check if bridge already exists
    let check = Command::new("ip").args(["link", "show", name]).output();

    if let Ok(output) = check {
        if output.status.success() {
            return Ok(());
        }
    }

    // Create bridge
    run_ip(["link", "add", name, "type", "bridge"])?;
    run_ip(["addr", "add", "10.88.0.1/16", "dev", name])?;
    run_ip(["link", "set", name, "up"])?;

    // Enable IP forwarding
    let _ = std::fs::write("/proc/sys/net/ipv4/ip_forward", "1");

    tracing::info!(bridge = name, "Bridge created and configured");
    Ok(())
}

fn run_ip(args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> ContainerResult<()> {
    let output =
        Command::new("ip")
            .args(args)
            .output()
            .map_err(|e| ContainerError::NetworkError {
                step: "run_ip",
                detail: format!("ip command failed: {e}"),
            })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerError::NetworkError {
            step: "run_ip",
            detail: format!("ip command failed: {stderr}"),
        });
    }

    Ok(())
}
