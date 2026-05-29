use crate::error::{ProtocolError, ProtocolResult};

pub const MAGIC_BYTES: u32 = 0xCAFE_BEEF;
pub const FRAME_HEADER_SIZE: usize = 30;
pub const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;
pub const PROTOCOL_VERSION: u16 = 1;

pub struct Frame {
    pub msg_type: u32,
    pub trace_id: u128,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(msg_type: u32, trace_id: u128, payload: Vec<u8>) -> Self {
        Self { msg_type, trace_id, payload }
    }

    pub fn from_message_type(mt: crate::message::MessageType, trace_id: u128, payload: Vec<u8>) -> Self {
        Self { msg_type: mt.to_u32(), trace_id, payload }
    }

    pub fn encode(&self) -> ProtocolResult<Vec<u8>> {
        let total_size = FRAME_HEADER_SIZE + self.payload.len();
        if total_size > MAX_FRAME_SIZE {
            return Err(ProtocolError::PayloadTooLarge(total_size));
        }

        let mut buf = Vec::with_capacity(total_size);
        buf.extend_from_slice(&MAGIC_BYTES.to_be_bytes());
        buf.extend_from_slice(&(total_size as u32).to_be_bytes());
        buf.extend_from_slice(&PROTOCOL_VERSION.to_be_bytes());
        buf.extend_from_slice(&self.msg_type.to_be_bytes());
        buf.extend_from_slice(&self.trace_id.to_be_bytes());
        buf.extend_from_slice(&self.payload);
        Ok(buf)
    }
}

pub struct FrameDecoder {
    buffer: Vec<u8>,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self { buffer: Vec::with_capacity(65536) }
    }

    pub fn feed_bytes(&mut self, data: &[u8]) -> ProtocolResult<Vec<Frame>> {
        self.buffer.extend_from_slice(data);

        if self.buffer.len() > MAX_FRAME_SIZE * 2 {
            self.buffer.clear();
            return Err(ProtocolError::InvalidFrame("buffer overflow".into()));
        }

        let mut frames = Vec::new();
        let mut offset = 0usize;

        while offset + FRAME_HEADER_SIZE <= self.buffer.len() {
            let magic = u32::from_be_bytes([
                self.buffer[offset], self.buffer[offset+1],
                self.buffer[offset+2], self.buffer[offset+3],
            ]);

            if magic != MAGIC_BYTES {
                self.buffer.clear();
                return Err(ProtocolError::InvalidFrame(format!("bad magic: {:08x}", magic)));
            }

            let total_len = u32::from_be_bytes([
                self.buffer[offset+4], self.buffer[offset+5],
                self.buffer[offset+6], self.buffer[offset+7],
            ]) as usize;

            if total_len < FRAME_HEADER_SIZE || total_len > MAX_FRAME_SIZE {
                self.buffer.clear();
                return Err(ProtocolError::InvalidFrame(format!("bad length: {}", total_len)));
            }

            if offset + total_len > self.buffer.len() {
                break;
            }

            let version = u16::from_be_bytes([self.buffer[offset+8], self.buffer[offset+9]]);
            let msg_type = u32::from_be_bytes([
                self.buffer[offset+10], self.buffer[offset+11],
                self.buffer[offset+12], self.buffer[offset+13],
            ]);
            let trace_id = u128::from_be_bytes([
                self.buffer[offset+14], self.buffer[offset+15],
                self.buffer[offset+16], self.buffer[offset+17],
                self.buffer[offset+18], self.buffer[offset+19],
                self.buffer[offset+20], self.buffer[offset+21],
                self.buffer[offset+22], self.buffer[offset+23],
                self.buffer[offset+24], self.buffer[offset+25],
                self.buffer[offset+26], self.buffer[offset+27],
                self.buffer[offset+28], self.buffer[offset+29],
            ]);

            if version != PROTOCOL_VERSION {
                self.buffer.clear();
                return Err(ProtocolError::InvalidFrame(format!("bad version: {}", version)));
            }

            let payload = self.buffer[offset+FRAME_HEADER_SIZE..offset+total_len].to_vec();
            frames.push(Frame { msg_type, trace_id, payload });
            offset += total_len;
        }

        if offset > 0 {
            self.buffer.drain(..offset);
            self.buffer.shrink_to(65536);
        }

        Ok(frames)
    }
}

impl Default for FrameDecoder {
    fn default() -> Self { Self::new() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_roundtrip() {
        let frame = Frame::new(42, 12345, b"hello".to_vec());
        let encoded = frame.encode().unwrap();
        let mut decoder = FrameDecoder::new();
        let decoded = decoder.feed_bytes(&encoded).unwrap();
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].msg_type, 42);
        assert_eq!(decoded[0].trace_id, 12345);
        assert_eq!(decoded[0].payload, b"hello");
    }
}
