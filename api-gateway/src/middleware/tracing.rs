use axum::{body::Body, http::Request, middleware::Next, response::Response};
use uuid::Uuid;

pub async fn trace_middleware(request: Request<Body>, next: Next) -> Response {
    let trace_id = request
        .headers()
        .get("x-trace-id")
        .and_then(|v| v.to_str().ok())
        .and_then(|s| Uuid::parse_str(s).ok())
        .map(|u| u.as_u128())
        .unwrap_or_else(|| Uuid::new_v4().as_u128());

    let mut response = next.run(request).await;
    if let Ok(val) = axum::http::HeaderValue::from_str(&Uuid::from_u128(trace_id).to_string()) {
        response.headers_mut().insert("x-trace-id", val);
    }
    response
}
