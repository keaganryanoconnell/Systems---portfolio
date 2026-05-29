//! SSTable - Sorted String Table Disk Storage
//!
//! Handles writing sorted MemTable records to structured binary files on disk,
//! reading entries using an in-memory binary-searchable index, and loading
//! sequential entries for compaction processing.

use std::fs::File;
use std::io::{self, BufReader, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

/// Magic footer suffix to identify valid LSM SSTable files (8 bytes).
const MAGIC_BYTES: &[u8; 8] = b"LSMSSTAB";

const MAX_KEY_LEN: usize = 16 * 1024 * 1024;
const MAX_VALUE_LEN: usize = 16 * 1024 * 1024;

/// Writes sorted key-value pairs into a binary SSTable file.
pub struct SstableWriter;

impl SstableWriter {
    /// Writes a sorted sequence of entries to a file, compiling data blocks,
    /// index blocks, and footer.
    pub fn write_to_file<I, P>(path: P, entries: I) -> io::Result<()>
    where
        P: AsRef<Path>,
        I: IntoIterator<Item = crate::storage::KeyValuePair>,
    {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);
        let mut index = Vec::new();
        let mut current_offset = 0u64;

        for (key, val_opt) in entries {
            let key_len = key.len() as u32;
            let is_tombstone = val_opt.is_none();
            let val_len = val_opt.as_ref().map_or(0u32, |v| v.len() as u32);

            // Store index reference (start of record)
            index.push((key.clone(), current_offset));

            // Write data header
            writer.write_all(&key_len.to_be_bytes())?;
            writer.write_all(&val_len.to_be_bytes())?;
            writer.write_all(&[is_tombstone as u8])?;
            writer.write_all(&key)?;
            current_offset += 4 + 4 + 1 + key_len as u64;

            if let Some(val) = val_opt {
                writer.write_all(&val)?;
                current_offset += val.len() as u64;
            }
        }

        // Write index block
        let index_offset = current_offset;
        let num_keys = index.len() as u32;

        for (key, offset) in index {
            let key_len = key.len() as u32;
            writer.write_all(&key_len.to_be_bytes())?;
            writer.write_all(&key)?;
            writer.write_all(&offset.to_be_bytes())?;
        }

        // Write footer
        writer.write_all(&index_offset.to_be_bytes())?;
        writer.write_all(&num_keys.to_be_bytes())?;
        writer.write_all(MAGIC_BYTES)?;

        writer.flush()?;
        Ok(())
    }
}

use std::sync::Mutex;

/// Reads sorted key-value records from an immutable SSTable file.
pub struct SstableReader {
    file_path: PathBuf,
    file: Mutex<File>,
    index: Vec<(Vec<u8>, u64)>,
}

impl SstableReader {
    /// Opens an existing SSTable file, reads footer, and parses the index block into memory.
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let mut file = File::open(&path)?;
        let file_len = file.metadata()?.len();

