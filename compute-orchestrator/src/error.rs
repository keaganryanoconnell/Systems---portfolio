use std::fmt;
use std::io;

#[derive(Debug)]
pub enum OrchestratorError {
    Io(io::Error),
    Serialization(String),
    Network(String),
    ActorNotFound(u64),
    TaskFailed(String),
    GossipTimeout(String),
    InvalidState(String),
    InvalidArgument(String),
}

impl fmt::Display for OrchestratorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::Network(msg) => write!(f, "Network error: {}", msg),
            Self::ActorNotFound(id) => write!(f, "Actor not found: id={}", id),
            Self::TaskFailed(msg) => write!(f, "Task failed: {}", msg),
            Self::GossipTimeout(msg) => write!(f, "Gossip timeout: {}", msg),
            Self::InvalidState(msg) => write!(f, "Invalid state: {}", msg),
            Self::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
        }
    }
}

impl std::error::Error for OrchestratorError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for OrchestratorError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<bincode::Error> for OrchestratorError {
    fn from(e: bincode::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, OrchestratorError>;
