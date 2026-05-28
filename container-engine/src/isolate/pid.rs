/// PID namespace isolation is handled via CLONE_NEWPID in the clone() flags.
/// After clone(), the child becomes PID 1 inside the new namespace.
///
/// This module provides helpers for PID namespace operations.
use nix::unistd::Pid;

/// Verify that the current process is PID 1 inside its PID namespace.
pub fn is_init_process() -> bool {
    match std::fs::read_to_string("/proc/self/status") {
        Ok(status) => status.lines().any(|line| line.starts_with("Pid:\t1")),
        Err(_) => false,
    }
}

/// Get the PID of the current process as seen by the initial PID namespace.
/// This requires /proc/self/status to be available.
pub fn get_outer_pid() -> Option<Pid> {
    std::fs::read_to_string("/proc/self/status")
        .ok()
        .and_then(|status| {
            for line in status.lines() {
                if line.starts_with("NSpid:") {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 3 {
                        // Last entry is the outermost PID
                        return parts.last()?.parse().ok().map(Pid::from_raw);
                    }
                }
            }
            None
        })
}
