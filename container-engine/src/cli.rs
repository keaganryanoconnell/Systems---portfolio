use std::net::IpAddr;
use std::path::PathBuf;

use clap::{Parser, Subcommand};

use crate::container::config::{DeviceRate, NetworkMode, PortMapping, PortProtocol};

fn parse_port_mapping(s: &str) -> Result<PortMapping, String> {
    let parts: Vec<&str> = s.splitn(3, ':').collect();
    if parts.len() < 2 {
        return Err(format!(
            "invalid port mapping: {s} (expected host:container or host:container/protocol)"
        ));
    }
    let host_port: u16 = parts[0]
        .parse()
        .map_err(|_| format!("invalid host port: {}", parts[0]))?;
    let rest = parts[1];
    let (container_port_str, protocol_str) = if let Some(idx) = rest.find('/') {
        (&rest[..idx], Some(&rest[idx + 1..]))
    } else {
        (rest, None)
    };
    let container_port: u16 = container_port_str
        .parse()
        .map_err(|_| format!("invalid container port: {container_port_str}"))?;
    let protocol = match protocol_str {
        Some("udp") => PortProtocol::Udp,
        _ => PortProtocol::Tcp,
    };
    Ok(PortMapping {
        host_port,
        container_port,
        protocol,
    })
}

#[derive(Parser)]
#[command(
    name = "container-engine",
    author,
    version,
    about = "Production-grade Linux container runtime"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create and start a container
    #[command(name = "run")]
    Run {
        /// Path to the unzipped Linux root filesystem directory
        rootfs: PathBuf,

        /// Memory limit in megabytes (default: 256)
        #[arg(long, default_value = "256")]
        memory_mb: u64,

        /// Number of CPU cores to limit to (e.g. 1.5)
        #[arg(long)]
        cpus: Option<f64>,

        /// CPU weight for CFS scheduler (1-10000, default: 100)
        #[arg(long, default_value = "100")]
        cpu_weight: u16,

        /// Maximum number of PIDs (default: 256)
        #[arg(long, default_value = "256")]
        pids_max: u32,

        /// Container hostname
        #[arg(long)]
        hostname: Option<String>,

        /// DNS nameservers
        #[arg(long)]
        dns: Vec<IpAddr>,

        /// Port mapping: host_port:container_port[/protocol]
        #[arg(long = "port", value_parser = parse_port_mapping)]
        port: Vec<PortMapping>,

        /// Mount rootfs as read-only via OverlayFS
        #[arg(long)]
        readonly: bool,

        /// Network mode: none, bridge, host
        #[arg(long, default_value = "bridge")]
        network: String,

        /// Print container config in JSON to stderr before starting
        #[arg(long)]
        debug_config: bool,

        /// Command and arguments to run inside the container
        #[arg(last = true)]
        command: Vec<String>,
    },

    /// Execute a command in an existing running container
    #[command(name = "exec")]
    Exec {
        /// Container ID
        id: String,

        /// Command and arguments to execute
        #[arg(last = true)]
        command: Vec<String>,
    },

    /// Send a signal to the container init process
    #[command(name = "kill")]
    Kill {
        /// Container ID
        id: String,

        /// Signal to send (default: SIGTERM)
        #[arg(long, default_value = "SIGTERM")]
        signal: String,
    },

    /// List containers
    #[command(name = "ps")]
    Ps {
        /// Show all containers (including stopped)
        #[arg(long, short)]
        all: bool,
    },

    /// Display live resource usage for a container
    #[command(name = "stats")]
    Stats {
        /// Container ID (optional; lists all if omitted)
        id: Option<String>,
    },

    /// Display detailed container configuration and state
    #[command(name = "inspect")]
    Inspect {
        /// Container ID
        id: String,
    },

    /// Fetch container logs
    #[command(name = "logs")]
    Logs {
        /// Container ID
        id: String,
    },

    /// Pause all processes in a container (freeze cgroup)
    #[command(name = "pause")]
    Pause {
        /// Container ID
        id: String,
    },

    /// Unpause a frozen container
    #[command(name = "resume")]
    Resume {
        /// Container ID
        id: String,
    },

    /// Remove a container and clean up all resources
    #[command(name = "rm")]
    Rm {
        /// Container ID
        id: String,

        /// Force removal even if running
        #[arg(long)]
        force: bool,
    },
}
