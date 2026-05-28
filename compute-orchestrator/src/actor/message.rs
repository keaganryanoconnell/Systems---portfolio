use serde::{Deserialize, Serialize};

use super::pid::ProcessId;
use crate::network::serializer::MessageType;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActorMessage {
    pub sender: ProcessId,
    pub recipient: ProcessId,
    pub msg_type: MessageType,
    pub payload: Vec<u8>,
}

impl ActorMessage {
    pub fn new<T: Serialize>(
        sender: ProcessId,
        recipient: ProcessId,
        msg_type: MessageType,
        payload: &T,
    ) -> crate::error::Result<Self> {
        let encoded = bincode::serialize(payload)?;
        Ok(Self {
            sender,
            recipient,
            msg_type,
            payload: encoded,
        })
    }

    pub fn decode<T: serde::de::DeserializeOwned>(&self) -> crate::error::Result<T> {
        Ok(bincode::deserialize(&self.payload)?)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ActorState {
    Created,
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
}
