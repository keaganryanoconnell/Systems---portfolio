use std::fmt;

#[derive(Debug)]
pub enum FusionBufferError {
    BufferFull,
    SlotContention(u64),
    OverwriteDetected { dropped: u64, total: u64 },
    AffinityFailed(String),
}

impl fmt::Display for FusionBufferError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BufferFull => write!(f, "Ring buffer full — all slots occupied by unread data"),
            Self::SlotContention(seq) => write!(f, "CAS contention on slot claim at sequence {}", seq),
            Self::OverwriteDetected { dropped, total } => write!(f, "Overwrite: {}/{} frames dropped", dropped, total),
            Self::AffinityFailed(msg) => write!(f, "CPU affinity failed: {}", msg),
        }
    }
}

impl std::error::Error for FusionBufferError {}

impl From<std::io::Error> for FusionBufferError {
    fn from(_e: std::io::Error) -> Self {
        Self::AffinityFailed("I/O error during affinity setup".into())
    }
}

pub type Result<T> = std::result::Result<T, FusionBufferError>;
