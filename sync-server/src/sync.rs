use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncDelta {
    pub peer_id: String,
    pub clock: u64,
    pub add_set: Vec<String>,
    pub remove_set: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMessage {
    pub msg_type: SyncMsgType,
    pub deltas: Vec<SyncDelta>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncMsgType {
    Delta,
    FullStateRequest,
    FullStateResponse,
    Heartbeat,
    Ack(u64),
}

pub struct SyncEngine {
    pub deltas_processed: u64,
    pub merges_rejected: u64,
}

impl SyncEngine {
    pub fn new() -> Self {
        Self {
            deltas_processed: 0,
            merges_rejected: 0,
        }
    }

    pub fn process_delta(&mut self, _delta: &SyncDelta) -> crate::error::Result<()> {
        self.deltas_processed += 1;
        Ok(())
    }

    pub fn stats(&self) -> (u64, u64) {
        (self.deltas_processed, self.merges_rejected)
    }
}

impl Default for SyncEngine {
    fn default() -> Self {
        Self::new()
    }
}
