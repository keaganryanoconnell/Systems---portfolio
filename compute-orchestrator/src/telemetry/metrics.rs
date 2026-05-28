use std::sync::atomic::{AtomicU64, Ordering};

pub struct OrchestratorMetrics {
    pub tasks_dispatched: AtomicU64,
    pub tasks_completed: AtomicU64,
    pub tasks_failed: AtomicU64,
    pub messages_sent: AtomicU64,
    pub messages_received: AtomicU64,
    pub actors_spawned: AtomicU64,
}

impl OrchestratorMetrics {
    pub fn new() -> Self {
        Self {
            tasks_dispatched: AtomicU64::new(0),
            tasks_completed: AtomicU64::new(0),
            tasks_failed: AtomicU64::new(0),
            messages_sent: AtomicU64::new(0),
            messages_received: AtomicU64::new(0),
            actors_spawned: AtomicU64::new(0),
        }
    }

    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            tasks_dispatched: self.tasks_dispatched.load(Ordering::Relaxed),
            tasks_completed: self.tasks_completed.load(Ordering::Relaxed),
            tasks_failed: self.tasks_failed.load(Ordering::Relaxed),
            messages_sent: self.messages_sent.load(Ordering::Relaxed),
            messages_received: self.messages_received.load(Ordering::Relaxed),
            actors_spawned: self.actors_spawned.load(Ordering::Relaxed),
        }
    }
}

impl Default for OrchestratorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MetricsSnapshot {
    pub tasks_dispatched: u64,
    pub tasks_completed: u64,
    pub tasks_failed: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub actors_spawned: u64,
}
