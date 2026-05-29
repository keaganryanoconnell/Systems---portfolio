use std::io::Write;

use crate::error::{BrokerError, BrokerResult};

pub const MAGIC_BYTES: u32 = 0xCAFEBABE;
pub const FRAME_HEADER_SIZE: usize = 16;
const MAX_FRAME_SIZE: usize = 16 * 1024 * 1024;

pub const API_PRODUCE: u16 = 0;
pub const API_FETCH: u16 = 1;
pub const API_LIST_OFFSETS: u16 = 2;
pub const API_COMMIT: u16 = 3;

pub const ERROR_NONE: u32 = 0;
pub const ERROR_TOPIC_NOT_FOUND: u32 = 1;
pub const ERROR_OFFSET_OUT_OF_RANGE: u32 = 2;
pub const ERROR_CORRUPT_DATA: u32 = 3;
pub const ERROR_INTERNAL: u32 = 4;

#[derive(Debug)]
pub struct Frame {
    pub api_key: u16,
    pub api_version: u16,
    pub correlation_id: u32,
    pub body: Vec<u8>,
}

impl Frame {
    pub fn produce(correlation_id: u32, topic: &str, key: &[u8], value: &[u8]) -> Self {
        let topic_bytes = topic.as_bytes();
        let mut body = Vec::with_capacity(2 + topic_bytes.len() + 12 + key.len() + value.len());
        body.extend_from_slice(&(topic_bytes.len() as u16).to_be_bytes());
        body.extend_from_slice(topic_bytes);
        body.extend_from_slice(&(key.len() as u32).to_be_bytes());
        body.extend_from_slice(&(value.len() as u32).to_be_bytes());
        body.extend_from_slice(key);
        body.extend_from_slice(value);

        Self {
            api_key: API_PRODUCE,
            api_version: 1,
            correlation_id,
            body,
        }
    }

    pub fn fetch(correlation_id: u32, topic: &str, start_offset: u64, max_bytes: u32) -> Self {
        let topic_bytes = topic.as_bytes();
        let mut body = Vec::with_capacity(2 + topic_bytes.len() + 16);
        body.extend_from_slice(&(topic_bytes.len() as u16).to_be_bytes());
        body.extend_from_slice(topic_bytes);
        body.extend_from_slice(&start_offset.to_be_bytes());
        body.extend_from_slice(&max_bytes.to_be_bytes());

        Self {
            api_key: API_FETCH,
            api_version: 1,
            correlation_id,
            body,
        }
    }

    pub fn list_offsets(correlation_id: u32, topic: &str) -> Self {
        let topic_bytes = topic.as_bytes();
        let mut body = Vec::with_capacity(2 + topic_bytes.len() + 1);
        body.extend_from_slice(&(topic_bytes.len() as u16).to_be_bytes());
        body.extend_from_slice(topic_bytes);
        body.push(1);

        Self {
            api_key: API_LIST_OFFSETS,
            api_version: 1,
            correlation_id,
            body,
        }
    }

    pub fn commit(correlation_id: u32, topic: &str, offset: u64) -> Self {
        let topic_bytes = topic.as_bytes();
        let mut body = Vec::with_capacity(2 + topic_bytes.len() + 8);
        body.extend_from_slice(&(topic_bytes.len() as u16).to_be_bytes());
        body.extend_from_slice(topic_bytes);
        body.extend_from_slice(&offset.to_be_bytes());

        Self {
            api_key: API_COMMIT,
            api_version: 1,
            correlation_id,
            body,
        }
    }

    pub fn response(correlation_id: u32, api_key: u16, error_code: u32, body: Vec<u8>) -> Self {
        let mut full_body = Vec::with_capacity(4 + body.len());
        full_body.extend_from_slice(&error_code.to_be_bytes());
        full_body.extend_from_slice(&body);

        Self {
            api_key,
            api_version: 1,
            correlation_id,
            body: full_body,
        }
    }

    pub fn produce_response(correlation_id: u32, error_code: u32, offset: u64) -> Self {
        let body = offset.to_be_bytes().to_vec();
        Self::response(correlation_id, API_PRODUCE, error_code, body)
    }

    pub fn fetch_response(
        correlation_id: u32,
        error_code: u32,
        messages: &[(u64, Vec<u8>, Vec<u8>)],
    ) -> Self {
        let mut body = Vec::new();
        body.extend_from_slice(&(messages.len() as u32).to_be_bytes());

        for (offset, key, value) in messages {
            body.extend_from_slice(&offset.to_be_bytes());
            body.extend_from_slice(&(key.len() as u32).to_be_bytes());
            body.extend_from_slice(&(value.len() as u32).to_be_bytes());
            body.extend_from_slice(key);
            body.extend_from_slice(value);
        }

        Self::response(correlation_id, API_FETCH, error_code, body)
    }

    pub fn list_offsets_response(
        correlation_id: u32,
        error_code: u32,
        earliest: u64,
        latest: u64,
    ) -> Self {
        let mut body = Vec::with_capacity(16);
        body.extend_from_slice(&earliest.to_be_bytes());
        body.extend_from_slice(&latest.to_be_bytes());
        Self::response(correlation_id, API_LIST_OFFSETS, error_code, body)
    }

    pub fn commit_response(correlation_id: u32, error_code: u32) -> Self {
        Self::response(correlation_id, API_COMMIT, error_code, Vec::new())
    }

