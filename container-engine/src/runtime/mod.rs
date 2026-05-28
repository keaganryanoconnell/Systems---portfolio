pub mod create;
pub mod delete;
pub mod exec;
pub mod kill;
pub mod start;

use std::io::Write;

use nix::sys::signal::Signal;
use nix::unistd::Pid;

use crate::container::config::{ContainerConfig, ContainerData};
use crate::container::lifecycle;
use crate::container::state::ContainerState;
use crate::monitor;
use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

/// Create + start a container from configuration.
pub fn run(config: ContainerConfig) -> ContainerResult<()> {
    let id = config.id.clone();

    // 1. Create (validate + persist)
    let data = lifecycle::create(config)?;
    let config = data.config;

    tracing::info!(id = %id, "Running container");

    // 2. Isolate (clone with namespaces)
    let isolated = crate::isolate::isolate(&config)?;

    // 3. Transition to Running state
    lifecycle::transition(&id, ContainerState::Running, Some(isolated.init_pid), None)?;

    // 4. Setup cgroups
    let cgroup_guard = crate::cgroups::setup_cgroups(&config, isolated.init_pid)?;

    // 5. Setup networking
    if let Err(e) = crate::network::setup_networking(&config, isolated.init_pid) {
        tracing::warn!(error = %e, "Network setup failed (non-fatal)");
    }

    // 6. Record event
    let _ = monitor::events::record_event(
        monitor::events::EventType::Started,
        id.as_str(),
        format!("Container started (PID {})", isolated.init_pid),
    );

    // 7. Wait for the init process to exit
    let exit_status = wait_for_process(isolated.init_pid);

    // 8. Cleanup
    cleanup(id.as_str(), isolated.init_pid, exit_status, cgroup_guard)?;

    Ok(())
}

/// Execute a command inside an existing running container.
pub fn exec(id: &ContainerId, command: Vec<String>) -> ContainerResult<()> {
    let data = lifecycle::load(id)?;
    if data.state != ContainerState::Running {
        return Err(ContainerError::Internal(format!(
            "Container {id} is not running (state: {})",
            data.state
        )));
    }

    let pid = Pid::from_raw(
        data.pid
            .ok_or_else(|| ContainerError::Internal(format!("Container {id} has no PID")))?,
    );

    if command.is_empty() {
        return Err(ContainerError::ConfigError(
            "exec requires a command".into(),
        ));
    }

    // Use nsenter to enter the container's namespaces and execute the command
    let mut cmd = std::process::Command::new("nsenter");
    cmd.arg("-t")
        .arg(pid.as_raw().to_string())
        .arg("-m")
        .arg("-u")
        .arg("-i")
        .arg("-n")
        .arg("-p")
        .arg("-U")
        .arg("--preserve-credentials")
        .arg("--")
        .args(&command);

    let status = cmd
        .status()
        .map_err(|e| ContainerError::Internal(format!("nsenter exec failed: {e}")))?;

    std::process::exit(status.code().unwrap_or(1));
}

/// Send a signal to a container's init process.
pub fn kill(id: &ContainerId, signal_str: String) -> ContainerResult<()> {
    let data = lifecycle::load(id)?;
    let pid = Pid::from_raw(
        data.pid
            .ok_or_else(|| ContainerError::Internal(format!("Container {id} has no PID")))?,
    );

    let signal = crate::process::signal::parse_signal(&signal_str)
        .map_err(|e| ContainerError::Internal(e))?;

    crate::process::signal::send_signal_to_container(pid, signal)?;

    if signal == Signal::SIGKILL || signal == Signal::SIGTERM || signal == Signal::SIGQUIT {
        lifecycle::transition(id, ContainerState::Stopped, None, None)?;
        let _ = monitor::events::record_event(
            monitor::events::EventType::Killed,
            id.as_str(),
            format!("Container killed with signal {signal:?}"),
        );
    }

    tracing::info!(id = %id, signal = %signal_str, "Signal sent");
    Ok(())
}

