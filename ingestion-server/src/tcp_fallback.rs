use tokio::net::TcpListener;
use tokio::io::AsyncReadExt;

use crate::pipeline::Pipeline;

const LISTEN_ADDR: &str = "127.0.0.1:8400";
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

pub async fn run_server() -> crate::error::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR).await
        .map_err(|e| crate::error::IngestError::BindFailed(format!("{}: {}", LISTEN_ADDR, e)))?;

    tracing::info!("[tcp-fallback] Ingestion server listening on {}", LISTEN_ADDR);

    loop {
        let (mut socket, peer) = listener.accept().await?;
        tracing::debug!("[tcp-fallback] Accepted connection from {}", peer);

        tokio::spawn(async move {
            let mut pipeline = Pipeline::new(MAX_FRAME_SIZE);
            let mut buf = vec![0u8; 65536];

            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Err(e) = pipeline.ingest(&buf[..n]) {
                            tracing::warn!("[tcp-fallback] Ingest error from {}: {}", peer, e);
                            break;
                        }
                    }
                    Err(e) => {
                        tracing::error!("[tcp-fallback] Read error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
