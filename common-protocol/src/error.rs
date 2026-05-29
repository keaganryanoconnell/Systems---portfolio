use std::fmt;

#[derive(Debug)]
pub enum ProtocolError {
    Serialization(String),
    InvalidFrame(String),
    UnknownMessageType(u32),
    PayloadTooLarge(usize),
    ConnectionClosed,
}

impl fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            Self::InvalidFrame(msg) => write!(f, "Invalid frame: {}", msg),
            Self::UnknownMessageType(id) => write!(f, "Unknown message type: {}", id),
            Self::PayloadTooLarge(size) => write!(f, "Payload too large: {} bytes", size),
            Self::ConnectionClosed => write!(f, "Connection closed"),
        }
    }
}

impl std::error::Error for ProtocolError {}

impl From<bincode::Error> for ProtocolError {
    fn from(e: bincode::Error) -> Self {
        Self::Serialization(e.to_string())
    }
}

pub type ProtocolResult<T> = std::result::Result<T, ProtocolError>;
