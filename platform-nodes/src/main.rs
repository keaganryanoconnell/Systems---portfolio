//! Platform Nodes Server Daemon
//!
//! Handles core cloud-native server engines (LSM Storage, HTTP epoll proxy, Raft cluster).
//! Uses target conditional compilation gates to isolate Linux-specific APIs (epoll).

use platform_nodes::consensus::{self, SwimNode};
use platform_nodes::storage;
use core_sys::{init_telemetry_daemon, log_error, log_info, stop_telemetry_daemon};

use std::sync::Arc;

/// Context containing handles to consensus and storage, shared with the HTTP proxy.
#[derive(Clone)]
pub struct ProxyContext {
    pub swim_node: Option<Arc<consensus::SwimNode>>,
    pub lsm_engine: Option<storage::LsmEngine>,
}

/// Linux-specific engine using native epoll system calls via libc.
#[cfg(target_os = "linux")]
pub mod linux_engine {
    use super::*;
    use std::io;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    pub struct HttpProxy {
        port: u16,
        running: Arc<AtomicBool>,
        context: ProxyContext,
    }

    impl HttpProxy {
        pub fn new(port: u16, context: ProxyContext) -> Self {
            Self {
                port,
                running: Arc::new(AtomicBool::new(false)),
                context,
            }
        }

