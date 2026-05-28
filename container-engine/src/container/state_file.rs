use std::path::Path;

use nix::unistd::Pid;
use serde::{Deserialize, Serialize};

use super::config::ContainerConfig;
use super::state::ContainerState;
use crate::util::errors::{ContainerError, ContainerResult};
use crate::util::id::ContainerId;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerData {
    pub id: ContainerId,
    pub state: ContainerState,
    pub pid: Option<i32>,
    pub config: ContainerConfig,
    pub created_at: String,
    pub exited_at: Option<String>,
    pub exit_code: Option<i32>,
}

const STATE_DIR: &str = "/var/run/container-engine";

fn state_dir() -> std::path::PathBuf {
    Path::new(STATE_DIR).to_path_buf()
}

fn container_dir(id: &ContainerId) -> std::path::PathBuf {
    state_dir().join(id.as_str())
}

fn state_file(id: &ContainerId) -> std::path::PathBuf {
    container_dir(id).join("state.json")
}

pub fn ensure_state_dir() -> ContainerResult<()> {
    let dir = state_dir();
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| ContainerError::Io(e))?;
    }
    Ok(())
}

pub fn save_state(data: &ContainerData) -> ContainerResult<()> {
    let dir = container_dir(&data.id);
    if !dir.exists() {
        std::fs::create_dir_all(&dir).map_err(|e| ContainerError::Io(e))?;
    }
    let json = serde_json::to_string_pretty(data)?;
    std::fs::write(state_file(&data.id), json)?;
    Ok(())
}

pub fn load_state(id: &ContainerId) -> ContainerResult<ContainerData> {
    let path = state_file(id);
    if !path.exists() {
        return Err(ContainerError::ContainerNotFound {
            id: id.to_string(),
            message: format!("no state file at {}", path.display()),
        });
    }
    let json = std::fs::read_to_string(&path)?;
    let data: ContainerData = serde_json::from_str(&json)?;
    Ok(data)
}

pub fn delete_state(id: &ContainerId) -> ContainerResult<()> {
    let dir = container_dir(id);
    if dir.exists() {
        std::fs::remove_dir_all(&dir)?;
    }
    Ok(())
}

pub fn list_containers() -> ContainerResult<Vec<ContainerData>> {
    let dir = state_dir();
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut containers = Vec::new();
    let entries = std::fs::read_dir(&dir)?;
    for entry in entries {
        let entry = entry?;
        if !entry.file_type()?.is_dir() {
            continue;
        }
        let dir_name = entry.file_name();
        let dir_name_str = dir_name.to_string_lossy();
        if let Some(id) = ContainerId::from_str(&dir_name_str) {
            if let Ok(data) = load_state(&id) {
                containers.push(data);
            }
        }
    }
    containers.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(containers)
}

pub fn transition_state(
    id: &ContainerId,
    new_state: ContainerState,
    pid: Option<Pid>,
    exit_code: Option<i32>,
) -> ContainerResult<ContainerData> {
    let mut data = load_state(id)?;
    if !data.state.valid_transition(new_state) {
        return Err(ContainerError::StateTransitionError {
            from: data.state,
            to: new_state,
        });
    }
    data.state = new_state;
    data.pid = pid.map(|p| p.as_raw());
    if new_state == ContainerState::Stopped {
        data.exited_at = Some(chrono_now());
        data.exit_code = exit_code;
    }
    save_state(&data)?;
    Ok(data)
}

fn chrono_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let dur = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", dur.as_secs())
}

pub fn new_container_data(config: ContainerConfig) -> ContainerData {
    ContainerData {
        id: config.id.clone(),
        state: ContainerState::Created,
        pid: None,
        config,
        created_at: chrono_now(),
        exited_at: None,
        exit_code: None,
    }
}
