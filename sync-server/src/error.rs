use std::fmt;

#[derive(Debug)]
pub enum SyncError {
    BindFailed(String),
    TlsConfig(String),
    Io(std::io::Error),
    SessionNotFound(uuid::Uuid),
    DeltaRejected(String),
    Shutdown,
}

impl fmt::Display for SyncError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BindFailed(msg) => write!(f, "Bind failed: {}", msg),
            Self::TlsConfig(msg) => write!(f, "TLS config: {}", msg),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::SessionNotFound(id) => write!(f, "Session not found: {}", id),
            Self::DeltaRejected(msg) => write!(f, "Delta rejected: {}", msg),
            Self::Shutdown => write!(f, "Server shutdown"),
        }
    }
}

impl std::error::Error for SyncError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for SyncError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, SyncError>;
