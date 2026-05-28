//! SWIM Gossip Consensus Node
//!
//! Handles cluster peer group membership discovery and heartbeat checks
//! over raw UDP sockets, tracking node liveness metrics.

use core_sys::{log_error, log_info};
use std::io;
use std::net::{SocketAddr, UdpSocket};
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

/// Heartbeat message types used in the gossip protocol.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SwimMessageType {
    /// Direct ping sent to a target node
    Ping,
    /// Acknowledgment message returned by a live target node
    Ack,
}

/// A serialized gossip message.
pub struct SwimPacket {
    pub msg_type: SwimMessageType,
    pub seq_num: u32,
}

impl SwimPacket {
    /// Serializes the packet into a fixed-size 5-byte stack buffer.
    pub fn serialize(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        buf[0] = match self.msg_type {
            SwimMessageType::Ping => 0,
            SwimMessageType::Ack => 1,
        };
        let seq_bytes = self.seq_num.to_be_bytes();
        buf[1..5].copy_from_slice(&seq_bytes);
        buf
    }

    /// Deserializes a byte buffer into a structured packet.
    pub fn deserialize(buf: &[u8]) -> Option<Self> {
        if buf.len() < 5 {
            return None;
        }
        let msg_type = match buf[0] {
            0 => SwimMessageType::Ping,
            1 => SwimMessageType::Ack,
            _ => return None,
        };
        let mut seq_bytes = [0u8; 4];
        seq_bytes.copy_from_slice(&buf[1..5]);
        let seq_num = u32::from_be_bytes(seq_bytes);

        Some(Self { msg_type, seq_num })
    }
}

/// Liveness states of cluster nodes.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PeerState {
    /// Node is responsive and communicating
    Alive,
    /// Node missed a ping and is under observation
    Suspect,
    /// Node is determined to be offline and evicted
    Dead,
}

/// Represents membership record metadata for a cluster peer.
#[derive(Debug, Clone)]
pub struct PeerInfo {
    pub address: SocketAddr,
    pub state: PeerState,
    pub last_seen: Instant,
}

/// Represents the consensus engine daemon instance.
pub struct SwimNode {
    socket: UdpSocket,
    peers: Arc<Mutex<Vec<PeerInfo>>>,
    running: Arc<AtomicBool>,
    seq_num: Arc<AtomicU32>,
}

impl SwimNode {
    /// Binds to the preferred port, falling back to an ephemeral port if occupied.
    pub fn new(preferred_port: u16) -> io::Result<Self> {
        let addr = SocketAddr::from(([127, 0, 0, 1], preferred_port));

        let socket = match UdpSocket::bind(addr) {
            Ok(s) => s,
            Err(_) => {
                let fallback = SocketAddr::from(([127, 0, 0, 1], 0));
                UdpSocket::bind(fallback)?
            }
        };

        // Configure socket to operate in non-blocking mode to keep threads responsive
        socket.set_nonblocking(true)?;

        Ok(Self {
            socket,
            peers: Arc::new(Mutex::new(Vec::new())),
            running: Arc::new(AtomicBool::new(false)),
            seq_num: Arc::new(AtomicU32::new(0)),
        })
    }

