//! Container start orchestration.
//! The full start logic is in runtime::run() which calls isolate::isolate().
//! This module exists for modularity and future extensibility.

use crate::container::config::ContainerData;
use crate::util::errors::ContainerResult;

/// Start a previously created container.
pub fn start_container(_data: &ContainerData) -> ContainerResult<()> {
    // Starting is handled inline in runtime::run
    Ok(())
}
