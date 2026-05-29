pub mod compute;
pub mod envelope;
pub mod error;
pub mod frame;
pub mod message;
pub mod raft;
pub mod sql;
pub mod storage;
pub mod telemetry;

pub use compute::{ClusterHealth, ComputeRange, MacroTask, MicroTask, NodeCapacity, TaskResult};
pub use envelope::MessageEnvelope;
pub use error::{ProtocolError, ProtocolResult};
pub use frame::{Frame, FrameDecoder, MAGIC_BYTES, MAX_FRAME_SIZE, PROTOCOL_VERSION};
pub use message::MessageType;
pub use raft::{
    AppendEntriesArgs, AppendEntriesReply, InstallSnapshotArgs, InstallSnapshotReply,
    LogEntry, RequestVoteArgs, RequestVoteReply,
};
pub use sql::{ColumnDef, QueryPlan, Row, SqlQuery, SqlResult, SqlValue};
pub use storage::{
    DeleteRequest, DeleteResponse, GetRequest, GetResponse, KeyValue,
    PutRequest, ScanRequest, ScanResponse,
};
pub use telemetry::{MetricSnapshot, NodeTelemetry, TelemetryQuery};

pub fn new_trace_id() -> u128 {
    uuid::Uuid::new_v4().as_u128()
}
