use std::net::IpAddr;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::state::ContainerState;
use crate::util::id::ContainerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerConfig {
    pub id: ContainerId,
    pub rootfs_path: PathBuf,

    pub memory_limit_bytes: Option<u64>,
    pub memory_swap_bytes: Option<u64>,
    pub cpu_weight: Option<u16>,
    pub cpu_max: Option<(u64, u64)>,
    pub io_weight: Option<u16>,
    pub io_max_bps: Option<DeviceRate>,
    pub io_max_iops: Option<DeviceRate>,
    pub pids_max: Option<u32>,

    pub readonly_rootfs: bool,
    pub masked_paths: Vec<PathBuf>,
    pub readonly_paths: Vec<PathBuf>,

    pub hostname: Option<String>,
    pub dns: Vec<IpAddr>,
    pub port_mappings: Vec<PortMapping>,
    pub network_mode: NetworkMode,

    pub command: Vec<String>,
    pub working_dir: Option<PathBuf>,
    pub env: Vec<(String, String)>,
    pub init_process: bool,
}

impl ContainerConfig {
    pub fn builder() -> ContainerConfigBuilder {
        ContainerConfigBuilder::default()
    }

    pub fn state_dir(&self) -> PathBuf {
        PathBuf::from("/var/run/container-engine").join(self.id.as_str())
    }

    pub fn state_file_path(&self) -> PathBuf {
        self.state_dir().join("state.json")
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: PortProtocol,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortProtocol {
    Tcp,
    Udp,
}

impl std::fmt::Display for PortProtocol {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PortProtocol::Tcp => write!(f, "tcp"),
            PortProtocol::Udp => write!(f, "udp"),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct DeviceRate {
    pub major: u32,
    pub minor: u32,
    pub read_bps: Option<u64>,
    pub write_bps: Option<u64>,
    pub read_iops: Option<u64>,
    pub write_iops: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NetworkMode {
    None,
    Bridge,
    Host,
}

#[derive(Debug, Clone, Default)]
pub struct ContainerConfigBuilder {
    rootfs_path: Option<PathBuf>,
    memory_limit_bytes: Option<u64>,
    memory_swap_bytes: Option<u64>,
    cpu_weight: Option<u16>,
    cpu_max: Option<(u64, u64)>,
    io_weight: Option<u16>,
    io_max_bps: Option<DeviceRate>,
    io_max_iops: Option<DeviceRate>,
    pids_max: Option<u32>,
    readonly_rootfs: bool,
    hostname: Option<String>,
    dns: Vec<IpAddr>,
    port_mappings: Vec<PortMapping>,
    network_mode: NetworkMode,
    command: Vec<String>,
    working_dir: Option<PathBuf>,
    env: Vec<(String, String)>,
    init_process: bool,
}

impl ContainerConfigBuilder {
    pub fn rootfs(mut self, path: PathBuf) -> Self {
        self.rootfs_path = Some(path);
        self
    }

    pub fn memory_limit_mb(mut self, mb: u64) -> Self {
        self.memory_limit_bytes = Some(mb * 1024 * 1024);
        self
    }

    pub fn memory_swap_mb(mut self, mb: u64) -> Self {
        self.memory_swap_bytes = Some(mb * 1024 * 1024);
        self
    }

    pub fn cpu_weight(mut self, weight: u16) -> Self {
        self.cpu_weight = Some(weight);
        self
    }

    pub fn cpu_max(mut self, quota: u64, period: u64) -> Self {
        self.cpu_max = Some((quota, period));
        self
    }

    pub fn pids_max(mut self, max: u32) -> Self {
        self.pids_max = Some(max);
        self
    }

    pub fn readonly_rootfs(mut self, ro: bool) -> Self {
        self.readonly_rootfs = ro;
        self
    }

    pub fn hostname(mut self, hostname: String) -> Self {
        self.hostname = Some(hostname);
        self
    }

    pub fn dns(mut self, dns: Vec<IpAddr>) -> Self {
        self.dns = dns;
        self
    }

    pub fn port_mapping(mut self, pm: PortMapping) -> Self {
        self.port_mappings.push(pm);
        self
    }

    pub fn network_mode(mut self, mode: NetworkMode) -> Self {
        self.network_mode = mode;
        self
    }

    pub fn command(mut self, cmd: Vec<String>) -> Self {
        self.command = cmd;
        self
    }

    pub fn working_dir(mut self, dir: PathBuf) -> Self {
        self.working_dir = Some(dir);
        self
    }

    pub fn env(mut self, key: String, val: String) -> Self {
        self.env.push((key, val));
        self
    }

    pub fn init_process(mut self, enable: bool) -> Self {
        self.init_process = enable;
        self
    }

    pub fn build(self) -> Result<ContainerConfig, crate::util::errors::ContainerError> {
        let rootfs = self.rootfs_path.ok_or_else(|| {
            crate::util::errors::ContainerError::ConfigError("rootfs_path is required".into())
        })?;

        if !rootfs.exists() {
            return Err(crate::util::errors::ContainerError::ConfigError(format!(
                "rootfs path does not exist: {}",
                rootfs.display()
            )));
        }

        Ok(ContainerConfig {
            id: crate::util::id::ContainerId::generate(),
            rootfs_path: rootfs,
            memory_limit_bytes: self.memory_limit_bytes,
            memory_swap_bytes: self.memory_swap_bytes,
            cpu_weight: self.cpu_weight,
            cpu_max: self.cpu_max,
            io_weight: self.io_weight,
            io_max_bps: self.io_max_bps,
            io_max_iops: self.io_max_iops,
            pids_max: self.pids_max.or(Some(256)),
            readonly_rootfs: self.readonly_rootfs,
            masked_paths: self.masked_paths,
            readonly_paths: self.readonly_paths,
            hostname: self.hostname,
            dns: self.dns,
            port_mappings: self.port_mappings,
            network_mode: self.network_mode,
            command: self.command,
            working_dir: self.working_dir,
            env: self.env,
            init_process: self.init_process,
        })
    }
}
