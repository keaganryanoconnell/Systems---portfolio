use crate::bitstream::{BitReader, BitWriter};
use crate::error::Result;
use crate::protocol::SensorPoint;

const BLOCK_SIZE: usize = 128;

pub struct GorillaCompressor {
    block_buffer: Vec<SensorPoint>,
}

impl GorillaCompressor {
    pub fn new() -> Self {
        Self { block_buffer: Vec::with_capacity(BLOCK_SIZE) }
    }

    pub fn ingest(&mut self, points: &[SensorPoint]) -> Option<Vec<u8>> {
        let mut output = None;
        for point in points {
            self.block_buffer.push(*point);
            if self.block_buffer.len() >= BLOCK_SIZE {
                output = Some(self.compress_block());
            }
        }
        output
    }

    pub fn flush(&mut self) -> Option<Vec<u8>> {
        if self.block_buffer.is_empty() {
            None
        } else {
            Some(self.compress_block())
        }
    }

    fn compress_block(&mut self) -> Vec<u8> {
        let block = std::mem::take(&mut self.block_buffer);
        self.block_buffer.clear();

        let mut writer = BitWriter::new(BLOCK_SIZE * 2);

        let first = &block[0];

        for i in 0..16 {
            let byte = (first.meter_id >> (8 * (15 - i))) as u8;
            writer.write_bits(byte as u64, 8);
        }

        writer.write_bits(first.timestamp_us, 64);
        writer.write_bits(first.sensor_value.to_bits(), 64);

        let mut prev_timestamp = first.timestamp_us;
        let mut prev_value = first.sensor_value.to_bits();

        for i in 1..block.len() {
            let p = &block[i];

            let delta = p.timestamp_us.wrapping_sub(prev_timestamp);
            prev_timestamp = p.timestamp_us;

            if delta == 0 {
                writer.write_bit(false);
            } else if delta < 1_000_000_000 {
                writer.write_bit(true);
                writer.write_bit(false);
                writer.write_bits(delta, 30);
            } else {
                writer.write_bit(true);
                writer.write_bit(true);
                writer.write_bits(delta, 64);
            }

            let current_value = p.sensor_value.to_bits();
            let xor = current_value ^ prev_value;
            prev_value = current_value;

            if xor == 0 {
                writer.write_bit(false);
            } else {
                writer.write_bit(true);

                let leading = xor.leading_zeros() as u8;
                let trailing = xor.trailing_zeros() as u8;
                let meaning = 64u8.saturating_sub(leading).saturating_sub(trailing);

                writer.write_bits(leading as u64, 6);
                writer.write_bits(meaning as u64, 6);
                writer.write_bits(xor >> trailing, meaning);
            }
        }

        writer.finish_bytes()
    }

    pub fn decompress_block(data: &[u8], expected_count: usize) -> Result<Vec<SensorPoint>> {
        let mut reader = BitReader::new_from_bytes(data);
        let mut points = Vec::with_capacity(expected_count);

        if data.len() < 1 {
            return Ok(points);
        }

        let mut meter_id: u128 = 0;
        for _ in 0..16 {
            meter_id = (meter_id << 8) | reader.read_bits(8) as u128;
        }

        let timestamp_us = reader.read_bits(64);
        let value_bits = reader.read_bits(64);
        let sensor_value = f64::from_bits(value_bits);
        points.push(SensorPoint::new(meter_id, timestamp_us, 0, sensor_value));

        let mut prev_timestamp = timestamp_us;
        let mut prev_value = value_bits;

        for _ in 1..expected_count {
            if reader.word_idx >= reader.words.len() {
                break;
            }

            let delta: u64;
            if !reader.read_bit() {
                delta = 0;
            } else if !reader.read_bit() {
                delta = reader.read_bits(30);
            } else {
                delta = reader.read_bits(64);
            }

            let timestamp = prev_timestamp.wrapping_add(delta);
            prev_timestamp = timestamp;

            let current_value: u64;
            if !reader.read_bit() {
                current_value = prev_value;
            } else {
                let leading = reader.read_bits(6) as u8;
                let meaning = reader.read_bits(6) as u8;
                if meaning == 0 {
                    current_value = prev_value;
                } else {
                    let trailing = 64u8.saturating_sub(leading).saturating_sub(meaning);
                    let xor = reader.read_bits(meaning) << trailing;
                    current_value = prev_value ^ xor;
                }
            }

            prev_value = current_value;
            let sv = f64::from_bits(current_value);
            points.push(SensorPoint::new(meter_id, timestamp, 0, sv));
        }

        Ok(points)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let mut comp = GorillaCompressor::new();
        let mut points = Vec::new();

        for i in 0..BLOCK_SIZE {
            points.push(SensorPoint::new(
                0xDEADBEEF_CAFEu128,
                1_717_012_345_000_000 + (i as u64 * 1_000_000),
                1,
                234.567 + (i as f64 * 0.001),
            ));
        }

        let compressed = comp.ingest(&points).unwrap();
        let decompressed = GorillaCompressor::decompress_block(&compressed, BLOCK_SIZE).unwrap();

        assert_eq!(decompressed.len(), BLOCK_SIZE);

        let raw_size = points.len() * 32;
        let ratio = raw_size as f64 / compressed.len() as f64;
        println!("Compressed {} points: {} bytes → {} bytes ({:.1}:1 ratio)",
            BLOCK_SIZE, raw_size, compressed.len(), ratio);
        assert!(ratio > 3.0, "Expected >3:1 compression, got {:.1}:1", ratio);
    }
}
