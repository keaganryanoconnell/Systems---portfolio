use axum::{http::StatusCode, Json};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct SqlQueryRequest {
    pub query: String,
}

pub async fn sql_query(
    Json(req): Json<SqlQueryRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = serde_json::json!({
        "columns": [],
        "rows": [],
        "affected_rows": 0,
        "plan": "passthrough — sql-engine backend not yet connected",
        "query": req.query,
    });

    Ok(Json(result))
}
