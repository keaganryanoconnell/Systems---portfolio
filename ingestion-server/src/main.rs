fn main() {
    tracing_subscriber::fmt::init();

    let rt = tokio::runtime::Runtime::new().expect("Failed to create tokio runtime");

    rt.block_on(async {
        #[cfg(target_os = "linux")]
        {
            tracing::info!("Starting ingestion server with io_uring backend (Linux native)");
            if let Err(e) = ingestion_server::io_uring::run_server().await {
                tracing::error!("io_uring server error: {}", e);
            }
        }

        #[cfg(not(target_os = "linux"))]
        {
            tracing::info!("Starting ingestion server with TCP fallback (non-Linux platform)");
            if let Err(e) = ingestion_server::tcp_fallback::run_server().await {
                tracing::error!("TCP fallback server error: {}", e);
            }
        }
    });
}
