fn main() {
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        #[cfg(target_os = "linux")]
        {
            tracing::info!("Starting sync server with QUIC/WebTransport backend (Linux native)");
            if let Err(e) = sync_server::quic::run_server().await {
                tracing::error!("QUIC server error: {}", e);
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            tracing::info!("Starting sync server with WebSocket fallback (non-Linux platform)");
            if let Err(e) = sync_server::ws_fallback::run_server().await {
                tracing::error!("WebSocket fallback error: {}", e);
            }
        }
    });
}
