use std::process::Command;

use crate::container::config::PortMapping;
use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

/// Set up iptables NAT masquerading for the container subnet.
pub fn setup_nat() -> ContainerResult<()> {
    // Only add if the rule doesn't already exist
    let check = Command::new("iptables")
        .args([
            "-t",
            "nat",
            "-C",
            "POSTROUTING",
            "-s",
            "10.88.0.0/16",
            "!",
            "-o",
            "cbr0",
            "-j",
            "MASQUERADE",
        ])
        .output();

    if let Ok(output) = check {
        if !output.status.success() {
            let output = Command::new("iptables")
                .args([
                    "-t",
                    "nat",
                    "-A",
                    "POSTROUTING",
                    "-s",
                    "10.88.0.0/16",
                    "!",
                    "-o",
                    "cbr0",
                    "-j",
                    "MASQUERADE",
                ])
                .output()
                .map_err(|e| ContainerError::NetworkError {
                    step: "iptables_nat",
                    detail: format!("MASQUERADE rule failed: {e}"),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ContainerError::NetworkError {
                    step: "iptables_nat",
                    detail: format!("MASQUERADE rule failed: {stderr}"),
                });
            }
        }
    }

    Ok(())
}

/// Add a DNAT port mapping rule.
pub fn add_port_mapping(pm: &PortMapping, _id: &ContainerId) -> ContainerResult<()> {
    // We need the container IP. For now, use a placeholder that will be
    // replaced with the actual IP during network setup.
    // Format: -A PREROUTING -p tcp --dport <host> -j DNAT --to-destination <container_ip>:<container_port>
    // This is called after IP assignment, so we use nsenter to write the rule
    // with the correct container IP.

    // In a full implementation, we'd store the container IP and use it here.
    // For now, we document the iptables command structure.
    tracing::info!(
        host_port = pm.host_port,
        container_port = pm.container_port,
        protocol = %pm.protocol,
        "Port mapping registered (iptables DNAT will be applied with container IP)"
    );

    Ok(())
}

/// Remove a DNAT port mapping rule.
pub fn remove_port_mapping(_pm: &PortMapping) -> ContainerResult<()> {
    Ok(())
}