        pub fn start(&self) -> io::Result<()> {
            self.running.store(true, Ordering::Release);
            let running = self.running.clone();
            let port = self.port;
            let context = self.context.clone();

            thread::spawn(move || {
                log_info!(
                    "platform-nodes::linux_engine",
                    "Starting HTTP epoll proxy on port {}...",
                    port
                );

                unsafe {
                    let fd = libc::socket(libc::AF_INET, libc::SOCK_STREAM, 0);
                    if fd < 0 {
                        log_error!("platform-nodes::linux_engine", "Failed to create socket");
                        return;
                    }

                    // Set SO_REUSEADDR
                    let optval: libc::c_int = 1;
                    let _ = libc::setsockopt(
                        fd,
                        libc::SOL_SOCKET,
                        libc::SO_REUSEADDR,
                        &optval as *const _ as *const libc::c_void,
                        std::mem::size_of::<libc::c_int>() as libc::socklen_t,
                    );

                    // Set non-blocking
                    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
                    let _ = libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK);

                    // Bind
                    let mut addr: libc::sockaddr_in = std::mem::zeroed();
                    addr.sin_family = libc::AF_INET as libc::sa_family_t;
                    addr.sin_port = port.to_be();
                    addr.sin_addr.s_addr = libc::INADDR_ANY;

                    if libc::bind(
                        fd,
                        &addr as *const _ as *const libc::sockaddr,
                        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t,
                    ) < 0
                    {
                        log_error!(
                            "platform-nodes::linux_engine",
                            "Failed to bind socket on port {}",
                            port
                        );
                        let _ = libc::close(fd);
                        return;
                    }

                    if libc::listen(fd, 128) < 0 {
                        log_error!("platform-nodes::linux_engine", "Failed to listen");
                        let _ = libc::close(fd);
                        return;
                    }

                    let epoll_fd = libc::epoll_create1(0);
                    if epoll_fd < 0 {
                        log_error!("platform-nodes::linux_engine", "Failed to create epoll fd");
                        let _ = libc::close(fd);
                        return;
                    }

                    let mut event = libc::epoll_event {
                        events: (libc::EPOLLIN | libc::EPOLLET) as u32,
                        u64: fd as u64,
                    };

                    if libc::epoll_ctl(epoll_fd, libc::EPOLL_CTL_ADD, fd, &mut event) < 0 {
                        log_error!(
                            "platform-nodes::linux_engine",
                            "Failed to register listener fd with epoll"
                        );
                        let _ = libc::close(epoll_fd);
                        let _ = libc::close(fd);
                        return;
                    }

                    let mut events = [libc::epoll_event { events: 0, u64: 0 }; 64];

                    while running.load(Ordering::Acquire) {
                        let num_events = libc::epoll_wait(epoll_fd, events.as_mut_ptr(), 64, 100);
                        if num_events < 0 {
                            let err = io::Error::last_os_error();
                            if err.kind() != io::ErrorKind::Interrupted {
                                log_error!(
                                    "platform-nodes::linux_engine",
                                    "epoll_wait error: {:?}",
                                    err
                                );
                                break;
                            }
                            continue;
                        }

                        for i in 0..num_events as usize {
                            let event_fd = events[i].u64 as libc::c_int;
                            let event_mask = events[i].events;

                            if event_fd == fd {
                                // Accept new connections in ET mode
                                loop {
                                    let mut client_addr: libc::sockaddr_in = std::mem::zeroed();
                                    let mut client_addr_len =
                                        std::mem::size_of::<libc::sockaddr_in>() as libc::socklen_t;
                                    let client_fd = libc::accept4(
                                        fd,
                                        &mut client_addr as *mut _ as *mut libc::sockaddr,
                                        &mut client_addr_len,
                                        libc::SOCK_NONBLOCK | libc::SOCK_CLOEXEC,
                                    );

                                    if client_fd < 0 {
                                        let err = io::Error::last_os_error();
                                        if err.kind() == io::ErrorKind::WouldBlock {
                                            break;
                                        }
                                        log_error!(
                                            "platform-nodes::linux_engine",
                                            "Failed to accept connection: {:?}",
                                            err
                                        );
                                        break;
                                    }

                                    let mut client_ev = libc::epoll_event {
                                        events: (libc::EPOLLIN | libc::EPOLLET) as u32,
                                        u64: client_fd as u64,
                                    };
                                    if libc::epoll_ctl(
                                        epoll_fd,
                                        libc::EPOLL_CTL_ADD,
                                        client_fd,
                                        &mut client_ev,
                                    ) < 0
                                    {
                                        let _ = libc::close(client_fd);
                                    }
                                }
                            } else {
                                if (event_mask & libc::EPOLLIN as u32) != 0 {
                                    let mut read_buf = [0u8; 1024];
                                    let bytes_read = libc::read(
                                        event_fd,
                                        read_buf.as_mut_ptr() as *mut libc::c_void,
                                        read_buf.len(),
                                    );
                                    if bytes_read > 0 {
                                        let req_str = String::from_utf8_lossy(
                                            &read_buf[..bytes_read as usize],
                                        );
                                        let mut path = "/";
                                        if let Some(first_line) = req_str.lines().next() {
                                            let parts: Vec<&str> =
                                                first_line.split_whitespace().collect();
                                            if parts.len() >= 2 {
                                                path = parts[1];
                                            }
                                        }

                                        let response = if path == "/telemetry" {
                                            let swim_peers = context
                                                .swim_node
                                                .as_ref()
                                                .map_or(0, |s| s.peer_count());
                                            let lsm_sstables = context
                                                .lsm_engine
                                                .as_ref()
                                                .and_then(|e| e.sstable_count().ok())
                                                .unwrap_or(0);
                                            let body = format!(
                                                "{{\n  \"status\": \"ACTIVE\",\n  \"swim_peers\": {},\n  \"lsm_sstables\": {}\n}}",
                                                swim_peers, lsm_sstables
                                            );
                                            format!(
                                                "HTTP/1.1 200 OK\r\n\
                                                 Content-Type: application/json\r\n\
                                                 Content-Length: {}\r\n\
                                                 Connection: close\r\n\r\n\
                                                 {}",
                                                body.len(),
                                                body
                                            )
                                        } else {
                                            let body = "{\n  \"error\": \"Not Found\"\n}";
                                            format!(
                                                "HTTP/1.1 404 Not Found\r\n\
                                                 Content-Type: application/json\r\n\
                                                 Content-Length: {}\r\n\
                                                 Connection: close\r\n\r\n\
                                                 {}",
                                                body.len(),
                                                body
                                            )
                                        };

                                        let _ = libc::write(
                                            event_fd,
                                            response.as_ptr() as *const libc::c_void,
                                            response.len(),
                                        );
                                        let _ = libc::close(event_fd);
                                    } else {
                                        let _ = libc::close(event_fd);
                                    }
                                } else {
                                    let _ = libc::close(event_fd);
                                }
                            }
                        }
                    }

                    let _ = libc::close(epoll_fd);
                    let _ = libc::close(fd);
                }

                log_info!(
                    "platform-nodes::linux_engine",
                    "HTTP epoll proxy stopped cleanly."
                );
            });

