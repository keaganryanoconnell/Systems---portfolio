//! Container kill orchestration.
//! The kill logic is in runtime::kill().
//! This module exists for modularity.

use crate::util::errors::ContainerResult;

/// Validate and prepare for killing.
pub fn prepare_kill() -> ContainerResult<()> {
    Ok(())
}
