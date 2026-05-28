use std::net::SocketAddr;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tracing::debug;

use crate::error::Result;

pub async fn send_message<T: Serialize>(addr: SocketAddr, msg: &T) -> Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;

    let encoded = bincode::serialize(msg)?;
    let len = encoded.len() as u32;
    let mut frame = Vec::with_capacity(4 + encoded.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&encoded);

    stream.write_all(&frame).await?;
    stream.flush().await?;

    debug!("Sent {} bytes to {}", frame.len(), addr);
    Ok(())
}

pub async fn recv_message<T: DeserializeOwned>(stream: &mut TcpStream) -> Result<T> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 16 * 1024 * 1024 {
        return Err(crate::error::OrchestratorError::Network(format!(
            "message too large: {} bytes",
            len
        )));
    }

    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf).await?;

    let msg = bincode::deserialize(&buf)?;
    Ok(msg)
}

pub fn serialize_to_vec<T: Serialize>(msg: &T) -> Result<Vec<u8>> {
    Ok(bincode::serialize(msg)?)
}

pub fn deserialize_from_slice<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
    Ok(bincode::deserialize(data)?)
}
