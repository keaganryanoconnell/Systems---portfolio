use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MessageType {
    TaskDispatch = 1,
    TaskResult = 2,
    ActorSpawn = 3,
    ActorStop = 4,
    Heartbeat = 5,
}

impl MessageType {
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::TaskDispatch => 1,
            Self::TaskResult => 2,
            Self::ActorSpawn => 3,
            Self::ActorStop => 4,
            Self::Heartbeat => 5,
        }
    }

    pub fn from_u32(n: u32) -> Self {
        match n {
            1 => Self::TaskDispatch,
            2 => Self::TaskResult,
            3 => Self::ActorSpawn,
            4 => Self::ActorStop,
            5 => Self::Heartbeat,
            _ => Self::TaskDispatch,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub actor_id: u64,
    pub msg_type: u32,
    pub payload: Vec<u8>,
}

impl MessageEnvelope {
    pub fn new<T: Serialize>(
        actor_id: u64,
        msg_type: MessageType,
        payload: &T,
    ) -> crate::error::Result<Self> {
        let encoded = bincode::serialize(payload)?;
        Ok(Self {
            actor_id,
            msg_type: msg_type.to_u32(),
            payload: encoded,
        })
    }

    pub fn decode_payload<T: serde::de::DeserializeOwned>(&self) -> crate::error::Result<T> {
        Ok(bincode::deserialize(&self.payload)?)
    }
}
