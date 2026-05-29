use std::collections::HashMap;

use crate::lww::LwwSet;

pub struct SyncStateEngine {
    pub crdt: LwwSet<String>,
    pub peers: HashMap<u32, PeerState>,
    pub stats: SyncStats,
}

pub struct PeerState {
    pub peer_id: u32,
    pub last_seen_clock: u64,
    pub sync_count: u32,
    pub last_latency_ms: u64,
    pub connected: bool,
}

pub struct SyncStats {
    pub total_merges: u64,
    pub total_deltas_sent: u64,
    pub total_bytes_synced: u64,
    pub conflicts_resolved: u64,
    pub last_merge_duration_us: u64,
}

impl SyncStateEngine {
    pub fn new(peer_id: u32) -> Self {
        Self {
            crdt: LwwSet::new(peer_id),
            peers: HashMap::new(),
            stats: SyncStats {
                total_merges: 0,
                total_deltas_sent: 0,
                total_bytes_synced: 0,
                conflicts_resolved: 0,
                last_merge_duration_us: 0,
            },
        }
    }

    pub fn apply_delta(&mut self, delta: &LwwSet<String>, from_peer: u32, latency_ms: u64) {
        self.crdt.merge(delta);
        self.stats.total_merges += 1;
        self.stats.total_bytes_synced += delta.delta_size_bytes() as u64;
        self.stats.last_merge_duration_us = latency_ms * 1000;

        self.peers.entry(from_peer)
            .and_modify(|p| {
                p.last_seen_clock = delta.clock;
                p.sync_count += 1;
                p.last_latency_ms = latency_ms;
            })
            .or_insert(PeerState {
                peer_id: from_peer,
                last_seen_clock: delta.clock,
                sync_count: 1,
                last_latency_ms: latency_ms,
                connected: true,
            });
    }

    pub fn generate_and_send_delta(&mut self) -> (LwwSet<String>, u64) {
        let delta = self.crdt.compute_delta();
        let size = delta.delta_size_bytes() as u64;
        self.crdt.mark_merged();
        self.stats.total_deltas_sent += 1;
        self.stats.total_bytes_synced += size;
        (delta, size)
    }

    pub fn active_elements(&self) -> Vec<&String> {
        self.crdt.elements()
    }

    pub fn connected_peers(&self) -> Vec<u32> {
        self.peers.iter()
            .filter(|(_, p)| p.connected)
            .map(|(id, _)| *id)
            .collect()
    }
}
