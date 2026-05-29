use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use uuid::Uuid;

use crate::session::SessionManager;
use crate::sync::SyncEngine;

const LISTEN_ADDR: &str = "127.0.0.1:9400";

pub async fn run_server() -> crate::error::Result<()> {
    let listener = TcpListener::bind(LISTEN_ADDR)
        .await
        .map_err(|e| crate::error::SyncError::BindFailed(format!("{}: {}", LISTEN_ADDR, e)))?;

    tracing::info!("[ws-fallback] Sync server listening on {}", LISTEN_ADDR);

    let mut sessions = SessionManager::new(256);
    let _engine = SyncEngine::new();

    loop {
        let (mut socket, peer) = listener.accept().await?;
        tracing::debug!("[ws-fallback] Accepted connection from {}", peer);

        let peer_id = Uuid::new_v4();
        sessions.register(peer_id)?;

        tokio::spawn(async move {
            let mut buf = vec![0u8; 65536];
            loop {
                match socket.read(&mut buf).await {
                    Ok(0) => break,
                    Ok(n) => {
                        if let Ok(_msg) = serde_json::from_slice::<serde_json::Value>(&buf[..n]) {
                            let ack =
                                serde_json::json!({"type": "ack", "peer": peer_id.to_string()});
                            let ack_bytes = serde_json::to_vec(&ack).unwrap_or_default();
                            let _ = socket.write_all(&ack_bytes).await;
                            tracing::debug!(
                                "[ws-fallback] Processed {}B message from {}",
                                n,
                                peer_id
                            );
                        }
                    }
                    Err(e) => {
                        tracing::error!("[ws-fallback] Read error: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
