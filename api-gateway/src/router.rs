use std::net::SocketAddr;

use axum::{routing::get, routing::post, Router};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::handlers::{cluster, compute, health, metrics, sql};

pub async fn run(bind_addr: SocketAddr) -> Result<(), Box<dyn std::error::Error>> {
    let cors = CorsLayer::permissive();
    let trace = TraceLayer::new_for_http();

    let app = Router::new()
        .route("/health", get(health::health_check))
        .route("/ready", get(health::ready_check))
        .route("/v1/sql/query", post(sql::sql_query))
        .route("/v1/jobs", post(compute::submit_job))
        .route("/v1/jobs/{id}", get(compute::job_status))
        .route("/v1/cluster/nodes", get(cluster::cluster_nodes))
        .route("/v1/cluster/health", get(cluster::cluster_health))
        .route("/v1/metrics", get(metrics::metrics))
        .layer(cors)
        .layer(trace);

    let listener = tokio::net::TcpListener::bind(bind_addr).await?;
    info!("API Gateway listening on {}", bind_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
