use std::fmt;

#[derive(Debug)]
pub enum IngestError {
    BindFailed(String),
    Io(std::io::Error),
    ParseFailed(String),
    BufferFull,
    Shutdown,
}

impl fmt::Display for IngestError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BindFailed(msg) => write!(f, "Bind failed: {}", msg),
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::ParseFailed(msg) => write!(f, "Parse failed: {}", msg),
            Self::BufferFull => write!(f, "Ingest buffer full"),
            Self::Shutdown => write!(f, "Server shutdown"),
        }
    }
}

impl std::error::Error for IngestError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for IngestError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, IngestError>;
