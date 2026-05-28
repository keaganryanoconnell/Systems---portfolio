use std::path::PathBuf;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ContainerError {
    #[error("Namespace isolation failed: {context}")]
    NamespaceError { source: nix::Error, context: String },

    #[error("Filesystem isolation failed at step '{step}': {detail}")]
    FilesystemError {
        step: &'static str,
        detail: String,
        source: Option<nix::Error>,
    },

    #[error("Cgroup controller '{controller}' failed: {detail}")]
    CgroupError {
        controller: &'static str,
        detail: String,
        source: Option<std::io::Error>,
    },

    #[error("Seccomp filter installation failed: {0}")]
    SeccompError(#[source] nix::Error),

    #[error("Capability '{cap}' drop failed: {detail}")]
    CapabilityError { cap: String, detail: String },

    #[error("Network setup failed at step '{step}': {detail}")]
    NetworkError { step: &'static str, detail: String },

    #[error("Container state transition '{from:?}' -> '{to:?}' is invalid")]
    StateTransitionError {
        from: super::container::state::ContainerState,
        to: super::container::state::ContainerState,
    },

    #[error("Container {id}: {message}")]
    ContainerNotFound { id: String, message: String },

    #[error("Resource leak detected: {resource} - cleanup required")]
    LeakError { resource: String },

    #[error("{0}")]
    Io(#[from] std::io::Error),

    #[error("Container ID '{0}' already exists")]
    IdConflict(String),

    #[error("Operation not supported on this platform: {0}")]
    UnsupportedPlatform(&'static str),

    #[error("Privilege check failed: {0}")]
    PrivilegeError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("{0}")]
    Internal(String),
}

impl From<nix::Error> for ContainerError {
    fn from(e: nix::Error) -> Self {
        ContainerError::Internal(format!("nix error: {e}"))
    }
}

impl From<serde_json::Error> for ContainerError {
    fn from(e: serde_json::Error) -> Self {
        ContainerError::Internal(format!("JSON error: {e}"))
    }
}

pub type ContainerResult<T> = Result<T, ContainerError>;

pub trait ContextExt<T> {
    fn context(self, msg: &str) -> ContainerResult<T>;
}

impl<T, E: std::fmt::Display> ContextExt<T> for Result<T, E> {
    fn context(self, msg: &str) -> ContainerResult<T> {
        self.map_err(|e| ContainerError::Internal(format!("{msg}: {e}")))
    }
}