            Ok(())
        }

        pub fn stop(&self) {
            self.running.store(false, Ordering::Release);
        }
    }
}

/// Fallback engine simulating event loop for development on non-Linux OS (Windows/macOS).
#[cfg(not(target_os = "linux"))]
pub mod fallback_engine {
    use super::*;
    use std::io::{self, Read, Write};
    use std::net::TcpListener;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::thread;

    pub struct HttpProxy {
        port: u16,
        running: Arc<AtomicBool>,
        context: ProxyContext,
    }

    impl HttpProxy {
        pub fn new(port: u16, context: ProxyContext) -> Self {
            Self {
                port,
                running: Arc::new(AtomicBool::new(false)),
                context,
            }
        }

        pub fn start(&self) -> io::Result<()> {
            self.running.store(true, Ordering::Release);
            let running = self.running.clone();
            let port = self.port;
            let context = self.context.clone();

            let listener = TcpListener::bind(format!("127.0.0.1:{}", port))?;
            listener.set_nonblocking(true)?;

            thread::spawn(move || {
                log_info!(
                    "platform-nodes::fallback_engine",
                    "Starting fallback TCP proxy on port {}...",
                    port
                );

                while running.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            let context_clone = context.clone();
                            thread::spawn(move || {
                                let mut read_buf = [0u8; 1024];
                                if let Ok(bytes_read) = stream.read(&mut read_buf) {
                                    if bytes_read > 0 {
                                        let req_str =
                                            String::from_utf8_lossy(&read_buf[..bytes_read]);
                                        let mut path = "/";
                                        if let Some(first_line) = req_str.lines().next() {
                                            let parts: Vec<&str> =
                                                first_line.split_whitespace().collect();
                                            if parts.len() >= 2 {
                                                path = parts[1];
                                            }
                                        }

                                        let response = if path == "/telemetry" {
                                            let swim_peers = context_clone
                                                .swim_node
                                                .as_ref()
                                                .map_or(0, |s| s.peer_count());
                                            let lsm_sstables = context_clone
                                                .lsm_engine
                                                .as_ref()
                                                .and_then(|e| e.sstable_count().ok())
                                                .unwrap_or(0);
                                            let body = format!(
                                                "{{\n  \"status\": \"ACTIVE\",\n  \"swim_peers\": {},\n  \"lsm_sstables\": {}\n}}",
                                                swim_peers, lsm_sstables
                                            );
                                            format!(
                                                "HTTP/1.1 200 OK\r\n\
                                                 Content-Type: application/json\r\n\
                                                 Content-Length: {}\r\n\
                                                 Connection: close\r\n\r\n\
                                                 {}",
                                                body.len(),
                                                body
                                            )
                                        } else {
                                            let body = "{\n  \"error\": \"Not Found\"\n}";
                                            format!(
                                                "HTTP/1.1 404 Not Found\r\n\
                                                 Content-Type: application/json\r\n\
                                                 Content-Length: {}\r\n\
                                                 Connection: close\r\n\r\n\
                                                 {}",
                                                body.len(),
                                                body
                                            )
                                        };

                                        let _ = stream.write_all(response.as_bytes());
                                        let _ = stream.flush();
                                    }
                                }
                            });
                        }
                        Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                            thread::sleep(std::time::Duration::from_millis(5));
                        }
                        Err(err) => {
                            log_error!(
                                "platform-nodes::fallback_engine",
                                "Accept error: {:?}",
                                err
                            );
                        }
                    }
                }

                log_info!(
                    "platform-nodes::fallback_engine",
                    "Fallback TCP proxy stopped cleanly."
                );
            });

            Ok(())
        }

        pub fn stop(&self) {
            self.running.store(false, Ordering::Release);
        }
    }
}

