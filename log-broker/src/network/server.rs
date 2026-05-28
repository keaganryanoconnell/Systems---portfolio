use std::collections::HashMap;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use std::sync::Arc;

use mio::net::{TcpListener, TcpStream};
use mio::{Events, Interest, Poll, Token};

use parking_lot::Mutex;

use crate::error::{BrokerError, BrokerResult};
use crate::log::LogManager;
use crate::network::protocol::{
    Frame, FrameDecoder, API_COMMIT, API_FETCH, API_LIST_OFFSETS, API_PRODUCE, ERROR_INTERNAL,
    ERROR_NONE, ERROR_OFFSET_OUT_OF_RANGE,
};

const SERVER_TOKEN: Token = Token(0);
const MAX_CONNECTIONS: usize = 1024;
const CONNECTION_START_TOKEN: usize = 1;

struct Connection {
    stream: TcpStream,
    decoder: FrameDecoder,
    write_buffer: Vec<u8>,
    read_buffer: Vec<u8>,
    addr: SocketAddr,
}

pub struct BrokerServer {
    log_manager: Arc<LogManager>,
    connections: Mutex<HashMap<Token, Connection>>,
    next_token: usize,
    poll: Poll,
    listener: TcpListener,
}

impl BrokerServer {
    pub fn new(log_manager: Arc<LogManager>, bind_addr: &str) -> BrokerResult<Self> {
        let addr: SocketAddr = bind_addr.parse().map_err(|e| {
            BrokerError::InvalidArgument(format!("invalid bind address '{}': {}", bind_addr, e))
        })?;

        let mut listener = TcpListener::bind(addr).map_err(BrokerError::from)?;
        let poll = Poll::new().map_err(BrokerError::from)?;

        poll.registry()
            .register(&mut listener, SERVER_TOKEN, Interest::READABLE)
            .map_err(BrokerError::from)?;

        Ok(Self {
            log_manager,
            connections: Mutex::new(HashMap::new()),
            next_token: CONNECTION_START_TOKEN,
            poll,
            listener,
        })
    }

    pub fn run(&mut self) -> BrokerResult<()> {
        let mut events = Events::with_capacity(1024);

        loop {
            self.poll
                .poll(&mut events, None)
                .map_err(BrokerError::from)?;

            for event in events.iter() {
                match event.token() {
                    SERVER_TOKEN => {
                        if event.is_readable() {
                            self.accept_new_connections()?;
                        }
                    }
                    token => {
                        if event.is_readable() {
                            self.handle_read(token)?;
                        }
                        if event.is_writable() {
                            self.handle_write(token)?;
                        }
                    }
                }
            }
        }
    }

    fn accept_new_connections(&mut self) -> BrokerResult<()> {
        loop {
            match self.listener.accept() {
                Ok((mut stream, addr)) => {
                    if self.next_token >= MAX_CONNECTIONS + CONNECTION_START_TOKEN {
                        let _ = stream.shutdown(std::net::Shutdown::Both);
                        eprintln!("[server] max connections reached, rejecting {}", addr);
                        continue;
                    }

                    let token = Token(self.next_token);
                    self.next_token += 1;

                    if let Err(e) =
                        self.poll
                            .registry()
                            .register(&mut stream, token, Interest::READABLE)
                    {
                        eprintln!("[server] failed to register connection {}: {}", addr, e);
                        continue;
                    }

                    let conn = Connection {
                        stream,
                        decoder: FrameDecoder::new(),
                        write_buffer: Vec::new(),
                        read_buffer: vec![0u8; 65536],
                        addr,
                    };

                    self.connections.lock().insert(token, conn);
                    eprintln!("[server] connection accepted: {} (token={})", addr, token.0);
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err(BrokerError::Io(e));
                }
            }
        }

        Ok(())
    }

