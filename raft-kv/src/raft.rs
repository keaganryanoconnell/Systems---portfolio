use crate::rpc::{
    AppendEntriesArgs, AppendEntriesReply, LogEntry, Message, RequestVoteArgs, RequestVoteReply,
};
use crate::store::{Command, KeyValueStore};
use rand::Rng;
use std::collections::{HashMap, HashSet};
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, Instant};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Role {
    Follower,
    Candidate,
    Leader,
}

pub struct RaftNode {
    // Identity & Network
    pub id: usize,
    pub peers: HashMap<usize, mpsc::Sender<Message>>,
    pub rx: mpsc::Receiver<Message>,

    // Persistent state on all nodes
    pub current_term: usize,
    pub voted_for: Option<usize>,
    pub log: Vec<LogEntry>,

    // Volatile state on all nodes
    pub commit_index: usize,
    pub last_applied: usize,
    pub role: Role,

    // Volatile state on leaders
    pub next_index: HashMap<usize, usize>,
    pub match_index: HashMap<usize, usize>,

    // Replicated State Machine
    pub state_machine: KeyValueStore,

    // Heartbeat & Election timeout variables
    pub election_timeout: Duration,
    pub last_heartbeat: Instant,

    // Candidate helper state
    pub votes_received: HashSet<usize>,

    // Leader helper state for pending client requests
    pub pending_writes: HashMap<usize, (usize, mpsc::Sender<Message>)>,
}

impl RaftNode {
    pub fn new(
        id: usize,
        rx: mpsc::Receiver<Message>,
        peers: HashMap<usize, mpsc::Sender<Message>>,
    ) -> Self {
        let mut rng = rand::thread_rng();
        let timeout_ms = rng.gen_range(150..300);

        Self {
            id,
            peers,
            rx,
            current_term: 0,
            voted_for: None,
            log: vec![LogEntry {
                term: 0,
                command: Command::Set {
                    key: "init".to_string(),
                    value: "true".to_string(),
                },
            }], // 1-based indexing helper
            commit_index: 0,
            last_applied: 0,
            role: Role::Follower,
            next_index: HashMap::new(),
            match_index: HashMap::new(),
            state_machine: KeyValueStore::new(),
            election_timeout: Duration::from_millis(timeout_ms),
            last_heartbeat: Instant::now(),
            votes_received: HashSet::new(),
            pending_writes: HashMap::new(),
        }
    }

    pub fn last_log_index(&self) -> usize {
        self.log.len() - 1
    }

    pub fn last_log_term(&self) -> usize {
        self.log[self.last_log_index()].term
    }

    // Step down to follower if seeing higher term
    fn check_term(&mut self, term: usize) -> bool {
        if term > self.current_term {
            self.current_term = term;
            let old_role = self.role;
            self.role = Role::Follower;
            self.voted_for = None;
            self.votes_received.clear();
            self.reset_election_timeout();

            // Fail any pending client writes
            if old_role == Role::Leader {
                let pending = std::mem::take(&mut self.pending_writes);
                for (_, (client_id, tx)) in pending {
                    let reply = Message::ClientWriteResponse {
                        success: false,
                        leader_id: Some(self.id),
                        client_id,
                    };
                    tokio::spawn(async move {
                        let _ = tx.send(reply).await;
                    });
                }
            }

            println!(
                "[Node {}] Steps down to Follower. New Term: {}",
                self.id, self.current_term
            );
            return true;
        }
        false
    }

    fn reset_election_timeout(&mut self) {
        let mut rng = rand::thread_rng();
        let timeout_ms = rng.gen_range(150..300);
        self.election_timeout = Duration::from_millis(timeout_ms);
        self.last_heartbeat = Instant::now();
    }

