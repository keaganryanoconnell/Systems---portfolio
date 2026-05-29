use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use crate::error::{AggregatorError, Result};

const RING_SIZE: usize = 1024;
const FLUSH_THRESHOLD: usize = 512;
const SEGMENT_MAGIC: u32 = 0xDEAD_BEEF;

#[derive(Debug, Clone)]
pub struct SegmentHeader {
    pub magic: u32,
    pub block_count: u32,
    pub timestamp: u64,
}

impl SegmentHeader {
    fn to_bytes(&self) -> [u8; 16] {
        let mut buf = [0u8; 16];
        buf[0..4].copy_from_slice(&self.magic.to_be_bytes());
        buf[4..8].copy_from_slice(&self.block_count.to_be_bytes());
        buf[8..16].copy_from_slice(&self.timestamp.to_be_bytes());
        buf
    }

    fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 16 {
            return None;
        }
        let magic = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
        if magic != SEGMENT_MAGIC {
            return None;
        }
        let block_count = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
        let timestamp = u64::from_be_bytes([
            bytes[8], bytes[9], bytes[10], bytes[11], bytes[12], bytes[13], bytes[14], bytes[15],
        ]);
        Some(Self {
            magic,
            block_count,
            timestamp,
        })
    }
}

pub struct LogBuffer {
    ring: Vec<Vec<u8>>,
    write_pos: usize,
    read_pos: usize,
    disk_path: PathBuf,
    disk_seq: u64,
    memory_cap: usize,
    memory_used: usize,
}

impl LogBuffer {
    pub fn new(memory_cap: usize, disk_path: &Path) -> Self {
        fs::create_dir_all(disk_path).ok();
        let ring: Vec<Vec<u8>> = (0..RING_SIZE).map(|_| Vec::new()).collect();

        Self {
            ring,
            write_pos: 0,
            read_pos: 0,
            disk_path: disk_path.to_path_buf(),
            disk_seq: 0,
            memory_cap,
            memory_used: 0,
        }
    }

    pub fn write(&mut self, block: &[u8]) -> Result<()> {
        self.memory_used -= self.ring[self.write_pos].len();
        self.ring[self.write_pos] = block.to_vec();
        self.memory_used += block.len();

        self.write_pos = (self.write_pos + 1) % RING_SIZE;

        if self.write_pos == self.read_pos {
            self.flush_to_disk()?;
        }

        if self.memory_used > self.memory_cap {
            self.evict_oldest()?;
        }

        Ok(())
    }

    fn flush_to_disk(&mut self) -> Result<()> {
        let blocks_to_flush = (self.write_pos + RING_SIZE - self.read_pos) % RING_SIZE;
        let flush_count = blocks_to_flush.min(FLUSH_THRESHOLD);

        if flush_count == 0 {
            return Ok(());
        }

        let filename = format!("segment_{:020}.bin", self.disk_seq);
        let path = self.disk_path.join(&filename);

        let mut file = fs::File::create(&path)?;

        let header = SegmentHeader {
            magic: SEGMENT_MAGIC,
            block_count: flush_count as u32,
            timestamp: 0,
        };
        file.write_all(&header.to_bytes())?;

        for i in 0..flush_count {
            let idx = (self.read_pos + i) % RING_SIZE;
            let block = &self.ring[idx];
            let len = block.len() as u32;
            file.write_all(&len.to_be_bytes())?;
            file.write_all(block)?;
        }

        file.flush()?;

        self.read_pos = (self.read_pos + flush_count) % RING_SIZE;
        self.disk_seq += 1;

        Ok(())
    }

    fn evict_oldest(&mut self) -> Result<()> {
        self.flush_to_disk()
    }

    pub fn read_segment(&self, segment_num: u64) -> Result<Vec<Vec<u8>>> {
        let filename = format!("segment_{:020}.bin", segment_num);
        let path = self.disk_path.join(&filename);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = fs::File::open(&path)?;
        let mut header_buf = [0u8; 16];
        file.read_exact(&mut header_buf)?;

        let header = SegmentHeader::from_bytes(&header_buf)
            .ok_or_else(|| AggregatorError::InvalidPacket("bad segment header".into()))?;

        let mut blocks = Vec::with_capacity(header.block_count as usize);

        for _ in 0..header.block_count {
            let mut len_buf = [0u8; 4];
            file.read_exact(&mut len_buf)?;
            let len = u32::from_be_bytes(len_buf) as usize;

            let mut block = vec![0u8; len];
            file.read_exact(&mut block)?;
            blocks.push(block);
        }

        Ok(blocks)
    }

    pub fn memory_used(&self) -> usize {
        self.memory_used
    }

    pub fn segment_count(&self) -> u64 {
        self.disk_seq
    }
}
