use axum::{http::StatusCode, response::IntoResponse};

pub async fn metrics() -> impl IntoResponse {
    let body = "# HELP api_gateway_requests_total Total requests\n\
         # TYPE api_gateway_requests_total counter\n\
         api_gateway_requests_total{endpoint=\"health\"} 0\n\
         # HELP api_gateway_uptime_seconds Gateway uptime\n\
         # TYPE api_gateway_uptime_seconds gauge\n\
         api_gateway_uptime_seconds 0\n";

    (StatusCode::OK, body)
}
