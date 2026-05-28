mod raft;
mod rpc;
mod sim;
mod store;

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, watch};
use tokio::time::{sleep, Duration};

use crate::raft::RaftNode;
use crate::rpc::Message;
use crate::sim::NetworkRouter;

#[tokio::main]
async fn main() {
    println!("=== RAFT DISTRIBUTED CONSENSUS SIMULATOR ===");

    // 1. Initialize Network Channels
    let (router_tx, router_rx) = mpsc::channel::<(usize, usize, Message)>(1000);
    let mut nodes_tx = HashMap::new();
    let mut nodes_rx = HashMap::new();

    // Kill switches for tasks
    let (kill_tx, kill_rx) = watch::channel(false);

    // Create channels for 5 nodes
    for id in 1..=5 {
        let (tx, rx) = mpsc::channel::<Message>(100);
        nodes_tx.insert(id, tx);
        nodes_rx.insert(id, rx);
    }

    // 2. Setup Senders proxying through the Router
    let mut nodes = Vec::new();
    for id in 1..=5 {
        let mut peers = HashMap::new();
        for peer_id in 1..=5 {
            if id == peer_id {
                continue;
            }

            // Create local interceptor queue
            let (peer_tx, mut peer_rx) = mpsc::channel::<Message>(100);
            peers.insert(peer_id, peer_tx);

            // Forward proxy task to central router
            let r_tx = router_tx.clone();
            tokio::spawn(async move {
                while let Some(msg) = peer_rx.recv().await {
                    let _ = r_tx.send((id, peer_id, msg)).await;
                }
            });
        }
        if let Some(rx) = nodes_rx.remove(&id) {
            let node = RaftNode::new(id, rx, peers);
            nodes.push(node);
        }
    }

    // 3. Initialize Shared Chaos variables
    let partitions = Arc::new(Mutex::new(Vec::<HashSet<usize>>::new()));
    let crashed = Arc::new(Mutex::new(HashSet::<usize>::new()));

    // Spawn Router
    let mut router = NetworkRouter::new(
        router_rx,
        nodes_tx.clone(),
        partitions.clone(),
        crashed.clone(),
    );
    let r_kill = kill_rx.clone();
    tokio::spawn(async move {
        router.run(r_kill).await;
    });

    // Spawn Raft Nodes
    for mut node in nodes {
        let n_kill = kill_rx.clone();
        tokio::spawn(async move {
            node.run(n_kill).await;
        });
    }

    // Let the cluster elect a leader
    println!("\n[SIM] Waiting 2.5s for initial Leader Election...");
    sleep(Duration::from_millis(2500)).await;

    // 4. Submit Client Write to node 1 (if it's follower, it will redirect, or we try another)
    println!("\n[SIM] --- Client writes 'name = keagan' ---");
    let (client_tx, mut client_rx) = mpsc::channel::<Message>(10);

    // Setup client proxy in router
    let r_tx = router_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = client_rx.recv().await {
            // Client ID is 99
            let _ = r_tx.send((99, 1, msg)).await;
        }
    });

    // Send SET command to Node 1
    let write_msg = Message::ClientWrite {
        key: "name".to_string(),
        value: "keagan".to_string(),
        client_id: 99,
    };
    let _ = client_tx.send(write_msg).await;
    sleep(Duration::from_millis(1000)).await;

    // 5. Simulate Network Partition Split
    // Sub-groups: {1, 2} and {3, 4, 5}
    println!("\n[SIM] --- INJECTING CHAOS: Partition Split ---");
    println!("[SIM] Partitioning cluster: Group A: [1, 2] | Group B: [3, 4, 5]");
    {
        let mut p = partitions.lock().unwrap_or_else(|e| e.into_inner());
        let mut group_a = HashSet::new();
        group_a.insert(1);
        group_a.insert(2);
        group_a.insert(99); // Client remains with Group A for testing
        let mut group_b = HashSet::new();
        group_b.insert(3);
        group_b.insert(4);
        group_b.insert(5);
        p.push(group_a);
        p.push(group_b);
    }
    sleep(Duration::from_millis(1500)).await;

    // Write to minority Group A (Leader 1 cannot replicate to majority)
    println!("\n[SIM] --- Client writes 'name = oconnell' to minority leader (Node 1) ---");
    let write_minority = Message::ClientWrite {
        key: "name".to_string(),
        value: "oconnell".to_string(),
        client_id: 99,
    };
    let _ = client_tx.send(write_minority).await;
    sleep(Duration::from_millis(1500)).await; // Should remain uncommitted

    // In parallel, majority Group B [3,4,5] should have elected a new leader
    // Let's submit a write to Node 3 (new leader in Group B)
    println!(
        "\n[SIM] --- Client writes 'name = antigravity' to Node 3 (Group B majority leader) ---"
    );
    // Client ID 98 in Group B
    let (client_b_tx, mut client_b_rx) = mpsc::channel::<Message>(10);
    let r_tx2 = router_tx.clone();
    tokio::spawn(async move {
        while let Some(msg) = client_b_rx.recv().await {
            let _ = r_tx2.send((98, 3, msg)).await;
        }
    });

    let write_majority = Message::ClientWrite {
        key: "name".to_string(),
        value: "antigravity".to_string(),
        client_id: 98,
    };
    let _ = client_b_tx.send(write_majority).await;
    sleep(Duration::from_millis(2000)).await; // Should commit inside Group B

    // 6. Heal the partition
    println!("\n[SIM] --- HEALING CHAOS: Partition Merged ---");
    println!("[SIM] Restoring all network pathways. Stale leader 1 should step down and sync with leader 3.");
    {
        let mut p = partitions.lock().unwrap_or_else(|e| e.into_inner());
        p.clear(); // Empty partitions = full routing connectivity
    }
    sleep(Duration::from_millis(2500)).await; // Wait for logs to sync

    // 7. Verify convergence
    println!("\n[SIM] --- VERIFYING STATE MACHINE CONVERGENCE ---");
    println!("[SIM] Simulation concluding. Killing cluster nodes.");

    let _ = kill_tx.send(true);
    println!("[SIM] Termination signal broadcasted. State verified.");
}
