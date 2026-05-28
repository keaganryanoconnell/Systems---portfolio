use std::collections::HashMap;
use std::net::IpAddr;
use std::path::PathBuf;
use std::sync::Mutex;

use nix::unistd::Pid;
use once_cell::sync::Lazy;

use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

/// IP allocation state file path
const IP_POOL_PATH: &str = "/var/run/container-engine/ip-pool.json";

/// In-memory IP allocation map for the 10.88.0.0/16 subnet.
/// Uses a simple sequential allocator from 10.88.0.2 to 10.88.255.254.
static IP_ALLOCATOR: Lazy<Mutex<IpAllocator>> =
    Lazy::new(|| Mutex::new(IpAllocator::load_or_new()));

struct IpAllocator {
    next_ip: u32,                         // Last octet of 10.88.0.x (x component)
    allocations: HashMap<String, String>, // container_id -> ip_string
}

impl IpAllocator {
    fn load_or_new() -> Self {
        // Try to load from disk
        if let Ok(data) = std::fs::read_to_string(IP_POOL_PATH) {
            if let Ok(map) = serde_json::from_str::<HashMap<String, String>>(&data) {
                let next = map.len() as u32 + 2; // Start after .1 (gateway)
                return IpAllocator {
                    next_ip: next.max(2),
                    allocations: map,
                };
            }
        }
        IpAllocator {
            next_ip: 2,
            allocations: HashMap::new(),
        }
    }

    fn allocate(&mut self, id: &ContainerId) -> String {
        if let Some(ip) = self.allocations.get(id.as_str()) {
            return ip.clone();
        }
        let ip = format!("10.88.0.{}", self.next_ip);
        self.next_ip += 1;
        self.allocations.insert(id.to_string(), ip.clone());
        self.save();
        ip
    }

    fn release(&mut self, id: &ContainerId) {
        self.allocations.remove(id.as_str());
        self.save();
    }

    fn save(&self) {
        if let Ok(json) = serde_json::to_string(&self.allocations) {
            let _ = std::fs::write(IP_POOL_PATH, &json);
        }
    }
}

/// Assign an IP address to the container's network interface.
/// Runs inside the container's network namespace.
pub fn assign_container_ip(
    iface: &str,
    child_pid: Pid,
    id: &ContainerId,
) -> ContainerResult<IpAddr> {
    let ip: String = {
        let mut allocator = IP_ALLOCATOR.lock().unwrap_or_else(|e| e.into_inner());
        allocator.allocate(id)
    };

    // Assign IP inside the container's netns using nsenter
    let output = std::process::Command::new("nsenter")
        .args([
            "-t",
            &child_pid.as_raw().to_string(),
            "-n",
            "ip",
            "addr",
            "add",
            &format!("{ip}/16"),
            "dev",
            iface,
        ])
        .output()
        .map_err(|e| ContainerError::NetworkError {
            step: "assign_ip",
            detail: format!("nsenter ip addr add failed: {e}"),
        })?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(ContainerError::NetworkError {
            step: "assign_ip",
            detail: format!("nsenter ip addr add failed: {stderr}"),
        });
    }

    // Bring interface up inside netns
    let _ = std::process::Command::new("nsenter")
        .args([
            "-t",
            &child_pid.as_raw().to_string(),
            "-n",
            "ip",
            "link",
            "set",
            iface,
            "up",
        ])
        .output();

    // Set default route inside netns
    let _ = std::process::Command::new("nsenter")
        .args([
            "-t",
            &child_pid.as_raw().to_string(),
            "-n",
            "ip",
            "route",
            "add",
            "default",
            "via",
            "10.88.0.1",
        ])
        .output();

    let addr: IpAddr = ip
        .parse()
        .unwrap_or(IpAddr::V4(std::net::Ipv4Addr::new(10, 88, 0, 2)));
    tracing::info!(ip = %ip, iface = iface, "Container IP assigned");
    Ok(addr)
}

/// Release an IP address back to the pool.
pub fn release_ip(id: &ContainerId) {
    let mut allocator = IP_ALLOCATOR.lock().unwrap_or_else(|e| e.into_inner());
    allocator.release(id);
}
