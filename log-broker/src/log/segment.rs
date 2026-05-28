use std::fs::{self, File, OpenOptions};
use std::io::{BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use crate::error::{BrokerError, BrokerResult};

const DEFAULT_MAX_SEGMENT_SIZE: u64 = 512 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct SegmentConfig {
    pub max_size: u64,
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            max_size: DEFAULT_MAX_SEGMENT_SIZE,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MessageHeader {
    pub offset: u64,
    pub key_len: u32,
    pub value_len: u32,
}

pub struct SegmentFile {
    path: PathBuf,
    pub base_offset: u64,
    file: BufWriter<File>,
    pub size: u64,
    pub message_count: u64,
    pub sealed: bool,
    config: SegmentConfig,
}

impl SegmentFile {
    pub fn create(dir: &Path, base_offset: u64, config: SegmentConfig) -> BrokerResult<Self> {
        fs::create_dir_all(dir).map_err(BrokerError::from)?;

        let filename = format!("{:020}.log", base_offset);
        let path = dir.join(&filename);

        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)
            .map_err(BrokerError::from)?;

        Ok(Self {
            path,
            base_offset,
            file: BufWriter::with_capacity(64 * 1024, file),
            size: 0,
            message_count: 0,
            sealed: false,
            config,
        })
    }

    pub fn open(dir: &Path, base_offset: u64, config: SegmentConfig) -> BrokerResult<Self> {
        let filename = format!("{:020}.log", base_offset);
        let path = dir.join(&filename);

        let file = OpenOptions::new()
            .read(true)
            .append(true)
            .open(&path)
            .map_err(BrokerError::from)?;

        let metadata = file.metadata().map_err(BrokerError::from)?;
        let file_size = metadata.len();

        let sealed = file_size >= config.max_size;

        Ok(Self {
            path,
            base_offset,
            file: BufWriter::with_capacity(64 * 1024, file),
            size: file_size,
            message_count: 0,
            sealed,
            config,
        })
    }

    pub fn append(&mut self, offset: u64, key: &[u8], value: &[u8]) -> BrokerResult<u64> {
        if self.sealed {
            return Err(BrokerError::InvalidArgument("segment is sealed".into()));
        }

        let key_len = key.len() as u32;
        let value_len = value.len() as u32;
        let total_message_size = 20 + key.len() + value.len();

        let mut header_buf = [0u8; 20];
        let crc_input_length = 16 + key.len() + value.len();

        let offset_bytes = offset.to_be_bytes();
        let key_len_bytes = key_len.to_be_bytes();
        let value_len_bytes = value_len.to_be_bytes();

        header_buf[4..12].copy_from_slice(&offset_bytes);
        header_buf[12..16].copy_from_slice(&key_len_bytes);
        header_buf[16..20].copy_from_slice(&value_len_bytes);

        let crc = compute_crc32c(&header_buf[4..], key, value, crc_input_length);
        header_buf[0..4].copy_from_slice(&crc.to_be_bytes());

        self.file
            .write_all(&header_buf)
            .map_err(BrokerError::from)?;
        self.file.write_all(key).map_err(BrokerError::from)?;
        self.file.write_all(value).map_err(BrokerError::from)?;
        self.file.flush().map_err(BrokerError::from)?;

        let pos_before = self.size;
        self.size += total_message_size as u64;
        self.message_count += 1;

        if self.size >= self.config.max_size {
            self.sealed = true;
        }

        Ok(pos_before)
    }

    pub fn read_message_at(
        &self,
        position: u64,
    ) -> BrokerResult<(MessageHeader, Vec<u8>, Vec<u8>)> {
        let mut file = File::open(&self.path).map_err(BrokerError::from)?;
        file.seek(SeekFrom::Start(position))
            .map_err(BrokerError::from)?;

        let mut header_buf = [0u8; 20];
        file.read_exact(&mut header_buf)
            .map_err(BrokerError::from)?;

        let crc_stored =
            u32::from_be_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
        let offset = u64::from_be_bytes(
            header_buf[4..12]
                .try_into()
                .map_err(|_| BrokerError::CorruptData("invalid offset bytes".into()))?,
        );

        let key_len = u32::from_be_bytes(
            header_buf[12..16]
                .try_into()
                .map_err(|_| BrokerError::CorruptData("invalid key_len bytes".into()))?,
        );

        let value_len = u32::from_be_bytes(
            header_buf[16..20]
                .try_into()
                .map_err(|_| BrokerError::CorruptData("invalid value_len bytes".into()))?,
        );

        let mut key = vec![0u8; key_len as usize];
        let mut value = vec![0u8; value_len as usize];

        file.read_exact(&mut key).map_err(BrokerError::from)?;
        file.read_exact(&mut value).map_err(BrokerError::from)?;

        let actual_crc =
            compute_crc32c(&header_buf[4..], &key, &value, 16 + key.len() + value.len());
        if actual_crc != crc_stored {
            return Err(BrokerError::CorruptData(format!(
                "CRC mismatch at offset {}: stored={:08x} computed={:08x}",
                offset, crc_stored, actual_crc
            )));
        }

        Ok((
            MessageHeader {
                offset,
                key_len,
                value_len,
            },
            key,
            value,
        ))
    }

    pub fn scan_for_rebuild(
        dir: &Path,
        base_offset: u64,
    ) -> BrokerResult<Vec<(MessageHeader, u64)>> {
        let filename = format!("{:020}.log", base_offset);
        let path = dir.join(&filename);

        if !path.exists() {
            return Ok(Vec::new());
        }

        let mut file = File::open(&path).map_err(BrokerError::from)?;
        let file_size = file.metadata().map_err(BrokerError::from)?.len();
        let mut entries = Vec::new();
        let mut pos: u64 = 0;

        while pos + 20 <= file_size {
            file.seek(SeekFrom::Start(pos)).map_err(BrokerError::from)?;

            let mut header_buf = [0u8; 20];
            if file.read_exact(&mut header_buf).is_err() {
                break;
            }

            let crc_stored =
                u32::from_be_bytes([header_buf[0], header_buf[1], header_buf[2], header_buf[3]]);
            let offset = u64::from_be_bytes(
                header_buf[4..12]
                    .try_into()
                    .map_err(|_| BrokerError::CorruptData("invalid offset".into()))?,
            );

            let key_len = u32::from_be_bytes(
                header_buf[12..16]
                    .try_into()
                    .map_err(|_| BrokerError::CorruptData("invalid key_len".into()))?,
            );

            let value_len = u32::from_be_bytes(
                header_buf[16..20]
                    .try_into()
                    .map_err(|_| BrokerError::CorruptData("invalid value_len".into()))?,
            );

            let msg_size = 20u64 + key_len as u64 + value_len as u64;

            if pos + msg_size > file_size {
                break;
            }

            let mut key = vec![0u8; key_len as usize];
            let mut value = vec![0u8; value_len as usize];
            file.read_exact(&mut key).map_err(BrokerError::from)?;
            file.read_exact(&mut value).map_err(BrokerError::from)?;

            let actual_crc =
                compute_crc32c(&header_buf[4..], &key, &value, 16 + key.len() + value.len());
            if actual_crc == crc_stored {
                entries.push((
                    MessageHeader {
                        offset,
                        key_len,
                        value_len,
                    },
                    pos,
                ));
            }

            pos += msg_size;
        }

        Ok(entries)
    }

    pub fn total_size_on_disk(&self) -> BrokerResult<u64> {
        let metadata = fs::metadata(&self.path).map_err(BrokerError::from)?;
        Ok(metadata.len())
    }
}

fn compute_crc32c(header: &[u8], key: &[u8], value: &[u8], input_len: usize) -> u32 {
    let mut crc: u32 = 0xFFFF_FFFF;
    let polynomial: u32 = 0x1EDC_6F41;

    let mut bytes_processed = 0;

    for &byte in header.iter().take(16) {
        if bytes_processed >= input_len {
            break;
        }
        crc = update_crc32c_byte(crc, byte, polynomial);
        bytes_processed += 1;
    }

    for &byte in key.iter() {
        if bytes_processed >= input_len {
            break;
        }
        crc = update_crc32c_byte(crc, byte, polynomial);
        bytes_processed += 1;
    }

    for &byte in value.iter() {
        if bytes_processed >= input_len {
            break;
        }
        crc = update_crc32c_byte(crc, byte, polynomial);
        bytes_processed += 1;
    }

    !crc
}

fn update_crc32c_byte(crc: u32, byte: u8, polynomial: u32) -> u32 {
    let mut crc = crc ^ (byte as u32);
    for _ in 0..8 {
        if (crc & 1) != 0 {
            crc = (crc >> 1) ^ polynomial;
        } else {
            crc >>= 1;
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_append_and_read() {
        let dir = tempfile::tempdir().unwrap();
        let config = SegmentConfig::default();
        let mut seg = SegmentFile::create(dir.path(), 0, config).unwrap();

        seg.append(0, b"key1", b"value1").unwrap();
        seg.append(1, b"key2", b"longer-value-data").unwrap();

        assert_eq!(seg.message_count, 2);

        let (hdr, key, value) = seg.read_message_at(0).unwrap();
        assert_eq!(hdr.offset, 0);
        assert_eq!(key, b"key1");
        assert_eq!(value, b"value1");

        let first_msg_size = 20 + 4 + 6;
        let (hdr2, key2, value2) = seg.read_message_at(first_msg_size as u64).unwrap();
        assert_eq!(hdr2.offset, 1);
        assert_eq!(key2, b"key2");
        assert_eq!(value2, b"longer-value-data");
    }

    #[test]
    fn test_scan_rebuild() {
        let dir = tempfile::tempdir().unwrap();
        let config = SegmentConfig::default();
        let mut seg = SegmentFile::create(dir.path(), 0, config).unwrap();

        seg.append(10, b"a", b"1").unwrap();
        seg.append(11, b"b", b"22").unwrap();
        seg.append(12, b"ccc", b"333").unwrap();

        let entries = SegmentFile::scan_for_rebuild(dir.path(), 0).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].0.offset, 10);
        assert_eq!(entries[1].0.offset, 11);
        assert_eq!(entries[2].0.offset, 12);
    }

    #[test]
    fn test_crc_detects_corruption() {
        let dir = tempfile::tempdir().unwrap();
        let config = SegmentConfig::default();
        let mut seg = SegmentFile::create(dir.path(), 0, config.clone()).unwrap();
        seg.append(0, b"key", b"value").unwrap();

        let path = dir.path().join("00000000000000000000.log");
        let mut bytes = std::fs::read(&path).unwrap();
        bytes[0] ^= 0xFF;
        std::fs::write(&path, &bytes).unwrap();

        let seg2 = SegmentFile::open(dir.path(), 0, config).unwrap();
        let result = seg2.read_message_at(0);
        assert!(result.is_err());
    }
}