    /// Returns the local socket bind address.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.socket.local_addr()
    }

    /// Adds a peer address to the gossip membership directory.
    pub fn add_peer(&self, addr: SocketAddr) {
        if let Ok(mut peers) = self.peers.lock() {
            // Check for duplicate listings
            if !peers.iter().any(|p| p.address == addr) {
                peers.push(PeerInfo {
                    address: addr,
                    state: PeerState::Alive,
                    last_seen: Instant::now(),
                });
                log_info!(
                    "platform-nodes::swim",
                    "Added peer node to SWIM gossip directory: {}",
                    addr
                );
            }
        }
    }

    /// Starts background threads for incoming packet listening and peer ping loops.
    pub fn start(&self) {
        self.running.store(true, Ordering::Release);

        let socket_rx = match self.socket.try_clone() {
            Ok(s) => s,
            Err(e) => {
                log_error!(
                    "platform-nodes::swim",
                    "Failed to duplicate socket: {:?}",
                    e
                );
                return;
            }
        };

        let peers_rx = self.peers.clone();
        let running_rx = self.running.clone();

        // 1. Spawns UDP Socket Listener Thread
        thread::spawn(move || {
            let mut read_buf = [0u8; 128];

            while running_rx.load(Ordering::Acquire) {
                match socket_rx.recv_from(&mut read_buf) {
                    Ok((bytes_read, src_addr)) => {
                        if let Some(packet) = SwimPacket::deserialize(&read_buf[..bytes_read]) {
                            match packet.msg_type {
                                SwimMessageType::Ping => {
                                    // Received peer ping: send back Ack response immediately
                                    let ack = SwimPacket {
                                        msg_type: SwimMessageType::Ack,
                                        seq_num: packet.seq_num,
                                    };
                                    let serialized = ack.serialize();
                                    let _ = socket_rx.send_to(&serialized, src_addr);
                                }
                                SwimMessageType::Ack => {
                                    // Received Ack response: refresh node state to Alive
                                    if let Ok(mut peers) = peers_rx.lock() {
                                        if let Some(peer) =
                                            peers.iter_mut().find(|p| p.address == src_addr)
                                        {
                                            if peer.state != PeerState::Alive {
                                                log_info!(
                                                    "platform-nodes::swim",
                                                    "Peer state recovered to ALIVE: {}",
                                                    src_addr
                                                );
                                            }
                                            peer.state = PeerState::Alive;
                                            peer.last_seen = Instant::now();
                                        }
                                    }
                                }
                            }
                        }
                    }
                    Err(ref err) if err.kind() == io::ErrorKind::WouldBlock => {
                        // Socket is empty, back off briefly
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(ref err) if err.kind() == io::ErrorKind::ConnectionReset => {
                        // Port unreachable, ignore and continue
                    }
                    Err(err) => {
                        log_error!("platform-nodes::swim", "UDP receive error: {:?}", err);
                    }
                }
            }
        });

        // 2. Spawns Periodic Gossip Ping Thread
        let socket_tx = match self.socket.try_clone() {
            Ok(s) => s,
            Err(e) => {
                log_error!(
                    "platform-nodes::swim",
                    "Failed to duplicate socket: {:?}",
                    e
                );
                return;
            }
        };
        let peers_tx = self.peers.clone();
        let running_tx = self.running.clone();
        let seq_num_tx = self.seq_num.clone();

        thread::spawn(move || {
            while running_tx.load(Ordering::Acquire) {
                thread::sleep(Duration::from_secs(1));

                let peer_to_ping = {
                    if let Ok(mut peers) = peers_tx.lock() {
                        let now = Instant::now();

                        // Age and suspect peers who missed heartbeats
                        for peer in peers.iter_mut() {
                            if peer.state == PeerState::Alive
                                && now.duration_since(peer.last_seen) > Duration::from_secs(3)
                            {
                                peer.state = PeerState::Suspect;
                                log_info!(
                                    "platform-nodes::swim",
                                    "Peer missed heartbeat, transitioning to SUSPECT: {}",
                                    peer.address
                                );
                            } else if peer.state == PeerState::Suspect
                                && now.duration_since(peer.last_seen) > Duration::from_secs(6)
                            {
                                peer.state = PeerState::Dead;
                                log_info!(
                                    "platform-nodes::swim",
                                    "Evicting unresponsive peer marked as DEAD: {}",
                                    peer.address
                                );
                            }
                        }

                        // Select a random non-dead peer to ping
                        let active_peers: Vec<SocketAddr> = peers
                            .iter()
                            .filter(|p| p.state != PeerState::Dead)
                            .map(|p| p.address)
                            .collect();

                        if !active_peers.is_empty() {
                            // Simple pseudo-random index selection
                            let idx =
                                (seq_num_tx.load(Ordering::Relaxed) as usize) % active_peers.len();
                            Some(active_peers[idx])
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                };

                if let Some(target) = peer_to_ping {
                    let seq = seq_num_tx.fetch_add(1, Ordering::SeqCst);
                    let ping = SwimPacket {
                        msg_type: SwimMessageType::Ping,
                        seq_num: seq,
                    };
                    let serialized = ping.serialize();
                    let _ = socket_tx.send_to(&serialized, target);
                }
            }
        });
    }

    /// Stops the consensus background loops.
    pub fn stop(&self) {
        self.running.store(false, Ordering::Release);
    }

    /// Queries the status of a specific peer address.
    pub fn get_peer_state(&self, addr: SocketAddr) -> Option<PeerState> {
        if let Ok(peers) = self.peers.lock() {
            peers.iter().find(|p| p.address == addr).map(|p| p.state)
        } else {
            None
        }
    }

    /// Returns the number of peers currently tracked in the directory.
    pub fn peer_count(&self) -> usize {
        if let Ok(peers) = self.peers.lock() {
            peers.len()
        } else {
            0
        }
    }
}

impl Drop for SwimNode {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_serialization() {
        let packet = SwimPacket {
            msg_type: SwimMessageType::Ping,
            seq_num: 1234,
        };
        let bytes = packet.serialize();
        let parsed = SwimPacket::deserialize(&bytes);
        assert!(parsed.is_some());
        let parsed = parsed.unwrap();
        assert_eq!(parsed.msg_type, SwimMessageType::Ping);
        assert_eq!(parsed.seq_num, 1234);

        let ack = SwimPacket {
            msg_type: SwimMessageType::Ack,
            seq_num: 5678,
        };
        let ack_bytes = ack.serialize();
        let parsed_ack = SwimPacket::deserialize(&ack_bytes);
        assert!(parsed_ack.is_some());
        let parsed_ack = parsed_ack.unwrap();
        assert_eq!(parsed_ack.msg_type, SwimMessageType::Ack);
        assert_eq!(parsed_ack.seq_num, 5678);
    }

    #[test]
    fn test_swim_node_ping_ack_flow() {
        // Initialize two nodes on local ephemeral ports
        let node_a = SwimNode::new(0).unwrap();
        let node_b = SwimNode::new(0).unwrap();

        let addr_a = node_a.local_addr().unwrap();
        let addr_b = node_b.local_addr().unwrap();

        // Register each other
        node_a.add_peer(addr_b);
        node_b.add_peer(addr_a);

        // Start background UDP listeners
        node_a.start();
        node_b.start();

        // Send a manual ping from A to B via UDP
        let ping = SwimPacket {
            msg_type: SwimMessageType::Ping,
            seq_num: 1,
        };
        let bytes = ping.serialize();
        node_a.socket.send_to(&bytes, addr_b).unwrap();

        // Wait a short duration for the UDP exchange to finish
        std::thread::sleep(Duration::from_millis(50));

        // Verify Node A received Node B's Ack and updated its state to Alive
        let state = node_a.get_peer_state(addr_b);
        assert!(state.is_some());
        assert_eq!(state.unwrap(), PeerState::Alive);

        // Stop nodes
        node_a.stop();
        node_b.stop();
    }
}
