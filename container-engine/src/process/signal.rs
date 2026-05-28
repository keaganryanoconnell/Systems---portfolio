use nix::sys::signal::{kill, Signal};
use nix::unistd::Pid;

use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

/// Send a signal to a container's init process.
pub fn send_signal_to_container(pid: Pid, signal: Signal) -> ContainerResult<()> {
    kill(pid, signal).map_err(|e| {
        ContainerError::Internal(format!(
            "failed to send signal {signal:?} to PID {pid}: {e}"
        ))
    })
}

/// Parse a signal name (e.g., "SIGTERM", "TERM", "9") into a Signal.
pub fn parse_signal(name: &str) -> Result<Signal, String> {
    let upper = name.to_uppercase();
    let signal_str = if upper.starts_with("SIG") {
        upper
    } else {
        format!("SIG{upper}")
    };

    match signal_str.as_str() {
        "SIGTERM" => Ok(Signal::SIGTERM),
        "SIGINT" => Ok(Signal::SIGINT),
        "SIGKILL" => Ok(Signal::SIGKILL),
        "SIGQUIT" => Ok(Signal::SIGQUIT),
        "SIGHUP" => Ok(Signal::SIGHUP),
        "SIGSTOP" => Ok(Signal::SIGSTOP),
        "SIGCONT" => Ok(Signal::SIGCONT),
        "SIGUSR1" => Ok(Signal::SIGUSR1),
        "SIGUSR2" => Ok(Signal::SIGUSR2),
        _ => {
            // Try parsing as a number
            if let Ok(num) = name.parse::<i32>() {
                Signal::try_from(num).map_err(|_| format!("unknown signal number: {num}"))
            } else {
                Err(format!("unknown signal: {name}"))
            }
        }
    }
}
