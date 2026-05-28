use rand::Rng as _;

use crate::gossip::{PeerInfo, PeerState};

#[derive(Debug, Clone, Default)]
pub struct PlacementScore {
    pub node_id: u32,
    pub score: f32,
    pub cpu_load: f32,
    pub mem_avail_mb: u64,
    pub queue_depth: u32,
}

pub fn score_node(
    peer: &PeerInfo,
    cpu_weight: f32,
    mem_weight: f32,
    queue_weight: f32,
) -> PlacementScore {
    let m = &peer.metadata;
    let cpu_score = (1.0 - m.cpu_load).clamp(0.0, 1.0);
    let mem_score = (m.mem_avail_mb as f32 / 1024.0).min(1.0);
    let queue_score = (1.0 - (m.task_queue_depth as f32 / 100.0).min(1.0)).clamp(0.0, 1.0);

    let score = cpu_score * cpu_weight + mem_score * mem_weight + queue_score * queue_weight;

    PlacementScore {
        node_id: 0,
        score,
        cpu_load: m.cpu_load,
        mem_avail_mb: m.mem_avail_mb,
        queue_depth: m.task_queue_depth,
    }
}

pub fn select_best_node(peers: &[PeerInfo]) -> Option<(usize, PlacementScore)> {
    let alive: Vec<usize> = peers
        .iter()
        .enumerate()
        .filter(|(_, p)| p.state == PeerState::Alive)
        .map(|(i, _)| i)
        .collect();

    if alive.is_empty() {
        return None;
    }

    let mut scored: Vec<(usize, PlacementScore)> = alive
        .iter()
        .map(|&i| {
            let score = score_node(&peers[i], 0.4, 0.3, 0.3);
            (i, score)
        })
        .collect();

    scored.sort_by(|a, b| {
        b.1.score
            .partial_cmp(&a.1.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if scored.is_empty() {
        return None;
    }

    let best = &scored[0];
    if best.1.score <= 0.0 {
        let pick = rand::thread_rng().gen_range(0..scored.len());
        return Some(scored[pick].clone());
    }

    Some(scored[0].clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gossip::{PeerInfo, PeerMetadata, PeerState};

    #[test]
    fn test_empty_peers_returns_none() {
        assert!(select_best_node(&[]).is_none());
    }

    #[test]
    fn test_all_dead_returns_none() {
        let peers = vec![PeerInfo {
            addr: "127.0.0.1:8000".parse().unwrap(),
            state: PeerState::Dead,
            last_heartbeat: 0,
            incarnation: 0,
            metadata: PeerMetadata::default(),
        }];
        assert!(select_best_node(&peers).is_none());
    }

    #[test]
    fn test_selects_alive_peer() {
        let peers = vec![PeerInfo {
            addr: "127.0.0.1:8000".parse().unwrap(),
            state: PeerState::Alive,
            last_heartbeat: 0,
            incarnation: 0,
            metadata: PeerMetadata {
                cpu_load: 0.2,
                mem_avail_mb: 4096,
                task_queue_depth: 5,
            },
        }];
        let result = select_best_node(&peers);
        assert!(result.is_some());
    }
}
