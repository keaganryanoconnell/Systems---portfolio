use axum::{extract::Path, Json};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SubmitJobRequest {
    pub name: String,
    pub payload_type: String,
    pub data_count: u64,
    pub partitions: u32,
}

#[derive(Debug, Serialize)]
pub struct JobResponse {
    pub job_id: u64,
    pub status: String,
    pub partitions: u32,
}

pub async fn submit_job(Json(req): Json<SubmitJobRequest>) -> Json<JobResponse> {
    Json(JobResponse {
        job_id: 1,
        status: "accepted — orchestrator backend not yet connected".into(),
        partitions: req.partitions,
    })
}

pub async fn job_status(Path(id): Path<u64>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "job_id": id,
        "status": "pending — orchestrator backend not yet connected"
    }))
}
