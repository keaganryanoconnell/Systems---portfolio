use std::io::{Read, Write};
use std::net::TcpStream;

use crate::error::{BrokerError, BrokerResult};
use crate::network::protocol::{Frame, FrameDecoder, ERROR_NONE};

pub type FetchedMessage = (u64, Vec<u8>, Vec<u8>);

pub struct BrokerClient {
    stream: TcpStream,
    decoder: FrameDecoder,
    correlation_id: u32,
}

impl BrokerClient {
    pub fn connect(addr: &str) -> BrokerResult<Self> {
        let stream = TcpStream::connect(addr).map_err(BrokerError::from)?;
        stream.set_nodelay(true).map_err(BrokerError::from)?;

        Ok(Self {
            stream,
            decoder: FrameDecoder::new(),
            correlation_id: 0,
        })
    }

    fn next_correlation_id(&mut self) -> u32 {
        self.correlation_id = self.correlation_id.wrapping_add(1);
        self.correlation_id
    }

    pub fn produce(&mut self, topic: &str, key: &[u8], value: &[u8]) -> BrokerResult<u64> {
        let corr_id = self.next_correlation_id();

        let frame = Frame::produce(corr_id, topic, key, value);
        self.send_and_receive(frame)?;

        let response = self.read_response()?;

        if response.body.len() < 12 {
            return Err(BrokerError::InvalidFrame(
                "produce response too short".into(),
            ));
        }

        let error_code = u32::from_be_bytes([
            response.body[0],
            response.body[1],
            response.body[2],
            response.body[3],
        ]);

        if error_code != ERROR_NONE {
            return Err(BrokerError::InvalidFrame(format!(
                "produce error: code={}",
                error_code
            )));
        }

        let offset = u64::from_be_bytes([
            response.body[4],
            response.body[5],
            response.body[6],
            response.body[7],
            response.body[8],
            response.body[9],
            response.body[10],
            response.body[11],
        ]);

        Ok(offset)
    }

    pub fn fetch(
        &mut self,
        topic: &str,
        start_offset: u64,
        max_bytes: u32,
    ) -> BrokerResult<Vec<FetchedMessage>> {
        let corr_id = self.next_correlation_id();

        let frame = Frame::fetch(corr_id, topic, start_offset, max_bytes);
        self.send_and_receive(frame)?;

        let response = self.read_response()?;

        if response.body.len() < 12 {
            return Err(BrokerError::InvalidFrame("fetch response too short".into()));
        }

        let error_code = u32::from_be_bytes([
            response.body[0],
            response.body[1],
            response.body[2],
            response.body[3],
        ]);

        if error_code != ERROR_NONE {
            return Err(BrokerError::InvalidFrame(format!(
                "fetch error: code={}",
                error_code
            )));
        }

        let msg_count = u32::from_be_bytes([
            response.body[4],
            response.body[5],
            response.body[6],
            response.body[7],
        ]) as usize;

        let mut messages = Vec::with_capacity(msg_count);
        let mut pos = 8;

        for _ in 0..msg_count {
            if pos + 16 > response.body.len() {
                break;
            }

            let offset = u64::from_be_bytes([
                response.body[pos],
                response.body[pos + 1],
                response.body[pos + 2],
                response.body[pos + 3],
                response.body[pos + 4],
                response.body[pos + 5],
                response.body[pos + 6],
                response.body[pos + 7],
            ]);
            pos += 8;

            let key_len = u32::from_be_bytes([
                response.body[pos],
                response.body[pos + 1],
                response.body[pos + 2],
                response.body[pos + 3],
            ]) as usize;
            pos += 4;

            let value_len = u32::from_be_bytes([
                response.body[pos],
                response.body[pos + 1],
                response.body[pos + 2],
                response.body[pos + 3],
            ]) as usize;
            pos += 4;

            if pos + key_len + value_len > response.body.len() {
                break;
            }

            let key = response.body[pos..pos + key_len].to_vec();
            pos += key_len;

            let value = response.body[pos..pos + value_len].to_vec();
            pos += value_len;

            messages.push((offset, key, value));
        }

        Ok(messages)
    }

    pub fn list_offsets(&mut self, topic: &str) -> BrokerResult<(u64, u64)> {
        let corr_id = self.next_correlation_id();

        let frame = Frame::list_offsets(corr_id, topic);
        self.send_and_receive(frame)?;

        let response = self.read_response()?;

        if response.body.len() < 20 {
            return Err(BrokerError::InvalidFrame(
                "list_offsets response too short".into(),
            ));
        }

        let error_code = u32::from_be_bytes([
            response.body[0],
            response.body[1],
            response.body[2],
            response.body[3],
        ]);

        if error_code != ERROR_NONE {
            return Err(BrokerError::InvalidFrame(format!(
                "list_offsets error: code={}",
                error_code
            )));
        }

        let earliest = u64::from_be_bytes([
            response.body[4],
            response.body[5],
            response.body[6],
            response.body[7],
            response.body[8],
            response.body[9],
            response.body[10],
            response.body[11],
        ]);

        let latest = u64::from_be_bytes([
            response.body[12],
            response.body[13],
            response.body[14],
            response.body[15],
            response.body[16],
            response.body[17],
            response.body[18],
            response.body[19],
        ]);

        Ok((earliest, latest))
    }

    pub fn commit(&mut self, topic: &str, offset: u64) -> BrokerResult<()> {
        let corr_id = self.next_correlation_id();

        let frame = Frame::commit(corr_id, topic, offset);
        self.send_and_receive(frame)?;

        let response = self.read_response()?;

        if response.body.len() < 4 {
            return Err(BrokerError::InvalidFrame(
                "commit response too short".into(),
            ));
        }

        let error_code = u32::from_be_bytes([
            response.body[0],
            response.body[1],
            response.body[2],
            response.body[3],
        ]);

        if error_code != ERROR_NONE {
            return Err(BrokerError::InvalidFrame(format!(
                "commit error: code={}",
                error_code
            )));
        }

        Ok(())
    }

    fn send_and_receive(&mut self, frame: Frame) -> BrokerResult<()> {
        let encoded = frame.encode()?;
        self.stream.write_all(&encoded).map_err(BrokerError::from)?;
        self.stream.flush().map_err(BrokerError::from)?;
        Ok(())
    }

    fn read_response(&mut self) -> BrokerResult<Frame> {
        let mut buf = [0u8; 65536];

        loop {
            let n = self.stream.read(&mut buf).map_err(BrokerError::from)?;
            if n == 0 {
                return Err(BrokerError::ConnectionClosed);
            }

            let frames = self.decoder.feed_bytes(&buf[..n])?;
            if let Some(frame) = frames.into_iter().next() {
                return Ok(frame);
            }
        }
    }
}
