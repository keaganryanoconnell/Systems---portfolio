use std::net::SocketAddr;
use tracing::info;
use tracing_subscriber::FmtSubscriber;

#[tokio::main]
async fn main() {
    let subscriber = FmtSubscriber::builder()
        .with_env_filter("api_gateway=info,tower_http=info")
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber).ok();

    let bind_addr: SocketAddr = "0.0.0.0:8080".parse().expect("invalid bind address");
    info!("Starting API Gateway on {}", bind_addr);
    api_gateway::router::run(bind_addr).await;
}
