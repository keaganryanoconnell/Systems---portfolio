pub mod capabilities;
pub mod no_new_privs;
pub mod seccomp;

use crate::container::config::ContainerConfig;
use crate::util::errors::{ContainerError, ContainerResult};

/// Apply all security configurations before executing the container command.
/// Order matters: no_new_privs must come first, then capabilities, then seccomp.
pub fn apply_security(config: &ContainerConfig) -> ContainerResult<()> {
    // 1. Set NO_NEW_PRIVS — this is irreversible, prevents gaining new privileges
    no_new_privs::set_no_new_privs()
        .map_err(|e| ContainerError::Internal(format!("PR_SET_NO_NEW_PRIVS failed: {e}")))?;

    // 2. Drop capabilities from the bounding set
    capabilities::apply_capabilities()?;

    // 3. Install seccomp-BPF filter
    seccomp::apply_seccomp_filter()?;

    tracing::info!("Security configuration applied (no_new_privs + caps + seccomp)");
    Ok(())
}
