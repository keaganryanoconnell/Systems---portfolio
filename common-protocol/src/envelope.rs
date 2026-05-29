use serde::{Deserialize, Serialize};

use crate::error::ProtocolResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageEnvelope {
    pub msg_type: u32,
    pub trace_id: u128,
    pub payload: Vec<u8>,
}

impl MessageEnvelope {
    pub fn new(msg_type: u32, trace_id: u128, payload: Vec<u8>) -> Self {
        Self { msg_type, trace_id, payload }
    }

    pub fn from_typed<T: Serialize>(msg_type: u32, trace_id: u128, data: &T) -> ProtocolResult<Self> {
        Ok(Self {
            msg_type,
            trace_id,
            payload: bincode::serialize(data)?,
        })
    }

    pub fn decode<T: serde::de::DeserializeOwned>(&self) -> ProtocolResult<T> {
        Ok(bincode::deserialize(&self.payload)?)
    }

    pub fn into_frame(self) -> crate::frame::Frame {
        crate::frame::Frame::new(self.msg_type, self.trace_id, self.payload)
    }

    pub fn from_frame(frame: crate::frame::Frame) -> Self {
        Self {
            msg_type: frame.msg_type,
            trace_id: frame.trace_id,
            payload: frame.payload,
        }
    }
}
