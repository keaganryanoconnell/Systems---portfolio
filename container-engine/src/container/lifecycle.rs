use nix::unistd::Pid;

use super::config::ContainerConfig;
use super::state::ContainerState;
use super::state_file;
use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

/// Create a new container from configuration, persisting its state to disk.
/// This does NOT start the container — it only validates the config and
/// writes the initial CREATED state.
pub fn create(config: ContainerConfig) -> ContainerResult<ContainerData> {
    let data = state_file::new_container_data(config);
    state_file::save_state(&data)?;
    tracing::info!(id = %data.id, state = %data.state, "Container created");
    Ok(data)
}

/// Transition container state, validating the state machine.
pub fn transition(
    id: &ContainerId,
    target: ContainerState,
    pid: Option<Pid>,
    exit_code: Option<i32>,
) -> ContainerResult<ContainerData> {
    state_file::transition_state(id, target, pid, exit_code)
}

/// Load container data from disk.
pub fn load(id: &ContainerId) -> ContainerResult<ContainerData> {
    state_file::load_state(id)
}

/// List all containers (optionally including stopped).
pub fn list_all() -> ContainerResult<Vec<ContainerData>> {
    state_file::list_containers()
}

/// Remove a container's state from disk.
pub fn remove_state(id: &ContainerId) -> ContainerResult<()> {
    state_file::delete_state(id)
}

use super::state_file::ContainerData;
