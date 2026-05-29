use std::net::SocketAddr;

use crate::session::SessionManager;
use crate::sync::SyncEngine;

const LISTEN_ADDR: &str = "127.0.0.1:9400";

pub async fn run_server() -> crate::error::Result<()> {
    let addr: SocketAddr = LISTEN_ADDR
        .parse()
        .map_err(|e| crate::error::SyncError::BindFailed(format!("invalid addr: {}", e)))?;

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!("[quic] QUIC sync server bound to {}", LISTEN_ADDR);

    let mut sessions = SessionManager::new(256);
    let mut engine = SyncEngine::new();

    loop {
        let (mut stream, peer) = listener.accept().await?;
        tracing::debug!("[quic] Accepted connection from {}", peer);

        let mut buf = vec![0u8; 65536];
        loop {
            use tokio::io::AsyncReadExt;
            match stream.read(&mut buf).await {
                Ok(0) => break,
                Ok(n) => {
                    tracing::debug!("[quic] Received {}B from {}", n, peer);
                }
                Err(e) => {
                    tracing::error!("[quic] Read error: {}", e);
                    break;
                }
            }
        }
    }
}
