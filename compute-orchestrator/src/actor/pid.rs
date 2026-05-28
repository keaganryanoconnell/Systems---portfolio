use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProcessId {
    pub node_id: u32,
    pub actor_id: u64,
}

impl ProcessId {
    pub fn new(node_id: u32, actor_id: u64) -> Self {
        Self { node_id, actor_id }
    }
}

impl std::fmt::Display for ProcessId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.node_id, self.actor_id)
    }
}