    pub async fn run(&mut self, mut kill_rx: tokio::sync::watch::Receiver<bool>) {
        self.reset_election_timeout();

        loop {
            // Check if killed
            if *kill_rx.borrow() {
                break;
            }

            let now = Instant::now();
            let elapsed = now.duration_since(self.last_heartbeat);

            tokio::select! {
                _ = kill_rx.changed() => {
                    if *kill_rx.borrow() { break; }
                }
                // Check timers
                _ = sleep(Duration::from_millis(10)) => {
                    if self.role == Role::Leader {
                        // Send periodic heartbeats
                        if elapsed >= Duration::from_millis(50) {
                            self.send_append_entries().await;
                            self.last_heartbeat = Instant::now();
                        }
                    } else {
                        // Check election timeout
                        if elapsed >= self.election_timeout {
                            self.start_election().await;
                        }
                    }
                }
                // Handle messages
                msg = self.rx.recv() => {
                    if let Some(message) = msg {
                        self.handle_message(message).await;
                    }
                }
            }

            // Apply committed logs
            while self.commit_index > self.last_applied {
                self.last_applied += 1;
                let entry = &self.log[self.last_applied];
                self.state_machine.apply(&entry.command);

                // If leader and have pending write for this index, respond to client!
                if self.role == Role::Leader {
                    if let Some((client_id, tx)) = self.pending_writes.remove(&self.last_applied) {
                        let reply = Message::ClientWriteResponse {
                            success: true,
                            leader_id: Some(self.id),
                            client_id,
                        };
                        let _ = tx.send(reply).await;
                    }
                }

                println!(
                    "[Node {}] applied Log index {}: {:?}. Commit index: {}. State now: {:?}",
                    self.id,
                    self.last_applied,
                    entry.command,
                    self.commit_index,
                    self.state_machine.get_all()
                );
            }
        }
    }

