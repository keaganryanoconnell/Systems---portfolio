use nix::unistd::Pid;
use std::process::Command;

use crate::util::errors::{ContainerError, ContainerResult};

/// Create a veth pair.
pub fn create_pair(host_name: &str, peer_name: &str) -> ContainerResult<()> {
    run_ip([
        "link", "add", host_name, "type", "veth", "peer", "name", peer_name,
    ])
}

/// Move one end of a veth pair into a container's network namespace.
pub fn move_to_netns(interface: &str, pid: Pid) -> ContainerResult<()> {
    run_ip(["link", "set", interface, "netns", &pid.as_raw().to_string()])
}

/// Delete a veth interface from the host.
pub fn delete_interface(iface: &str) -> ContainerResult<()> {
    run_ip(["link", "delete", iface])
}

/// Bring a network interface up.
pub fn set_link_up(iface: &str) -> ContainerResult<()> {
    run_ip(["link", "set", iface, "up"])
}

fn run_ip(args: impl IntoIterator<Item = impl AsRef<std::ffi::OsStr>>) -> ContainerResult<()> {
    let output =
        Command::new("ip")
            .args(args)
            .output()
            .map_err(|e| ContainerError::NetworkError {
                step: "veth",
                detail: format!("ip command failed: {e}"),
            })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerError::NetworkError {
            step: "veth",
            detail: format!("ip command failed: {stderr}"),
        });
    }

    Ok(())
}
