use crate::store::Command;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub term: usize,
    pub command: Command,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestVoteArgs {
    pub term: usize,
    pub candidate_id: usize,
    pub last_log_index: usize,
    pub last_log_term: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RequestVoteReply {
    pub term: usize,
    pub vote_granted: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendEntriesArgs {
    pub term: usize,
    pub leader_id: usize,
    pub prev_log_index: usize,
    pub prev_log_term: usize,
    pub entries: Vec<LogEntry>,
    pub leader_commit: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AppendEntriesReply {
    pub term: usize,
    pub success: bool,
    pub match_index: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Message {
    RequestVote {
        args: RequestVoteArgs,
        from: usize,
    },
    RequestVoteResponse {
        reply: RequestVoteReply,
        from: usize,
    },
    AppendEntries {
        args: AppendEntriesArgs,
        from: usize,
    },
    AppendEntriesResponse {
        reply: AppendEntriesReply,
        from: usize,
    },
    ClientWrite {
        key: String,
        value: String,
        client_id: usize,
    },
    ClientWriteResponse {
        success: bool,
        leader_id: Option<usize>,
        client_id: usize,
    },
}
