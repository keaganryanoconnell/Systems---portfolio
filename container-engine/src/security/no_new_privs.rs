use nix::sys::prctl;

use crate::util::errors::{ContainerError, ContainerResult};

/// Set PR_SET_NO_NEW_PRIVS on the current process.
/// This is a one-way operation: once set, neither the process nor its
/// children can gain new privileges (e.g., via setuid binaries, file
/// capabilities, or ambivalent capabilities).
///
/// This MUST be called before execve() and ideally before any other
/// security setup to prevent privilege escalation attacks.
pub fn set_no_new_privs() -> ContainerResult<()> {
    // prctl(PR_SET_NO_NEW_PRIVS, 1, 0, 0, 0)
    prctl::set_no_new_privs()
        .map_err(|e| ContainerError::Internal(format!("PR_SET_NO_NEW_PRIVS failed: {e}")))
}

/// Check whether NO_NEW_PRIVS is active.
pub fn is_no_new_privs_active() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .map(|s| s.lines().any(|l| l.contains("NoNewPrivs:\t1")))
        .unwrap_or(false)
}