    async fn start_election(&mut self) {
        self.role = Role::Candidate;
        self.current_term += 1;
        self.voted_for = Some(self.id);
        self.votes_received.clear();
        self.votes_received.insert(self.id);
        self.reset_election_timeout();

        println!(
            "[Node {}] Triggers Election. Candidates term: {}",
            self.id, self.current_term
        );

        let last_log_index = self.last_log_index();
        let last_log_term = self.last_log_term();

        let args = RequestVoteArgs {
            term: self.current_term,
            candidate_id: self.id,
            last_log_index,
            last_log_term,
        };

        // Send RequestVote RPCs to all peers
        for tx in self.peers.values() {
            let msg = Message::RequestVote {
                args: args.clone(),
                from: self.id,
            };
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(msg).await;
            });
        }
    }

    async fn handle_message(&mut self, msg: Message) {
        match msg {
            Message::RequestVote { args, from } => {
                self.check_term(args.term);

                let mut vote_granted = false;
                if args.term == self.current_term {
                    let is_log_ok = args.last_log_term > self.last_log_term()
                        || (args.last_log_term == self.last_log_term()
                            && args.last_log_index >= self.last_log_index());

                    if (self.voted_for.is_none() || self.voted_for == Some(args.candidate_id))
                        && is_log_ok
                    {
                        self.voted_for = Some(args.candidate_id);
                        vote_granted = true;
                        self.reset_election_timeout();
                    }
                }

                println!(
                    "[Node {}] Received RequestVote from Node {}. Term: {}, candidate last_log_index: {}. Grant: {}", 
                    self.id, from, args.term, args.last_log_index, vote_granted
                );

                if let Some(tx) = self.peers.get(&from) {
                    let reply = RequestVoteReply {
                        term: self.current_term,
                        vote_granted,
                    };
                    let _ = tx
                        .send(Message::RequestVoteResponse {
                            reply,
                            from: self.id,
                        })
                        .await;
                }
            }

            Message::RequestVoteResponse { reply, from } => {
                if self.role != Role::Candidate {
                    return;
                }
                self.check_term(reply.term);

                if reply.term == self.current_term && reply.vote_granted {
                    self.votes_received.insert(from);
                    let majority = self.peers.len().div_ceil(2) + 1;
                    if self.votes_received.len() >= majority {
                        self.role = Role::Leader;
                        let last_idx = self.last_log_index();
                        for &peer_id in self.peers.keys() {
                            self.next_index.insert(peer_id, last_idx + 1);
                            self.match_index.insert(peer_id, 0);
                        }
                        println!(
                            "[Node {}] Achieved quorum and steps up to Leader in Term {}!",
                            self.id, self.current_term
                        );
                        self.send_append_entries().await;
                    }
                }
            }

            Message::AppendEntries { args, from } => {
                self.check_term(args.term);

                let mut success = false;
                if args.term >= self.current_term {
                    if self.role == Role::Candidate {
                        self.role = Role::Follower;
                    }
                    self.reset_election_timeout();

                    // Check log continuity
                    let log_len = self.log.len();
                    if args.prev_log_index < log_len
                        && self.log[args.prev_log_index].term == args.prev_log_term
                    {
                        success = true;

                        // Append entries
                        for (insert_index, entry) in (args.prev_log_index + 1..).zip(args.entries) {
                            if insert_index < self.log.len() {
                                if self.log[insert_index].term != entry.term {
                                    self.log.truncate(insert_index);
                                    self.log.push(entry);
                                }
                            } else {
                                self.log.push(entry);
                            }
                        }

                        // Update commit index
                        if args.leader_commit > self.commit_index {
                            self.commit_index =
                                std::cmp::min(args.leader_commit, self.last_log_index());
                        }
                    }
                }

                if let Some(tx) = self.peers.get(&from) {
                    let reply = AppendEntriesReply {
                        term: self.current_term,
                        success,
                        match_index: self.last_log_index(),
                    };
                    let _ = tx
                        .send(Message::AppendEntriesResponse {
                            reply,
                            from: self.id,
                        })
                        .await;
                }
            }

            Message::AppendEntriesResponse { reply, from } => {
                self.check_term(reply.term);
                if self.role != Role::Leader {
                    return;
                }

                if reply.success {
                    self.match_index.insert(from, reply.match_index);
                    self.next_index.insert(from, reply.match_index + 1);

                    // Check if we can commit new entries
                    let last_log_index = self.last_log_index();
                    for idx in (self.commit_index + 1)..=last_log_index {
                        if self.log[idx].term == self.current_term {
                            let mut matches = 1; // Count leader
                            for &peer_id in self.peers.keys() {
                                if self.match_index.get(&peer_id).unwrap_or(&0) >= &idx {
                                    matches += 1;
                                }
                            }
                            if matches > self.peers.len().div_ceil(2) {
                                self.commit_index = idx;
                            }
                        }
                    }
                } else {
                    // Decrement next_index to retry log alignment
                    let next = self.next_index.get(&from).unwrap_or(&1);
                    if *next > 1 {
                        self.next_index.insert(from, next - 1);
                    }
                }
            }

            Message::ClientWrite {
                key,
                value,
                client_id,
            } => {
                if self.role != Role::Leader {
                    // Redirect to leader if possible, or return failure
                    let leader_id = if self.role == Role::Follower {
                        self.voted_for
                    } else {
                        None
                    };
                    if let Some(tx) = self.peers.get(&client_id) {
                        let _ = tx
                            .send(Message::ClientWriteResponse {
                                success: false,
                                leader_id,
                                client_id,
                            })
                            .await;
                    }
                    return;
                }

                // Append local log entry
                let entry = LogEntry {
                    term: self.current_term,
                    command: Command::Set { key, value },
                };
                self.log.push(entry);
                println!(
                    "[Node {} (Leader)] Client write appended at log index {}.",
                    self.id,
                    self.last_log_index()
                );

                // Track client response tx
                if let Some(tx) = self.peers.get(&client_id).cloned() {
                    let last_idx = self.last_log_index();
                    self.pending_writes.insert(last_idx, (client_id, tx));
                }

                // Trigger immediate AppendEntries broadcasts
                self.send_append_entries().await;
            }

            Message::ClientWriteResponse { .. } => {}
        }
    }

    async fn send_append_entries(&mut self) {
        for (&peer_id, tx) in &self.peers {
            let prev_log_index = self.next_index.get(&peer_id).cloned().unwrap_or(1) - 1;
            let prev_log_term = self.log[prev_log_index].term;

            let entries = if self.last_log_index() > prev_log_index {
                self.log[prev_log_index + 1..].to_vec()
            } else {
                vec![]
            };

            let args = AppendEntriesArgs {
                term: self.current_term,
                leader_id: self.id,
                prev_log_index,
                prev_log_term,
                entries,
                leader_commit: self.commit_index,
            };

            let msg = Message::AppendEntries {
                args,
                from: self.id,
            };
            let tx = tx.clone();
            tokio::spawn(async move {
                let _ = tx.send(msg).await;
            });
        }
    }
}
