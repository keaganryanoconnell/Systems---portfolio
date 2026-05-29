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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_register_and_count() {
        let mut mgr = SessionManager::new(10);
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();

        assert!(mgr.register(id1).is_ok());
        assert_eq!(mgr.peer_count(), 1);

        assert!(mgr.register(id2).is_ok());
        assert_eq!(mgr.peer_count(), 2);
    }

    #[test]
    fn test_session_manager_max_capacity() {
        let mut mgr = SessionManager::new(2);
        assert!(mgr.register(Uuid::new_v4()).is_ok());
        assert!(mgr.register(Uuid::new_v4()).is_ok());
        assert!(mgr.register(Uuid::new_v4()).is_err());
        assert_eq!(mgr.peer_count(), 2);
    }

    #[test]
    fn test_session_manager_remove() {
        let mut mgr = SessionManager::new(10);
        let id = Uuid::new_v4();
        mgr.register(id).unwrap();
        assert_eq!(mgr.peer_count(), 1);
        mgr.remove(&id);
        assert_eq!(mgr.peer_count(), 0);
    }

    #[test]
    fn test_session_manager_ids() {
        let mut mgr = SessionManager::new(10);
        let id1 = Uuid::new_v4();
        let id2 = Uuid::new_v4();
        mgr.register(id1).unwrap();
        mgr.register(id2).unwrap();

        let ids = mgr.ids();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[test]
    fn test_peer_session_initial_state() {
        let id = Uuid::new_v4();
        let session = PeerSession::new(id);
        assert_eq!(session.peer_id, id);
        assert_eq!(session.deltas_sent, 0);
        assert_eq!(session.deltas_received, 0);
        assert!(session.connected_at > 0);
    }
}