    pub fn encode(&self) -> BrokerResult<Vec<u8>> {
        let total_size = FRAME_HEADER_SIZE + self.body.len();
        let mut buf = Vec::with_capacity(total_size);

        buf.extend_from_slice(&MAGIC_BYTES.to_be_bytes());
        buf.extend_from_slice(&(total_size as u32).to_be_bytes());
        buf.extend_from_slice(&self.api_key.to_be_bytes());
        buf.extend_from_slice(&self.api_version.to_be_bytes());
        buf.extend_from_slice(&self.correlation_id.to_be_bytes());
        buf.extend_from_slice(&self.body);

        Ok(buf)
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> BrokerResult<usize> {
        let encoded = self.encode()?;
        writer.write_all(&encoded).map_err(BrokerError::from)?;
        Ok(encoded.len())
    }
}

pub struct FrameDecoder {
    buffer: Vec<u8>,
    state: DecodeState,
    expected_size: usize,
    header: Option<ParsedHeader>,
}

struct ParsedHeader {
    api_key: u16,
    api_version: u16,
    correlation_id: u32,
}

#[derive(PartialEq)]
enum DecodeState {
    ReadingHeader,
    ReadingBody,
}

impl FrameDecoder {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(65536),
            state: DecodeState::ReadingHeader,
            expected_size: FRAME_HEADER_SIZE,
            header: None,
        }
    }

    pub fn feed_bytes(&mut self, data: &[u8]) -> BrokerResult<Vec<Frame>> {
        self.buffer.extend_from_slice(data);

        if self.buffer.len() > MAX_FRAME_SIZE * 2 {
            self.buffer.clear();
            self.state = DecodeState::ReadingHeader;
            self.expected_size = FRAME_HEADER_SIZE;
            self.header = None;
            return Err(BrokerError::InvalidFrame(
                "buffer exceeded maximum size".into(),
            ));
        }

        let mut frames = Vec::new();

        loop {
            match self.state {
                DecodeState::ReadingHeader => {
                    if self.buffer.len() < FRAME_HEADER_SIZE {
                        break;
                    }

                    let magic = u32::from_be_bytes([
                        self.buffer[0],
                        self.buffer[1],
                        self.buffer[2],
                        self.buffer[3],
                    ]);

                    if magic != MAGIC_BYTES {
                        let err_msg = format!("invalid magic: {:08x}", magic);
                        self.buffer.clear();
                        self.state = DecodeState::ReadingHeader;
                        self.expected_size = FRAME_HEADER_SIZE;
                        self.header = None;
                        return Err(BrokerError::InvalidFrame(err_msg));
                    }

                    let total_len = u32::from_be_bytes([
                        self.buffer[4],
                        self.buffer[5],
                        self.buffer[6],
                        self.buffer[7],
                    ]) as usize;

                    if total_len > MAX_FRAME_SIZE {
                        self.buffer.clear();
                        self.state = DecodeState::ReadingHeader;
                        self.expected_size = FRAME_HEADER_SIZE;
                        self.header = None;
                        return Err(BrokerError::InvalidFrame(
                            "total_len exceeds MAX_FRAME_SIZE".into(),
                        ));
                    }

                    let api_key = u16::from_be_bytes([self.buffer[8], self.buffer[9]]);
                    let api_version = u16::from_be_bytes([self.buffer[10], self.buffer[11]]);
                    let correlation_id = u32::from_be_bytes([
                        self.buffer[12],
                        self.buffer[13],
                        self.buffer[14],
                        self.buffer[15],
                    ]);

                    if total_len < FRAME_HEADER_SIZE {
                        self.buffer.clear();
                        self.state = DecodeState::ReadingHeader;
                        self.expected_size = FRAME_HEADER_SIZE;
                        self.header = None;
                        return Err(BrokerError::InvalidFrame("total_len too small".into()));
                    }

                    self.header = Some(ParsedHeader {
                        api_key,
                        api_version,
                        correlation_id,
                    });

                    self.state = DecodeState::ReadingBody;
                    self.expected_size = total_len;
                }

                DecodeState::ReadingBody => {
                    let total_len = self.expected_size;

                    if self.buffer.len() < total_len {
                        break;
                    }

                    let header = self
                        .header
                        .take()
                        .ok_or_else(|| BrokerError::InvalidFrame("no header state".into()))?;

                    let body = self.buffer[FRAME_HEADER_SIZE..total_len].to_vec();
                    self.buffer.drain(..total_len);

                    frames.push(Frame {
                        api_key: header.api_key,
                        api_version: header.api_version,
                        correlation_id: header.correlation_id,
                        body,
                    });

                    self.state = DecodeState::ReadingHeader;
                    self.expected_size = FRAME_HEADER_SIZE;
                    self.header = None;
                }
            }
        }

        self.buffer.shrink_to(65536);
        Ok(frames)
    }
}

impl Default for FrameDecoder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_produce_frame_roundtrip() {
        let frame = Frame::produce(42, "test-topic", b"key-1", b"value-1");
        let encoded = frame.encode().unwrap();

        let mut decoder = FrameDecoder::new();
        let decoded = decoder.feed_bytes(&encoded).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].api_key, API_PRODUCE);
        assert_eq!(decoded[0].correlation_id, 42);
    }

    #[test]
    fn test_fetch_response_roundtrip() {
        let messages = vec![
            (0u64, b"k1".to_vec(), b"v1".to_vec()),
            (1u64, b"k2".to_vec(), b"v2".to_vec()),
        ];
        let frame = Frame::fetch_response(100, ERROR_NONE, &messages);
        let encoded = frame.encode().unwrap();

        let mut decoder = FrameDecoder::new();
        let decoded = decoder.feed_bytes(&encoded).unwrap();

        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].api_key, API_FETCH);
    }

    #[test]
    fn test_invalid_magic_rejected() {
        let mut decoder = FrameDecoder::new();
        let garbage = vec![0xFFu8; 16];
        let result = decoder.feed_bytes(&garbage);
        assert!(result.is_err());
    }
}
