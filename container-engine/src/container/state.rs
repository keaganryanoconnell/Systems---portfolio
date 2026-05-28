use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContainerState {
    Created,
    Running,
    Paused,
    Stopped,
}

impl ContainerState {
    pub fn valid_transition(self, target: ContainerState) -> bool {
        use ContainerState::*;
        matches!(
            (self, target),
            (Created, Running)
                | (Running, Paused)
                | (Running, Stopped)
                | (Paused, Running)
                | (Paused, Stopped)
                | (Stopped, Created)
        )
    }

    pub fn is_active(self) -> bool {
        matches!(self, ContainerState::Running | ContainerState::Paused)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            ContainerState::Created => "created",
            ContainerState::Running => "running",
            ContainerState::Paused => "paused",
            ContainerState::Stopped => "stopped",
        }
    }
}

impl std::fmt::Display for ContainerState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}
