pub mod address;
pub mod bridge;
pub mod dns;
pub mod iptables;
pub mod loopback;
pub mod veth;

use std::net::IpAddr;
use std::path::Path;

use nix::unistd::Pid;

use crate::container::config::ContainerConfig;
use crate::util::errors::{ContainerError, ContainerResult};

/// Write essential /etc files inside the container rootfs.
pub fn write_etc_files(config: &ContainerConfig) -> ContainerResult<()> {
    let rootfs = &config.rootfs_path;

    // /etc/hostname
    if let Some(ref hostname) = config.hostname {
        let hostname_path = rootfs.join("etc").join("hostname");
        if hostname_path.exists() {
            std::fs::write(&hostname_path, hostname.as_bytes()).map_err(|e| {
                ContainerError::FilesystemError {
                    step: "write_hostname",
                    detail: format!("failed to write {}", hostname_path.display()),
                    source: None,
                }
            })?;
        }
    }

    // /etc/hosts
    let hosts_path = rootfs.join("etc").join("hosts");
    if hosts_path.exists() {
        let mut hosts_content =
            String::from("127.0.0.1\tlocalhost\n::1\tlocalhost ip6-localhost ip6-loopback\n");
        if let Some(ref hostname) = config.hostname {
            hosts_content.push_str(&format!("127.0.0.1\t{hostname}\n"));
        }
        std::fs::write(&hosts_path, hosts_content.as_bytes()).map_err(|e| {
            ContainerError::FilesystemError {
                step: "write_hosts",
                detail: format!("failed to write {}", hosts_path.display()),
                source: None,
            }
        })?;
    }

    // /etc/resolv.conf (only if DNS servers configured)
    if !config.dns.is_empty() {
        let resolv_path = rootfs.join("etc").join("resolv.conf");
        if resolv_path.exists() {
            let mut resolv_content = String::new();
            for dns in &config.dns {
                resolv_content.push_str(&format!("nameserver {dns}\n"));
            }
            std::fs::write(&resolv_path, resolv_content.as_bytes()).map_err(|e| {
                ContainerError::FilesystemError {
                    step: "write_resolv",
                    detail: format!("failed to write {}", resolv_path.display()),
                    source: None,
                }
            })?;
        }
    }

    Ok(())
}

/// Set up networking for a container:
/// 1. Ensure bridge exists
/// 2. Create veth pair
/// 3. Move container peer into the container's network namespace
/// 4. Attach host peer to bridge
/// 5. Assign IP to container peer
/// 6. Set up NAT and port mappings
pub fn setup_networking(config: &ContainerConfig, child_pid: Pid) -> ContainerResult<()> {
    match config.network_mode {
        crate::container::config::NetworkMode::None => {
            tracing::info!("Network mode: none (loopback only)");
            Ok(())
        }
        crate::container::config::NetworkMode::Host => {
            tracing::info!("Network mode: host (shared with host)");
            Ok(())
        }
        crate::container::config::NetworkMode::Bridge => {
            let bridge_name = "cbr0";
            let host_iface = format!("veth-{}", config.id.short_id());
            let peer_iface = "eth0";

            // 1. Ensure bridge exists
            bridge::ensure_bridge(bridge_name)?;

            // 2. Create veth pair
            veth::create_pair(&host_iface, peer_iface)?;

            // 3. Move peer into container's netns
            veth::move_to_netns(peer_iface, child_pid)?;

            // 4. Attach host peer to bridge
            bridge::attach_interface(bridge_name, &host_iface)?;

            // 5. Bring up the host side
            veth::set_link_up(&host_iface)?;

            // 6. Assign IP to container peer
            address::assign_container_ip(peer_iface, child_pid, &config.id)?;

            // 7. Set up NAT
            iptables::setup_nat()?;

            // 8. Set up port mappings
            for pm in &config.port_mappings {
                iptables::add_port_mapping(pm, &config.id)?;
            }

            tracing::info!(
                bridge = bridge_name,
                host_iface = host_iface,
                "Network setup complete"
            );

            Ok(())
        }
    }
}

/// Tear down networking resources when a container stops.
pub fn teardown_networking(config: &ContainerConfig) -> ContainerResult<()> {
    if config.network_mode != crate::container::config::NetworkMode::Bridge {
        return Ok(());
    }

    let host_iface = format!("veth-{}", config.id.short_id());

    // Remove port mappings
    for pm in &config.port_mappings {
        let _ = iptables::remove_port_mapping(pm);
    }

    // Release IP address
    let _ = address::release_ip(&config.id);

    // Delete veth interface
    let _ = veth::delete_interface(&host_iface);

    tracing::info!("Network teardown complete");
    Ok(())
}
