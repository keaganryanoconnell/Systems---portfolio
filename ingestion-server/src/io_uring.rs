use std::net::SocketAddr;

use crate::pipeline::Pipeline;

const LISTEN_ADDR: &str = "127.0.0.1:8400";
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

pub async fn run_server() -> crate::error::Result<()> {
    let addr: SocketAddr = LISTEN_ADDR.parse().map_err(|e| {
        crate::error::IngestError::BindFailed(format!("invalid addr {}: {}", LISTEN_ADDR, e))
    })?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("[io_uring] Server bound to {}", LISTEN_ADDR);

    let mut pipeline = Pipeline::new(MAX_FRAME_SIZE);

    loop {
        let (mut stream, peer) = listener.accept().await?;
        tracing::debug!("[io_uring] Accepted connection from {}", peer);

        let mut buf = vec![0u8; 65536];
        loop {
            use tokio::io::AsyncReadExt;
            match stream.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    pipeline.ingest(&buf[..n])?;
                }
                Err(e) => {
                    tracing::error!("[io_uring] Read error from {}: {}", peer, e);
                    break;
                }
            }
        }
    }
}
