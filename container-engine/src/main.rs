#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("Error: container-engine requires a Linux host operating system.");
    eprintln!("The container runtime depends on Linux kernel primitives:");
    eprintln!("  - clone() with CLONE_NEW* namespace flags");
    eprintln!("  - cgroups v2 filesystem at /sys/fs/cgroup/");
    eprintln!("  - pivot_root(), mount(), and seccomp() syscalls");
    std::process::exit(1);
}

#[cfg(target_os = "linux")]
fn main() {
    use clap::Parser;
    use container_engine::cli::{Cli, Commands};
    use container_engine::container::state_file::ensure_state_dir;
    use container_engine::container::{config::ContainerConfig, state_file};
    use container_engine::runtime;
    use container_engine::util::id::ContainerId;
    use std::io::Write;

    // Initialize structured logging
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with_target(false)
        .init();

    // Ensure state directory exists
    if let Err(e) = ensure_state_dir() {
        eprintln!("Warning: could not create state directory: {e}");
    }

    let cli = Cli::parse();

    let result = match &cli.command {
        Commands::Run {
            rootfs,
            memory_mb,
            cpus,
            cpu_weight,
            pids_max,
            hostname,
            dns,
            port,
            readonly,
            network,
            debug_config,
            command,
        } => {
            let network_mode = match network.as_str() {
                "none" => crate::container::config::NetworkMode::None,
                "host" => crate::container::config::NetworkMode::Host,
                _ => crate::container::config::NetworkMode::Bridge,
            };

            let config_result = {
                let mut builder = ContainerConfig::builder()
                    .rootfs(rootfs.clone())
                    .memory_limit_mb(*memory_mb)
                    .cpu_weight(*cpu_weight)
                    .pids_max(*pids_max)
                    .readonly_rootfs(*readonly)
                    .network_mode(network_mode);

                if let Some(cpus) = cpus {
                    let period = 100_000u64;
                    let quota = (*cpus * period as f64) as u64;
                    builder = builder.cpu_max(quota, period);
                }

                if let Some(hostname) = hostname {
                    builder = builder.hostname(hostname.clone());
                }

                for dns_server in dns {
                    builder = builder.dns(vec![*dns_server]);
                }

                for pm in port {
                    builder = builder.port_mapping(pm.clone());
                }

                if !command.is_empty() {
                    builder = builder.command(command.clone());
                }

                builder.build()
            };

            match config_result {
                Ok(config) => {
                    if *debug_config {
                        let json = serde_json::to_string_pretty(&config).unwrap_or_default();
                        eprintln!("{json}");
                    }
                    runtime::run(config)
                }
                Err(e) => Err(e),
            }
        }

        Commands::Exec { id, command } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::exec(&id, command.clone())
        }

        Commands::Kill { id, signal } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::kill(&id, signal.clone())
        }

        Commands::Ps { all } => runtime::ps(*all),

        Commands::Stats { id } => {
            let id = id.as_ref().and_then(|s| ContainerId::from_str(s));
            runtime::stats(id)
        }

        Commands::Inspect { id } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::inspect(&id)
        }

        Commands::Logs { id } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::logs(&id)
        }

        Commands::Pause { id } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::pause(&id)
        }

        Commands::Resume { id } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::resume(&id)
        }

        Commands::Rm { id, force } => {
            let id = ContainerId::from_str(id).ok_or_else(|| {
                crate::util::errors::ContainerError::Internal(format!("invalid container ID: {id}"))
            })?;
            runtime::remove(&id, *force)
        }
    };

    match result {
        Ok(_) => {}
        Err(e) => {
            // Errors are already formatted by thiserror Display impl
            let stderr = std::io::stderr();
            let mut handle = stderr.lock();
            let _ = writeln!(handle, "Error: {e}");
            std::process::exit(1);
        }
    }
}