fn main() {
    // Initialize the telemetry logger background daemon
    init_telemetry_daemon();

    log_info!("platform-nodes::main", "Starting Platform Nodes daemon...");

    // Initialize LSM Storage Engine
    let lsm_config = storage::LsmConfig::default();
    let _lsm_engine = match storage::LsmEngine::open(lsm_config) {
        Ok(engine) => {
            log_info!(
                "platform-nodes::main",
                "LSM Storage Engine active at {:?}",
                storage::LsmConfig::default().data_dir
            );
            Some(engine)
        }
        Err(err) => {
            log_error!(
                "platform-nodes::main",
                "Failed to initialize LSM Storage Engine: {:?}",
                err
            );
            None
        }
    };

    // Bind and start the SWIM Gossip Consensus Node (standard port 7946)
    let swim_port = 7946;
    let swim_node = match SwimNode::new(swim_port) {
        Ok(node) => {
            let node_arc = Arc::new(node);
            let local_addr = match node_arc.local_addr() {
                Ok(addr) => addr,
                Err(_) => std::net::SocketAddr::from(([127, 0, 0, 1], 0)),
            };
            log_info!(
                "platform-nodes::main",
                "SWIM Gossip Consensus active on UDP: {}",
                local_addr
            );
            node_arc.start();
            Some(node_arc)
        }
        Err(err) => {
            log_error!(
                "platform-nodes::main",
                "Failed to initialize SWIM Gossip node: {:?}",
                err
            );
            None
        }
    };

    // Create the shared proxy context
    let proxy_context = ProxyContext {
        swim_node: swim_node.clone(),
        lsm_engine: _lsm_engine.clone(),
    };

    // Start HTTP & Telemetry Proxy on port 8080
    let http_port = 8080;

    #[cfg(target_os = "linux")]
    let http_proxy = linux_engine::HttpProxy::new(http_port, proxy_context);

    #[cfg(not(target_os = "linux"))]
    let http_proxy = fallback_engine::HttpProxy::new(http_port, proxy_context);

    if let Err(err) = http_proxy.start() {
        log_error!(
            "platform-nodes::main",
            "Failed to start HTTP Telemetry Proxy: {:?}",
            err
        );
    }

    // Let the daemon run briefly in this skeleton to verify operation
    std::thread::sleep(std::time::Duration::from_millis(50));

    // Stop the HTTP proxy
    http_proxy.stop();

    // Stop background consensus loops
    if let Some(node) = swim_node {
        node.stop();
    }

    log_info!(
        "platform-nodes::main",
        "Platform Nodes daemon shutdown cleanly."
    );

    // Stop background logger
    stop_telemetry_daemon();
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;

    #[test]
    fn test_http_proxy_telemetry_endpoint() {
        let test_dir = std::path::PathBuf::from("data/http_proxy_test_lsm");
        if test_dir.exists() {
            let _ = std::fs::remove_dir_all(&test_dir);
        }
        let config = storage::LsmConfig {
            data_dir: test_dir.clone(),
            flush_threshold_bytes: 1024,
            compaction_trigger_files: 4,
        };
        let lsm = storage::LsmEngine::open(config).unwrap();

        let context = ProxyContext {
            swim_node: None,
            lsm_engine: Some(lsm),
        };

        let test_port = 18080;

        #[cfg(target_os = "linux")]
        let proxy = linux_engine::HttpProxy::new(test_port, context);

        #[cfg(not(target_os = "linux"))]
        let proxy = fallback_engine::HttpProxy::new(test_port, context);

        proxy.start().unwrap();

        // Let the background thread bind the socket
        std::thread::sleep(std::time::Duration::from_millis(50));

        // Connect and query the endpoint
        let mut stream = TcpStream::connect(format!("127.0.0.1:{}", test_port)).unwrap();
        stream
            .write_all(b"GET /telemetry HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n")
            .unwrap();
        stream.flush().unwrap();

        let mut response = String::new();
        stream.read_to_string(&mut response).unwrap();

        assert!(response.starts_with("HTTP/1.1 200 OK"));
        assert!(response.contains("swim_peers"));
        assert!(response.contains("lsm_sstables"));
        assert!(response.contains("\"status\": \"ACTIVE\""));

        proxy.stop();

        let _ = std::fs::remove_dir_all(&test_dir);
    }
}
