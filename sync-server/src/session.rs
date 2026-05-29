use std::collections::HashMap;
use uuid::Uuid;

pub struct PeerSession {
    pub peer_id: Uuid,
    pub connected_at: u64,
    pub last_seen: u64,
    pub deltas_sent: u64,
    pub deltas_received: u64,
}

impl PeerSession {
    pub fn new(peer_id: Uuid) -> Self {
        let now = Self::now_ms();
        Self {
            peer_id,
            connected_at: now,
            last_seen: now,
            deltas_sent: 0,
            deltas_received: 0,
        }
    }

    fn now_ms() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }
}

pub struct SessionManager {
    sessions: HashMap<Uuid, PeerSession>,
    max_sessions: usize,
}

impl SessionManager {
    pub fn new(max_sessions: usize) -> Self {
        Self {
            sessions: HashMap::with_capacity(max_sessions),
            max_sessions,
        }
    }

    pub fn register(&mut self, peer_id: Uuid) -> crate::error::Result<()> {
        if self.sessions.len() >= self.max_sessions {
            return Err(crate::error::SyncError::SessionNotFound(peer_id));
        }
        self.sessions.insert(peer_id, PeerSession::new(peer_id));
        Ok(())
    }

    pub fn remove(&mut self, peer_id: &Uuid) {
        self.sessions.remove(peer_id);
    }

    pub fn peer_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn ids(&self) -> Vec<Uuid> {
        self.sessions.keys().copied().collect()
    }
}