/// List containers.
pub fn ps(all: bool) -> ContainerResult<()> {
    let containers = lifecycle::list_all()?;
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    if containers.is_empty() {
        writeln!(handle, "No containers found.")?;
        return Ok(());
    }

    // Header
    writeln!(
        handle,
        "{:<13} {:<10} {:<8} {:<16} {:<20}",
        "CONTAINER ID", "STATE", "PID", "CREATED", "COMMAND"
    )?;
    writeln!(handle, "{}", "-".repeat(80))?;

    for c in &containers {
        if !all && c.state == ContainerState::Stopped {
            continue;
        }
        let pid_str = c.pid.map(|p| p.to_string()).unwrap_or_else(|| "-".into());
        let cmd = c
            .config
            .command
            .first()
            .cloned()
            .unwrap_or_else(|| "/bin/sh".to_string());
        writeln!(
            handle,
            "{:<13} {:<10} {:<8} {:<16} {:<20}",
            c.id,
            c.state.as_str(),
            pid_str,
            c.created_at,
            if cmd.len() > 18 {
                format!("{}...", &cmd[..18])
            } else {
                cmd
            }
        )?;
    }

    Ok(())
}

/// Display resource stats for a container (or all containers).
pub fn stats(id: Option<ContainerId>) -> ContainerResult<()> {
    let stdout = std::io::stdout();
    let mut handle = stdout.lock();

    if let Some(ref id) = id {
        let stats = monitor::stats::read_cgroup_stats(id)?;
        writeln!(handle, "Container: {}", stats.id)?;
        writeln!(handle, "  CPU usage:    {} us", stats.cpu_usage_us)?;
        writeln!(
            handle,
            "  CPU throttled: {} / {} periods",
            stats.cpu_nr_throttled, stats.cpu_nr_periods
        )?;
        writeln!(
            handle,
            "  Memory:       {} / {} bytes",
            stats.memory_usage_bytes, stats.memory_max_bytes
        )?;
        writeln!(
            handle,
            "  PIDs:         {} / {}",
            stats.pids_current, stats.pids_limit
        )?;
        writeln!(
            handle,
            "  IO read:      {} bytes ({} ops)",
            stats.io_read_bytes, stats.io_read_ops
        )?;
        writeln!(
            handle,
            "  IO write:     {} bytes ({} ops)",
            stats.io_write_bytes, stats.io_write_ops
        )?;
    } else {
        let containers = lifecycle::list_all()?;
        for c in containers {
            if c.state != ContainerState::Running {
                continue;
            }
            if let Ok(stats) = monitor::stats::read_cgroup_stats(&c.id) {
                writeln!(
                    handle,
                    "{}  CPU: {:>8}us  MEM: {:>8}/{:>8}  PID: {}/{}",
                    c.id,
                    stats.cpu_usage_us,
                    stats.memory_usage_bytes,
                    stats.memory_max_bytes,
                    stats.pids_current,
                    stats.pids_limit,
                )?;
            }
        }
    }

    Ok(())
}

/// Inspect a container (print full config as JSON).
pub fn inspect(id: &ContainerId) -> ContainerResult<()> {
    let data = lifecycle::load(id)?;
    let json = serde_json::to_string_pretty(&data)?;
    println!("{json}");
    Ok(())
}

/// Fetch container events.
pub fn logs(id: &ContainerId) -> ContainerResult<()> {
    let events = monitor::events::get_events(Some(id.as_str()));
    if events.is_empty() {
        println!("No events for container {id}");
    }
    for event in &events {
        println!("[{}] {}: {}", event.timestamp_ns, event.id, event.message);
    }
    Ok(())
}

/// Pause a container (freeze all processes via cgroup).
pub fn pause(id: &ContainerId) -> ContainerResult<()> {
    let data = lifecycle::load(id)?;
    if data.state != ContainerState::Running {
        return Err(ContainerError::StateTransitionError {
            from: data.state,
            to: ContainerState::Paused,
        });
    }

    let cgroup_path =
        std::path::Path::new("/sys/fs/cgroup").join(format!("container-{}", id.short_id()));

    std::fs::write(cgroup_path.join("cgroup.freeze"), "1").map_err(|e| {
        ContainerError::CgroupError {
            controller: "freeze",
            detail: format!("failed to freeze cgroup for {id}: {e}"),
            source: Some(e),
        }
    })?;

    lifecycle::transition(id, ContainerState::Paused, None, None)?;
    let _ = monitor::events::record_event(
        monitor::events::EventType::Paused,
        id.as_str(),
        "Container paused".into(),
    );

    tracing::info!(id = %id, "Container paused");
    Ok(())
}

