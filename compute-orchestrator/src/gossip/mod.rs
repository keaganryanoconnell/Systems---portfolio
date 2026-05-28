use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;

use rand::Rng as _;
use serde::{Deserialize, Serialize};
use tokio::net::UdpSocket;
use tokio::sync::Mutex;
use tokio::time::{interval, timeout};
use tracing::{debug, info, warn};

use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PeerState {
    Alive,
    Suspect,
    Dead,
    Joining,
    Left,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub addr: SocketAddr,
    pub state: PeerState,
    pub last_heartbeat: u64,
    pub incarnation: u64,
    #[serde(default)]
    pub metadata: PeerMetadata,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PeerMetadata {
    pub cpu_load: f32,
    pub mem_avail_mb: u64,
    pub task_queue_depth: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SwimMessageType {
    Ping = 0,
    Ack = 1,
    PingReq = 2,
}

impl SwimMessageType {
    pub fn from_byte(b: u8) -> Option<Self> {
        match b {
            0 => Some(Self::Ping),
            1 => Some(Self::Ack),
            2 => Some(Self::PingReq),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct SwimPacket {
    pub msg_type: SwimMessageType,
    pub seq_num: u32,
}

impl SwimPacket {
    pub fn to_bytes(&self) -> [u8; 5] {
        let mut buf = [0u8; 5];
        buf[0] = self.msg_type as u8;
        buf[1..5].copy_from_slice(&self.seq_num.to_be_bytes());
        buf
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 5 {
            return None;
        }
        let msg_type = SwimMessageType::from_byte(bytes[0])?;
        let seq_num = u32::from_be_bytes([bytes[1], bytes[2], bytes[3], bytes[4]]);
        Some(Self { msg_type, seq_num })
    }
}

#[derive(Debug, Clone)]
pub struct SwimConfig {
    pub ping_interval: Duration,
    pub ping_timeout: Duration,
    pub suspect_timeout: Duration,
    pub dead_timeout: Duration,
    pub metadata_ttl: u32,
}

impl Default for SwimConfig {
    fn default() -> Self {
        Self {
            ping_interval: Duration::from_secs(1),
            ping_timeout: Duration::from_millis(500),
            suspect_timeout: Duration::from_secs(3),
            dead_timeout: Duration::from_secs(6),
            metadata_ttl: 3,
        }
    }
}

pub struct SwimNode {
    addr: SocketAddr,
    socket: Arc<UdpSocket>,
    peers: Arc<Mutex<Vec<PeerInfo>>>,
    config: SwimConfig,
    seq_num: Arc<Mutex<u32>>,
    metadata: Arc<Mutex<PeerMetadata>>,
}

impl SwimNode {
    pub async fn new(bind_addr: SocketAddr, config: SwimConfig) -> Result<Self> {
        let socket = UdpSocket::bind(bind_addr).await?;
        let socket = Arc::new(socket);

        info!("SWIM node bound to {}", bind_addr);

        Ok(Self {
            addr: bind_addr,
            socket,
            peers: Arc::new(Mutex::new(Vec::new())),
            config,
            seq_num: Arc::new(Mutex::new(0)),
            metadata: Arc::new(Mutex::new(PeerMetadata::default())),
        })
    }

    pub async fn join(&self, peer_addrs: &[SocketAddr]) {
        let mut peers = self.peers.lock().await;
        let now = self.now_ms();

        for addr in peer_addrs {
            if !peers.iter().any(|p| p.addr == *addr) {
                peers.push(PeerInfo {
                    addr: *addr,
                    state: PeerState::Joining,
                    last_heartbeat: now,
                    incarnation: 0,
                    metadata: PeerMetadata::default(),
                });
            }
        }
    }

    pub async fn update_metadata(&self, metadata: PeerMetadata) {
        let mut m = self.metadata.lock().await;
        *m = metadata;
    }

    pub async fn start(self) {
        let peers = self.peers.clone();
        let socket = self.socket.clone();
        let config = self.config;
        let addr = self.addr;
        let seq_num = self.seq_num.clone();
        let metadata = self.metadata.clone();

        let listener_socket = socket.clone();
        let listener_peers = peers.clone();

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];
            loop {
                match listener_socket.recv_from(&mut buf).await {
                    Ok((n, src)) => {
                        if let Some(packet) = SwimPacket::from_bytes(&buf[..n]) {
                            Self::handle_packet(
                                packet,
                                src,
                                &listener_peers,
                                &listener_socket,
                                addr,
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        warn!("SWIM recv error: {}", e);
                    }
                }
            }
        });

        tokio::spawn(async move {
            let mut tick = interval(config.ping_interval);
            loop {
                tick.tick().await;
                Self::send_ping(&peers, &socket, &seq_num, &config, &metadata).await;
                Self::check_timeouts(&peers, &config).await;
            }
        });
    }

    async fn send_ping(
        peers: &Arc<Mutex<Vec<PeerInfo>>>,
        socket: &Arc<UdpSocket>,
        seq_num: &Arc<Mutex<u32>>,
        config: &SwimConfig,
        _metadata: &Arc<Mutex<PeerMetadata>>,
    ) {
        let p = peers.lock().await;
        if p.is_empty() {
            return;
        }

        let alive: Vec<usize> = p
            .iter()
            .enumerate()
            .filter(|(_, pi)| pi.state == PeerState::Alive || pi.state == PeerState::Joining)
            .map(|(i, _)| i)
            .collect();

        if alive.is_empty() {
            return;
        }

        let idx = alive[rand::thread_rng().gen_range(0..alive.len())];
        let target = p[idx].addr;

        let mut seq = seq_num.lock().await;
        *seq = seq.wrapping_add(1);
        let seq_val = *seq;

        let packet = SwimPacket {
            msg_type: SwimMessageType::Ping,
            seq_num: seq_val,
        };

        let _ = socket.send_to(&packet.to_bytes(), target).await;

        let peers_clone = peers.clone();
        let socket_clone = socket.clone();
        let target_addr = target;
        let config_clone = config.clone();
        tokio::spawn(async move {
            let mut buf = [0u8; 5];
            let result = timeout(config_clone.ping_timeout, async {
                loop {
                    match socket_clone.recv_from(&mut buf).await {
                        Ok((n, src)) if src == target_addr => {
                            if let Some(pkt) = SwimPacket::from_bytes(&buf[..n]) {
                                if pkt.msg_type == SwimMessageType::Ack && pkt.seq_num == seq_val {
                                    return;
                                }
                            }
                        }
                        Ok(_) => continue,
                        Err(_) => break,
                    }
                }
            })
            .await;

            if result.is_err() {
                let mut peers = peers_clone.lock().await;
                if let Some(pi) = peers.iter_mut().find(|p| p.addr == target_addr) {
                    if pi.state == PeerState::Alive {
                        pi.state = PeerState::Suspect;
                        debug!("Node {} marked suspect (ping timeout)", target_addr);
                    } else if pi.state == PeerState::Suspect {
                        pi.state = PeerState::Dead;
                        warn!("Node {} marked dead", target_addr);
                    }
                }
            } else {
                let mut peers = peers_clone.lock().await;
                if let Some(pi) = peers.iter_mut().find(|p| p.addr == target_addr) {
                    if pi.state != PeerState::Alive {
                        pi.state = PeerState::Alive;
                        pi.incarnation += 1;
                        debug!("Node {} restored to alive", target_addr);
                    }
                    pi.last_heartbeat = Self::now_ms_static();
                }
            }
        });
    }

    async fn handle_packet(
        packet: SwimPacket,
        src: SocketAddr,
        peers: &Arc<Mutex<Vec<PeerInfo>>>,
        socket: &Arc<UdpSocket>,
        _local_addr: SocketAddr,
    ) {
        let peers = peers.clone();
        let socket = socket.clone();

        tokio::spawn(async move {
            match packet.msg_type {
                SwimMessageType::Ping => {
                    let ack = SwimPacket {
                        msg_type: SwimMessageType::Ack,
                        seq_num: packet.seq_num,
                    };
                    let _ = socket.send_to(&ack.to_bytes(), src).await;
                }
                SwimMessageType::PingReq => {
                    let mut p = peers.lock().await;
                    if let Some(pi) = p.iter_mut().find(|p| p.addr == src) {
                        pi.state = PeerState::Alive;
                        pi.last_heartbeat = Self::now_ms_static();
                    }
                }
                SwimMessageType::Ack => {}
            }
        });
    }

    async fn check_timeouts(peers: &Arc<Mutex<Vec<PeerInfo>>>, config: &SwimConfig) {
        let mut p = peers.lock().await;
        let now = Self::now_ms_static();

        for pi in p.iter_mut() {
            match pi.state {
                PeerState::Alive
                    if now - pi.last_heartbeat > config.suspect_timeout.as_millis() as u64 =>
                {
                    pi.state = PeerState::Suspect;
                    warn!("Node {} marked suspect (heartbeat timeout)", pi.addr);
                }
                PeerState::Suspect
                    if now - pi.last_heartbeat > config.dead_timeout.as_millis() as u64 =>
                {
                    pi.state = PeerState::Dead;
                    warn!("Node {} marked dead", pi.addr);
                }
                _ => {}
            }
        }
    }

    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.lock().await.clone()
    }

    fn now_ms(&self) -> u64 {
        Self::now_ms_static()
    }

    fn now_ms_static() -> u64 {
        use std::time::SystemTime;
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64
    }
}
