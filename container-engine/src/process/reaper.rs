use nix::sys::wait::{waitpid, WaitPidFlag, WaitStatus};
use nix::unistd::Pid;

use crate::util::errors::ContainerResult;

/// Reap any zombie child processes.
/// In a PID namespace, when a process dies, its children are reparented
/// to PID 1. The init process must reap them to prevent zombies.
///
/// This function performs a non-blocking wait for any child.
pub fn reap_zombies() -> ContainerResult<usize> {
    let mut reaped = 0;
    loop {
        match waitpid(Pid::from_raw(-1), Some(WaitPidFlag::WNOHANG)) {
            Ok(WaitStatus::Exited(pid, _code)) => {
                tracing::debug!(pid = %pid, "Reaped zombie");
                reaped += 1;
            }
            Ok(WaitStatus::Signaled(pid, _sig, _)) => {
                tracing::debug!(pid = %pid, "Reaped zombie (signaled)");
                reaped += 1;
            }
            Ok(_) => {
                // StillAlive or other — no more zombies right now
                break;
            }
            Err(nix::Error::ECHILD) => {
                // No children at all
                break;
            }
            Err(e) => {
                tracing::warn!(error = %e, "Reaper: waitpid error");
                break;
            }
        }
    }
    Ok(reaped)
}