    fn handle_read(&mut self, token: Token) -> BrokerResult<()> {
        let remove_connection;

        {
            let mut connections = self.connections.lock();
            let conn = match connections.get_mut(&token) {
                Some(c) => c,
                None => return Ok(()),
            };

            let bytes_read = match conn.stream.read(&mut conn.read_buffer) {
                Ok(0) => {
                    eprintln!("[server] connection closed by client: {}", conn.addr);
                    remove_connection = true;
                    0
                }
                Ok(n) => {
                    remove_connection = false;
                    n
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    remove_connection = false;
                    0
                }
                Err(e) => {
                    eprintln!("[server] read error from {}: {}", conn.addr, e);
                    remove_connection = true;
                    0
                }
            };

            if remove_connection {
                drop(connections);
                self.remove_connection(token);
                return Ok(());
            }

            if bytes_read == 0 {
                return Ok(());
            }

            let frames = match conn.decoder.feed_bytes(&conn.read_buffer[..bytes_read]) {
                Ok(f) => f,
                Err(e) => {
                    eprintln!("[server] frame decode error from {}: {}", conn.addr, e);
                    drop(connections);
                    self.remove_connection(token);
                    return Ok(());
                }
            };

            for frame in frames {
                match self.process_frame(frame) {
                    Ok(response_frame) => match response_frame.encode() {
                        Ok(encoded) => {
                            conn.write_buffer.extend_from_slice(&encoded);
                        }
                        Err(e) => {
                            eprintln!("[server] encode error for {}: {}", conn.addr, e);
                        }
                    },
                    Err(e) => {
                        eprintln!("[server] frame processing error for {}: {}", conn.addr, e);
                        let error_frame = Frame::produce_response(0, ERROR_INTERNAL, 0);
                        if let Ok(encoded) = error_frame.encode() {
                            conn.write_buffer.extend_from_slice(&encoded);
                        }
                    }
                }
            }

            if !conn.write_buffer.is_empty() {
                let registry = self.poll.registry();
                let _ = registry.reregister(
                    &mut conn.stream,
                    token,
                    Interest::READABLE.add(Interest::WRITABLE),
                );
            }
        }

        Ok(())
    }

    fn handle_write(&mut self, token: Token) -> BrokerResult<()> {
        let remove_connection;

        {
            let mut connections = self.connections.lock();
            let conn = match connections.get_mut(&token) {
                Some(c) => c,
                None => return Ok(()),
            };

            if conn.write_buffer.is_empty() {
                let registry = self.poll.registry();
                let _ = registry.reregister(&mut conn.stream, token, Interest::READABLE);
                return Ok(());
            }

            match conn.stream.write(&conn.write_buffer) {
                Ok(n) => {
                    conn.write_buffer.drain(..n);
                    remove_connection = false;
                }
                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                    remove_connection = false;
                }
                Err(e) => {
                    eprintln!("[server] write error to {}: {}", conn.addr, e);
                    remove_connection = true;
                }
            }
        }

        if remove_connection {
            self.remove_connection(token);
        }

