use nix::unistd::Pid;
use std::time::{Duration, Instant};

use crate::util::errors::ContainerResult;

/// Health check result.
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Unhealthy(String),
}

/// Run a health check inside a container by executing a command
/// via nsenter and checking the exit code.
pub fn run_health_check(
    pid: Pid,
    command: &[String],
    timeout_secs: u64,
) -> ContainerResult<HealthStatus> {
    if command.is_empty() {
        // Default health check: just verify the process is alive
        return match nix::sys::signal::kill(pid, None) {
            Ok(()) => Ok(HealthStatus::Healthy),
            Err(_) => Ok(HealthStatus::Unhealthy("process not found".into())),
        };
    }

    let start = Instant::now();
    let deadline = Duration::from_secs(timeout_secs);

    // Build nsenter command to execute inside the container's namespaces
    let mut nsenter_cmd = std::process::Command::new("nsenter");
    nsenter_cmd
        .arg("-t")
        .arg(pid.as_raw().to_string())
        .arg("-m")
        .arg("-u")
        .arg("-i")
        .arg("-n")
        .arg("-p")
        .arg("--")
        .args(command);

    let output = nsenter_cmd.output().map_err(|e| {
        crate::util::errors::ContainerError::Internal(format!("health check nsenter failed: {e}"))
    })?;

    let elapsed = start.elapsed();
    if elapsed > deadline {
        return Ok(HealthStatus::Unhealthy(format!(
            "health check timed out after {timeout_secs}s"
        )));
    }

    if output.status.success() {
        Ok(HealthStatus::Healthy)
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Ok(HealthStatus::Unhealthy(format!(
            "exit code: {}, stderr: {stderr}",
            output.status.code().unwrap_or(-1)
        )))
    }
}
