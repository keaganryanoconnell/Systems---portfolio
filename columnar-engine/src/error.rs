use std::fmt;

#[derive(Debug)]
pub enum EngineError {
    InvalidMagic([u8; 4]),
    InvalidOffset { field: &'static str, offset: u32, total: usize },
    InsufficientData { expected: usize, actual: usize },
    BufferTooSmall { needed: usize, capacity: usize },
    MemoryLimitExceeded { current: usize, max: usize },
    ColumnOverflow { column: &'static str },
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidMagic(b) => write!(f, "Invalid magic bytes: {:?}", b),
            Self::InvalidOffset { field, offset, total } => write!(f, "Invalid offset for {}: {} exceeds total {}", field, offset, total),
            Self::InsufficientData { expected, actual } => write!(f, "Insufficient data: need {} bytes, got {}", expected, actual),
            Self::BufferTooSmall { needed, capacity } => write!(f, "Output buffer too small: need {} slots, have {}", needed, capacity),
            Self::MemoryLimitExceeded { current, max } => write!(f, "Memory limit exceeded: {} / {} bytes", current, max),
            Self::ColumnOverflow { column } => write!(f, "Column overflow: {}", column),
        }
    }
}

impl std::error::Error for EngineError {}

pub type EngineResult<T> = std::result::Result<T, EngineError>;
