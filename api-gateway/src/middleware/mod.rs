pub mod tracing;

use axum::http::HeaderMap;
use uuid::Uuid;

pub fn extract_trace_id(headers: &HeaderMap) -> u128 {
    headers
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(|u| u.as_u128())
        .unwrap_or_else(|| common_protocol::new_trace_id())
}