/// Resume a paused container.
pub fn resume(id: &ContainerId) -> ContainerResult<()> {
    let data = lifecycle::load(id)?;
    if data.state != ContainerState::Paused {
        return Err(ContainerError::StateTransitionError {
            from: data.state,
            to: ContainerState::Running,
        });
    }

    let cgroup_path =
        std::path::Path::new("/sys/fs/cgroup").join(format!("container-{}", id.short_id()));

    std::fs::write(cgroup_path.join("cgroup.freeze"), "0").map_err(|e| {
        ContainerError::CgroupError {
            controller: "freeze",
            detail: format!("failed to unfreeze cgroup for {id}: {e}"),
            source: Some(e),
        }
    })?;

    lifecycle::transition(id, ContainerState::Running, None, None)?;
    let _ = monitor::events::record_event(
        monitor::events::EventType::Resumed,
        id.as_str(),
        "Container resumed".into(),
    );

    tracing::info!(id = %id, "Container resumed");
    Ok(())
}

/// Remove a container and clean up all resources.
pub fn remove(id: &ContainerId, force: bool) -> ContainerResult<()> {
    let data = lifecycle::load(id)?;

    if data.state.is_active() && !force {
        return Err(ContainerError::Internal(format!(
            "Container {id} is still running. Use --force to remove."
        )));
    }

    if data.state.is_active() && force {
        // Kill the container first
        if let Some(pid) = data.pid.map(Pid::from_raw) {
            let _ = crate::process::signal::send_signal_to_container(pid, Signal::SIGKILL);
        }
    }

    // Cleanup: cgroups, network, state
    cleanup_cgroup(id)?;
    cleanup_network(&data.config)?;
    lifecycle::remove_state(id)?;

    let _ = monitor::events::record_event(
        monitor::events::EventType::Created,
        id.as_str(),
        "Container removed".into(),
    );

    tracing::info!(id = %id, "Container removed");
    Ok(())
}

// ── Internal helpers ──

fn wait_for_process(pid: Pid) -> i32 {
    use nix::sys::wait::{waitpid, WaitPidFlag};

    loop {
        match waitpid(pid, Some(WaitPidFlag::WUNTRACED)) {
            Ok(status) => {
                return match status {
                    nix::sys::wait::WaitStatus::Exited(_, code) => code,
                    nix::sys::wait::WaitStatus::Signaled(_, sig, _) => 128 + sig as i32,
                    _ => {
                        std::thread::sleep(std::time::Duration::from_millis(50));
                        continue;
                    }
                };
            }
            Err(e) => {
                tracing::error!(error = %e, pid = %pid, "waitpid failed");
                return -1;
            }
        }
    }
}

fn cleanup(
    id: &str,
    pid: Pid,
    exit_code: i32,
    cgroup_guard: crate::cgroups::CgroupGuard,
) -> ContainerResult<()> {
    let container_id = ContainerId::from_str(id)
        .ok_or_else(|| ContainerError::Internal(format!("invalid container ID: {id}")))?;

    // 1. Transition state
    let _ = lifecycle::transition(
        &container_id,
        ContainerState::Stopped,
        Some(pid),
        Some(exit_code),
    );

    // 2. Record event
    let _ = monitor::events::record_event(
        monitor::events::EventType::Stopped,
        id,
        format!("Container exited with code {exit_code}"),
    );

    // 3. Cgroup cleanup (via guard Drop)
    drop(cgroup_guard);

    // 4. Network cleanup
    if let Ok(data) = lifecycle::load(&container_id) {
        let _ = crate::network::teardown_networking(&data.config);
    }

    tracing::info!(id = id, exit_code = exit_code, "Container cleanup complete");
    Ok(())
}

fn cleanup_cgroup(id: &ContainerId) -> ContainerResult<()> {
    let cgroup_path =
        std::path::Path::new("/sys/fs/cgroup").join(format!("container-{}", id.short_id()));

    if cgroup_path.exists() {
        // Kill all processes in the cgroup
        let _ = std::fs::write(cgroup_path.join("cgroup.kill"), "1");
        // Remove the cgroup directory
        let _ = std::fs::remove_dir(&cgroup_path);
    }

    Ok(())
}

fn cleanup_network(config: &ContainerConfig) -> ContainerResult<()> {
    crate::network::teardown_networking(config)
}
