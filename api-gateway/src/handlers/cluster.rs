use axum::Json;
use serde::Serialize;

#[derive(Serialize)]
pub struct ClusterNode {
    node_id: u32,
    role: String,
    status: String,
    cpu: f32,
    memory_mb: u64,
}

pub async fn cluster_nodes() -> Json<Vec<ClusterNode>> {
    Json(vec![ClusterNode {
        node_id: 1,
        role: "Leader".into(),
        status: "Healthy".into(),
        cpu: 12.5,
        memory_mb: 512,
    }])
}

pub async fn cluster_health() -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "total_nodes": 3,
        "healthy": 3,
        "degraded": 0,
        "dead": 0,
        "backend": "mock — compute-orchestrator not yet connected"
    }))
}
