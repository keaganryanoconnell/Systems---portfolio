use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeTelemetry {
    pub node_id: u32,
    pub role: String,
    pub status: String,
    pub cpu_percent: f32,
    pub memory_allocated_mb: u64,
    pub memory_total_mb: u64,
    pub active_fds: u32,
    pub replication_lag_ms: u32,
    pub storage_bytes: u64,
    pub iops: u32,
    pub uptime_secs: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricSnapshot {
    pub timestamp_ms: u64,
    pub nodes: Vec<NodeTelemetry>,
    pub total_requests: u64,
    pub requests_per_second: f64,
    pub p50_latency_ms: f64,
    pub p95_latency_ms: f64,
    pub p99_latency_ms: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryQuery {
    pub node_ids: Option<Vec<u32>>,
    pub since_ms: Option<u64>,
}
