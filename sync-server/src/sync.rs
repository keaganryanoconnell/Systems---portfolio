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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sync_engine_new() {
        let engine = SyncEngine::new();
        assert_eq!(engine.deltas_processed, 0);
        assert_eq!(engine.merges_rejected, 0);
    }

    #[test]
    fn test_process_delta_increments_counter() {
        let mut engine = SyncEngine::new();
        let delta = SyncDelta {
            peer_id: "peer-1".into(),
            clock: 1,
            add_set: vec!["key1".into()],
            remove_set: vec![],
        };
        engine.process_delta(&delta).unwrap();
        assert_eq!(engine.deltas_processed, 1);
    }

    #[test]
    fn test_multiple_deltas() {
        let mut engine = SyncEngine::new();
        for i in 0..5 {
            let delta = SyncDelta {
                peer_id: format!("peer-{}", i),
                clock: i,
                add_set: vec![],
                remove_set: vec![],
            };
            engine.process_delta(&delta).unwrap();
        }
        let (processed, rejected) = engine.stats();
        assert_eq!(processed, 5);
        assert_eq!(rejected, 0);
    }

    #[test]
    fn test_sync_delta_serialization_roundtrip() {
        let delta = SyncDelta {
            peer_id: "peer-alpha".into(),
            clock: 42,
            add_set: vec!["a".into(), "b".into()],
            remove_set: vec!["c".into()],
        };
        let json = serde_json::to_string(&delta).unwrap();
        let parsed: SyncDelta = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.peer_id, "peer-alpha");
        assert_eq!(parsed.clock, 42);
        assert_eq!(parsed.add_set.len(), 2);
        assert_eq!(parsed.remove_set.len(), 1);
    }

    #[test]
    fn test_sync_message_serialization() {
        let msg = SyncMessage {
            msg_type: SyncMsgType::Heartbeat,
            deltas: vec![],
        };
        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("Heartbeat"));
    }

    #[test]
    fn test_sync_engine_default() {
        let engine = SyncEngine::default();
        assert_eq!(engine.deltas_processed, 0);
        assert_eq!(engine.merges_rejected, 0);
    }
}
