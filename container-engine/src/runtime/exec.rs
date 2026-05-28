//! Container exec orchestration.
//! The exec logic is in runtime::exec() which uses nsenter.
//! This module exists for modularity.

use crate::util::errors::ContainerResult;

/// Validate and prepare for exec.
pub fn prepare_exec() -> ContainerResult<()> {
    Ok(())
}
