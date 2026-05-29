use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MacroTask {
    pub id: u64,
    pub name: String,
    pub payload_type: String,
    pub data_range: ComputeRange,
    pub partition_count: u32,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeRange {
    pub start: u64,
    pub end: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicroTask {
    pub id: u64,
    pub macro_id: u64,
    pub partition: u32,
    pub range: ComputeRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResult {
    pub task_id: u64,
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
    pub node_id: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapacity {
    pub node_id: u32,
    pub cpu_available: f32,
    pub memory_available_mb: u64,
    pub task_queue_depth: u32,
    pub max_tasks: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClusterHealth {
    pub total_nodes: u32,
    pub healthy_nodes: u32,
    pub degraded_nodes: u32,
    pub dead_nodes: u32,
    pub active_tasks: u64,
    pub completed_tasks: u64,
}
