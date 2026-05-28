use crate::rpc::Message;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

pub struct NetworkRouter {
    // Inbound queue for router to dispatch
    rx: mpsc::Receiver<(usize, usize, Message)>,

    // Outbound channels going directly to each node
    nodes_tx: HashMap<usize, mpsc::Sender<Message>>,

    // Partition topological sets: e.g. [{1, 2}, {3, 4, 5}]
    partitions: Arc<Mutex<Vec<HashSet<usize>>>>,

    // Set of currently offline (crashed) node IDs
    crashed: Arc<Mutex<HashSet<usize>>>,
}

impl NetworkRouter {
    pub fn new(
        rx: mpsc::Receiver<(usize, usize, Message)>,
        nodes_tx: HashMap<usize, mpsc::Sender<Message>>,
        partitions: Arc<Mutex<Vec<HashSet<usize>>>>,
        crashed: Arc<Mutex<HashSet<usize>>>,
    ) -> Self {
        Self {
            rx,
            nodes_tx,
            partitions,
            crashed,
        }
    }

    pub async fn run(&mut self, mut kill_rx: tokio::sync::watch::Receiver<bool>) {
        loop {
            tokio::select! {
                _ = kill_rx.changed() => {
                    if *kill_rx.borrow() { break; }
                }
                envelope = self.rx.recv() => {
                    if let Some((from, to, msg)) = envelope {
                        // 1. Check if either source or destination node is crashed
                        {
                            let crashed = self.crashed.lock().unwrap_or_else(|e| e.into_inner());
                            if crashed.contains(&from) || crashed.contains(&to) {
                                continue; // Drop packet
                            }
                        }

                        // 2. Check if there is a partition blocking communication
                        {
                            let partitions = self.partitions.lock().unwrap_or_else(|e| e.into_inner());
                            if !partitions.is_empty() {
                                let mut in_same_partition = false;
                                for group in partitions.iter() {
                                    if group.contains(&from) && group.contains(&to) {
                                        in_same_partition = true;
                                        break;
                                    }
                                }
                                if !in_same_partition {
                                    continue; // Drop packet due to split brain
                                }
                            }
                        }

                        // 3. Dispatch to node receiver
                        if let Some(tx) = self.nodes_tx.get(&to) {
                            let _ = tx.send(msg).await;
                        }
                    }
                }
            }
        }
    }
}
