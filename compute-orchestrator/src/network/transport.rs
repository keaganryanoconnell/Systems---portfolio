use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio_rustls::rustls::{self, ClientConfig};
use tokio_rustls::TlsConnector;
use tracing::debug;

use crate::error::Result;

pub async fn send_message<T: Serialize>(addr: SocketAddr, msg: &T) -> Result<()> {
    let mut stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;

    write_frame(&mut stream, msg).await
}

pub async fn recv_message<T: DeserializeOwned>(stream: &mut TcpStream) -> Result<T> {
    read_frame(stream).await
}

pub async fn send_tls<T: Serialize>(
    addr: SocketAddr,
    msg: &T,
    tls_config: &Arc<rustls::ClientConfig>,
) -> Result<()> {
    let stream = TcpStream::connect(addr).await?;
    stream.set_nodelay(true)?;

    let domain = rustls::pki_types::ServerName::try_from("localhost")
        .map_err(|_| crate::error::OrchestratorError::Network("invalid server name".into()))?
        .to_owned();

    let connector = TlsConnector::from(Arc::clone(tls_config));
    let mut tls_stream = connector.connect(domain, stream).await.map_err(|e| {
        crate::error::OrchestratorError::Network(format!("TLS handshake failed: {e}"))
    })?;

    write_frame(&mut tls_stream, msg).await
}

pub async fn accept_tls(
    listener: &TcpListener,
    tls_config: &Arc<rustls::ServerConfig>,
) -> Result<(tokio_rustls::server::TlsStream<TcpStream>, SocketAddr)> {
    let (stream, addr) = listener.accept().await?;
    stream.set_nodelay(true)?;

    let acceptor = tokio_rustls::TlsAcceptor::from(Arc::clone(tls_config));
    let tls_stream = acceptor
        .accept(stream)
        .await
        .map_err(|e| crate::error::OrchestratorError::Network(format!("TLS accept failed: {e}")))?;

    Ok((tls_stream, addr))
}

pub fn serialize_to_vec<T: Serialize>(msg: &T) -> Result<Vec<u8>> {
    Ok(bincode::serialize(msg)?)
}

pub fn deserialize_from_slice<T: DeserializeOwned>(data: &[u8]) -> Result<T> {
    Ok(bincode::deserialize(data)?)
}

pub fn make_tls_client_config(ca_cert_pem: &str) -> Result<Arc<ClientConfig>> {
    let mut root_store = rustls::RootCertStore::empty();
    let certs = rustls_pemfile::certs(&mut ca_cert_pem.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| crate::error::OrchestratorError::Network(format!("invalid PEM certs: {e}")))?;

    for cert in certs {
        root_store.add(cert).map_err(|e| {
            crate::error::OrchestratorError::Network(format!("invalid certificate: {e}"))
        })?;
    }

    let config = ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(Arc::new(config))
}

pub fn make_tls_server_config(cert_pem: &str, key_pem: &str) -> Result<Arc<rustls::ServerConfig>> {
    let certs = rustls_pemfile::certs(&mut cert_pem.as_bytes())
        .collect::<std::result::Result<Vec<_>, _>>()
        .map_err(|e| crate::error::OrchestratorError::Network(format!("invalid PEM certs: {e}")))?;

    let key = rustls_pemfile::private_key(&mut key_pem.as_bytes())
        .map_err(|e| crate::error::OrchestratorError::Network(format!("invalid PEM key: {e}")))?
        .ok_or_else(|| crate::error::OrchestratorError::Network("missing private key".into()))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| crate::error::OrchestratorError::Network(format!("TLS server config: {e}")))?;

    Ok(Arc::new(config))
}

async fn write_frame<W: AsyncWriteExt + Unpin, T: Serialize>(
    writer: &mut W,
    msg: &T,
) -> Result<()> {
    let encoded = bincode::serialize(msg)?;
    let len = encoded.len() as u32;
    let mut frame = Vec::with_capacity(4 + encoded.len());
    frame.extend_from_slice(&len.to_be_bytes());
    frame.extend_from_slice(&encoded);

    writer.write_all(&frame).await?;
    writer.flush().await?;

    debug!("Sent {} bytes", frame.len());
    Ok(())
}

async fn read_frame<T: DeserializeOwned>(stream: &mut TcpStream) -> Result<T> {
    let mut len_buf = [0u8; 4];
    tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut len_buf))
        .await
        .map_err(|_| {
            crate::error::OrchestratorError::Network("read timeout waiting for length".into())
        })??;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 1024 * 1024 {
        return Err(crate::error::OrchestratorError::Network(format!(
            "message too large: {} bytes",
            len
        )));
    }

    let mut buf = vec![0u8; len];
    tokio::time::timeout(Duration::from_secs(30), stream.read_exact(&mut buf))
        .await
        .map_err(|_| {
            crate::error::OrchestratorError::Network("read timeout waiting for body".into())
        })??;

    let msg = bincode::deserialize(&buf)?;
    Ok(msg)
}