        Ok(())
    }

    fn process_frame(&self, frame: Frame) -> BrokerResult<Frame> {
        match frame.api_key {
            API_PRODUCE => {
                if frame.body.len() < 2 {
                    return Err(BrokerError::InvalidFrame("produce body too short".into()));
                }

                let topic_name_len = u16::from_be_bytes([frame.body[0], frame.body[1]]) as usize;
                if 2 + topic_name_len + 8 > frame.body.len() {
                    return Err(BrokerError::InvalidFrame(
                        "produce body size mismatch".into(),
                    ));
                }

                let topic_name = std::str::from_utf8(&frame.body[2..2 + topic_name_len])
                    .map_err(|e| BrokerError::InvalidFrame(format!("invalid topic name: {}", e)))?;

                let pos = 2 + topic_name_len;
                let key_len = u32::from_be_bytes([
                    frame.body[pos],
                    frame.body[pos + 1],
                    frame.body[pos + 2],
                    frame.body[pos + 3],
                ]) as usize;
                let value_len = u32::from_be_bytes([
                    frame.body[pos + 4],
                    frame.body[pos + 5],
                    frame.body[pos + 6],
                    frame.body[pos + 7],
                ]) as usize;

                let data_start = pos + 8;
                if data_start + key_len + value_len > frame.body.len() {
                    return Err(BrokerError::InvalidFrame(
                        "produce body size mismatch".into(),
                    ));
                }

                let key = &frame.body[data_start..data_start + key_len];
                let value = &frame.body[data_start + key_len..data_start + key_len + value_len];

                let offset = self.log_manager.append(topic_name, key, value)?;

                Ok(Frame::produce_response(
                    frame.correlation_id,
                    ERROR_NONE,
                    offset,
                ))
            }

            API_FETCH => {
                if frame.body.len() < 2 {
                    return Err(BrokerError::InvalidFrame("fetch body too short".into()));
                }

                let topic_name_len = u16::from_be_bytes([frame.body[0], frame.body[1]]) as usize;
                if 2 + topic_name_len + 12 > frame.body.len() {
                    return Err(BrokerError::InvalidFrame("fetch body size mismatch".into()));
                }

                let topic_name = std::str::from_utf8(&frame.body[2..2 + topic_name_len])
                    .map_err(|e| BrokerError::InvalidFrame(format!("invalid topic name: {}", e)))?;

                let pos = 2 + topic_name_len;
                let start_offset = u64::from_be_bytes([
                    frame.body[pos],
                    frame.body[pos + 1],
                    frame.body[pos + 2],
                    frame.body[pos + 3],
                    frame.body[pos + 4],
                    frame.body[pos + 5],
                    frame.body[pos + 6],
                    frame.body[pos + 7],
                ]);
                let _max_bytes = u32::from_be_bytes([
                    frame.body[pos + 8],
                    frame.body[pos + 9],
                    frame.body[pos + 10],
                    frame.body[pos + 11],
                ]);

                let mut current_offset = start_offset;
                let mut messages: Vec<(u64, Vec<u8>, Vec<u8>)> = Vec::new();

                for _ in 0..64 {
                    match self.log_manager.fetch(topic_name, current_offset) {
                        Ok((hdr, key, value)) => {
                            messages.push((hdr.offset, key, value));
                            current_offset = hdr.offset + 1;
                        }
                        Err(BrokerError::OffsetOutOfRange { .. }) => {
                            if messages.is_empty() {
                                return Ok(Frame::fetch_response(
                                    frame.correlation_id,
                                    ERROR_OFFSET_OUT_OF_RANGE,
                                    &[],
                                ));
                            }
                            break;
                        }
                        Err(BrokerError::TopicNotFound(_)) => {
                            return Ok(Frame::fetch_response(
                                frame.correlation_id,
                                ERROR_OFFSET_OUT_OF_RANGE,
                                &[],
                            ));
                        }
                        Err(e) => return Err(e),
                    }
                }

                Ok(Frame::fetch_response(
                    frame.correlation_id,
                    ERROR_NONE,
                    &messages,
                ))
            }

            API_LIST_OFFSETS => {
                if frame.body.len() < 2 {
                    return Err(BrokerError::InvalidFrame(
                        "list_offsets body too short".into(),
                    ));
                }

                let topic_name_len = u16::from_be_bytes([frame.body[0], frame.body[1]]) as usize;
                if 2 + topic_name_len > frame.body.len() {
                    return Err(BrokerError::InvalidFrame(
                        "list_offsets body size mismatch".into(),
                    ));
                }

                let topic_name = std::str::from_utf8(&frame.body[2..2 + topic_name_len])
                    .map_err(|e| BrokerError::InvalidFrame(format!("invalid topic name: {}", e)))?;

                let (earliest, latest) = self
                    .log_manager
                    .get_topic_offsets(topic_name)
                    .unwrap_or((Some(0), 0));

                Ok(Frame::list_offsets_response(
                    frame.correlation_id,
                    ERROR_NONE,
                    earliest.unwrap_or(0),
                    latest,
                ))
            }

            API_COMMIT => {
                if frame.body.len() < 2 {
                    return Err(BrokerError::InvalidFrame("commit body too short".into()));
                }

                let topic_name_len = u16::from_be_bytes([frame.body[0], frame.body[1]]) as usize;
                if 2 + topic_name_len + 8 > frame.body.len() {
                    return Err(BrokerError::InvalidFrame(
                        "commit body size mismatch".into(),
                    ));
                }

                Ok(Frame::commit_response(frame.correlation_id, ERROR_NONE))
            }

            _ => Err(BrokerError::InvalidFrame(format!(
                "unknown api_key: {}",
                frame.api_key
            ))),
        }
    }

    fn remove_connection(&mut self, token: Token) {
        let mut connections = self.connections.lock();
        if let Some(conn) = connections.remove(&token) {
            let _ = conn.stream.shutdown(std::net::Shutdown::Both);
            eprintln!(
                "[server] connection removed: {} (token={})",
                conn.addr, token.0
            );
        }
    }
}

impl Drop for BrokerServer {
    fn drop(&mut self) {
        let mut connections = self.connections.lock();
        for (_, conn) in connections.drain() {
            let _ = conn.stream.shutdown(std::net::Shutdown::Both);
        }
    }
}
