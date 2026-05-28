use std::fmt;
use std::io;

#[derive(Debug)]
pub enum BrokerError {
    Io(io::Error),
    InvalidFrame(String),
    CorruptData(String),
    TopicNotFound(u32),
    OffsetOutOfRange {
        requested: u64,
        earliest: u64,
        latest: u64,
    },
    ConnectionClosed,
    BufferFull,
    InvalidArgument(String),
    AlreadyExists(String),
    NotFound(String),
}

impl fmt::Display for BrokerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BrokerError::Io(e) => write!(f, "I/O error: {}", e),
            BrokerError::InvalidFrame(msg) => write!(f, "Invalid frame: {}", msg),
            BrokerError::CorruptData(msg) => write!(f, "Corrupt data: {}", msg),
            BrokerError::TopicNotFound(id) => write!(f, "Topic not found: id={}", id),
            BrokerError::OffsetOutOfRange {
                requested,
                earliest,
                latest,
            } => {
                write!(
                    f,
                    "Offset out of range: requested={}, earliest={}, latest={}",
                    requested, earliest, latest
                )
            }
            BrokerError::ConnectionClosed => write!(f, "Connection closed"),
            BrokerError::BufferFull => write!(f, "Ring buffer full"),
            BrokerError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            BrokerError::AlreadyExists(msg) => write!(f, "Already exists: {}", msg),
            BrokerError::NotFound(msg) => write!(f, "Not found: {}", msg),
        }
    }
}

impl std::error::Error for BrokerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            BrokerError::Io(e) => Some(e),
            _ => None,
        }
    }
}

impl From<io::Error> for BrokerError {
    fn from(e: io::Error) -> Self {
        BrokerError::Io(e)
    }
}

pub type BrokerResult<T> = Result<T, BrokerError>;
