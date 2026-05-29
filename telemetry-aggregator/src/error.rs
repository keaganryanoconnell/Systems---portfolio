use std::fmt;

#[derive(Debug)]
pub enum AggregatorError {
    Io(std::io::Error),
    BufferFull,
    InvalidPacket(String),
    CompressionFailed(String),
}

impl fmt::Display for AggregatorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {}", e),
            Self::BufferFull => write!(f, "Buffer full"),
            Self::InvalidPacket(msg) => write!(f, "Invalid packet: {}", msg),
            Self::CompressionFailed(msg) => write!(f, "Compression failed: {}", msg),
        }
    }
}

impl std::error::Error for AggregatorError {}

impl From<std::io::Error> for AggregatorError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

pub type Result<T> = std::result::Result<T, AggregatorError>;