        // Footer is exactly 20 bytes (8 bytes offset + 4 bytes key_count + 8 bytes magic)
        if file_len < 20 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SSTable file is corrupted or too small",
            ));
        }

        file.seek(SeekFrom::End(-20))?;
        let mut footer = [0u8; 20];
        file.read_exact(&mut footer)?;

        let mut offset_bytes = [0u8; 8];
        offset_bytes.copy_from_slice(&footer[0..8]);
        let index_offset = u64::from_be_bytes(offset_bytes);

        let mut count_bytes = [0u8; 4];
        count_bytes.copy_from_slice(&footer[8..12]);
        let num_keys = u32::from_be_bytes(count_bytes);

        if &footer[12..20] != MAGIC_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "SSTable footer magic bytes mismatch",
            ));
        }

        // Parse the index block
        file.seek(SeekFrom::Start(index_offset))?;
        let mut index_reader = BufReader::new(file);

        let mut index = Vec::with_capacity(num_keys as usize);
        for _ in 0..num_keys {
            let mut len_bytes = [0u8; 4];
            index_reader.read_exact(&mut len_bytes)?;
            let key_len = u32::from_be_bytes(len_bytes) as usize;

            if key_len > MAX_KEY_LEN {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    format!("key_len {} exceeds MAX_KEY_LEN", key_len),
                ));
            }

            let mut key = vec![0u8; key_len];
            index_reader.read_exact(&mut key)?;

            let mut offset_bytes = [0u8; 8];
            index_reader.read_exact(&mut offset_bytes)?;
            let data_offset = u64::from_be_bytes(offset_bytes);

            index.push((key, data_offset));
        }

        let original_file = index_reader.into_inner();

        Ok(Self {
            file_path: path.as_ref().to_path_buf(),
            file: Mutex::new(original_file),
            index,
        })
    }

    /// Queries the SSTable for a specific key using binary search.
    ///
    /// Returns:
    /// * `Ok(Some(Some(value)))` if the key exists with a value.
    /// * `Ok(Some(None))` if the key is marked with a tombstone (deleted).
    /// * `Ok(None)` if the key does not exist in this SSTable.
    pub fn get(&self, key: &[u8]) -> io::Result<Option<Option<Vec<u8>>>> {
        let result = self.index.binary_search_by(|(k, _)| k.as_slice().cmp(key));

        let idx = match result {
            Ok(i) => i,
            Err(_) => return Ok(None),
        };

        let data_offset = self.index[idx].1;

        let mut file = self.file.lock().unwrap_or_else(|e| e.into_inner());
        file.seek(SeekFrom::Start(data_offset))?;

        let mut key_len_bytes = [0u8; 4];
        file.read_exact(&mut key_len_bytes)?;
        let key_len = u32::from_be_bytes(key_len_bytes);

        if key_len > i64::MAX as u32 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "key_len exceeds maximum supported value",
            ));
        }

        let mut val_len_bytes = [0u8; 4];
        file.read_exact(&mut val_len_bytes)?;
        let val_len = u32::from_be_bytes(val_len_bytes);

        if val_len as usize > MAX_VALUE_LEN {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!("val_len {} exceeds MAX_VALUE_LEN", val_len),
            ));
        }

        let mut tombstone_byte = [0u8; 1];
        file.read_exact(&mut tombstone_byte)?;
        let is_tombstone = tombstone_byte[0] != 0;

        // Skip the key bytes to reach the value bytes
        file.seek(SeekFrom::Current(key_len as i64))?;

        if is_tombstone {
            Ok(Some(None))
        } else {
            let mut val = vec![0u8; val_len as usize];
            file.read_exact(&mut val)?;
            Ok(Some(Some(val)))
        }
    }

    /// Returns the number of keys inside the SSTable index.
    pub fn len(&self) -> usize {
        self.index.len()
    }

    /// Returns true if the SSTable index is empty.
    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }

    /// Returns the smallest key in the SSTable.
    pub fn min_key(&self) -> Option<&[u8]> {
        self.index.first().map(|(k, _)| k.as_slice())
    }

    /// Returns the largest key in the SSTable.
    pub fn max_key(&self) -> Option<&[u8]> {
        self.index.last().map(|(k, _)| k.as_slice())
    }

    /// Reads all key-value entries sequentially from the SSTable file.
    /// This is highly optimized for sequential disk streaming during compaction.
    pub fn all_entries(&self) -> io::Result<Vec<crate::storage::KeyValuePair>> {
        let file = File::open(&self.file_path)?;
        let mut reader = BufReader::new(file);
        let mut entries = Vec::with_capacity(self.index.len());

        for _ in 0..self.index.len() {
            let mut len_bytes = [0u8; 4];
            reader.read_exact(&mut len_bytes)?;
            let key_len = u32::from_be_bytes(len_bytes) as usize;

            let mut val_len_bytes = [0u8; 4];
            reader.read_exact(&mut val_len_bytes)?;
            let val_len = u32::from_be_bytes(val_len_bytes) as usize;

            let mut tombstone_byte = [0u8; 1];
            reader.read_exact(&mut tombstone_byte)?;
            let is_tombstone = tombstone_byte[0] != 0;

            let mut key = vec![0u8; key_len];
            reader.read_exact(&mut key)?;

            let value = if is_tombstone {
                None
            } else {
                let mut val = vec![0u8; val_len];
                reader.read_exact(&mut val)?;
                Some(val)
            };

            entries.push((key, value));
        }

        Ok(entries)
    }

    /// Returns the file path of this SSTable.
    pub fn path(&self) -> &Path {
        &self.file_path
    }
}
