//! LSM Storage Engine - Core Module Coordinator
//!
//! Exposes the primary database operations and manages thread-safe interaction
//! between the active MemTable, frozen MemTable, and disk-backed SSTables.

pub mod compactor;
pub mod memtable;
pub mod sstable;

/// A single LSM key-value entry, where None represents a tombstone/delete marker.
pub type KeyValuePair = (Vec<u8>, Option<Vec<u8>>);

use memtable::MemTable;
use sstable::{SstableReader, SstableWriter};

use core_sys::{log_error, log_info};
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

/// Configuration parameters for the LSM Storage Engine.
#[derive(Debug, Clone)]
pub struct LsmConfig {
    /// Directory where SSTables are stored on disk.
    pub data_dir: PathBuf,
    /// Threshold in bytes before flushing MemTable to disk (default 1 MB).
    pub flush_threshold_bytes: usize,
    /// Number of SSTables that triggers a background compaction (default 4).
    pub compaction_trigger_files: usize,
}

impl Default for LsmConfig {
    fn default() -> Self {
        Self {
            data_dir: PathBuf::from("data/lsm_storage"),
            flush_threshold_bytes: 1024 * 1024, // 1 MB
            compaction_trigger_files: 4,
        }
    }
}

struct LsmEngineInner {
    config: LsmConfig,
    active_memtable: RwLock<Arc<MemTable>>,
    frozen_memtable: RwLock<Option<Arc<MemTable>>>,
    sstables: RwLock<Vec<Arc<SstableReader>>>,
    next_sstable_id: AtomicU64,
}

/// The main entry point to the Log-Structured Merge-tree (LSM) Storage Engine.
///
/// Thread-safe and cheap to clone, wrapping the core state in an Arc.
#[derive(Clone)]
pub struct LsmEngine {
    inner: Arc<LsmEngineInner>,
}

impl LsmEngine {
    /// Opens or creates an LSM storage engine instance at the configured path.
    /// Loads any existing SSTables on disk to resume state.
    pub fn open(config: LsmConfig) -> io::Result<Self> {
        // Ensure data directory exists
        fs::create_dir_all(&config.data_dir)?;

        let mut existing_sstables = Vec::new();
        let mut max_id = 0u64;

        // Scan directory for existing SSTables (.db files)
        if let Ok(entries) = fs::read_dir(&config.data_dir) {
            let mut file_ids = Vec::new();
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().is_some_and(|ext| ext == "db") {
                    if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                        if let Ok(id) = stem.parse::<u64>() {
                            file_ids.push((id, path));
                            if id > max_id {
                                max_id = id;
                            }
                        }
                    }
                }
            }

            // Sort descending: newest files (highest IDs) first
            file_ids.sort_by_key(|b| std::cmp::Reverse(b.0));

            for (_, path) in file_ids {
                let reader = SstableReader::open(path)?;
                existing_sstables.push(Arc::new(reader));
            }
        }

        let next_sstable_id = AtomicU64::new(max_id + 1);

        Ok(Self {
            inner: Arc::new(LsmEngineInner {
                config,
                active_memtable: RwLock::new(Arc::new(MemTable::new())),
                frozen_memtable: RwLock::new(None),
                sstables: RwLock::new(existing_sstables),
                next_sstable_id,
            }),
        })
    }

    /// Writes a key-value record to the storage engine.
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), String> {
        let mem = {
            let active = self
                .inner
                .active_memtable
                .read()
                .map_err(|_| "Lock poisoned during put")?;
            active.clone()
        };

        mem.put(key, value).map_err(|e| e.to_string())?;
        self.check_flush(mem)?;
        Ok(())
    }

    /// Deletes a key from the storage engine by inserting a tombstone.
    pub fn delete(&self, key: Vec<u8>) -> Result<(), String> {
        let mem = {
            let active = self
                .inner
                .active_memtable
                .read()
                .map_err(|_| "Lock poisoned during delete")?;
            active.clone()
        };

        mem.delete(key).map_err(|e| e.to_string())?;
        self.check_flush(mem)?;
        Ok(())
    }

    /// Retrieves the value of a key, if it exists and is not deleted.
    pub fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>, String> {
        // 1. Search active memtable
        let active = {
            let active_lock = self
                .inner
                .active_memtable
                .read()
                .map_err(|_| "Lock poisoned during active get")?;
            active_lock.clone()
        };
        if let Some(res) = active.get(key).map_err(|e| e.to_string())? {
            return Ok(res); // Might return Some(value) or None (deleted)
        }

        // 2. Search frozen memtable (if present)
        let frozen = {
            let frozen_lock = self
                .inner
                .frozen_memtable
                .read()
                .map_err(|_| "Lock poisoned during frozen get")?;
            frozen_lock.clone()
        };
        if let Some(f) = frozen {
            if let Some(res) = f.get(key).map_err(|e| e.to_string())? {
                return Ok(res);
            }
        }

        // 3. Search SSTables on disk (newest first)
        let ssts = {
            let ssts_lock = self
                .inner
                .sstables
                .read()
                .map_err(|_| "Lock poisoned during SSTable get")?;
            ssts_lock.clone()
        };
        for reader in ssts {
            if let Some(res) = reader.get(key).map_err(|e| e.to_string())? {
                return Ok(res);
            }
        }

        Ok(None)
    }

    /// Checks if the provided active memtable exceeds size thresholds and triggers flushing.
    fn check_flush(&self, mem: Arc<MemTable>) -> Result<(), String> {
        if mem.size() >= self.inner.config.flush_threshold_bytes {
            let mut active_lock = self
                .inner
                .active_memtable
                .write()
                .map_err(|_| "Lock poisoned during flush check")?;

            // Assert that another thread has not already swapped it
            if !Arc::ptr_eq(&*active_lock, &mem) {
                return Ok(());
            }

            let mut frozen_lock = self
                .inner
                .frozen_memtable
                .write()
                .map_err(|_| "Lock poisoned during flush freeze")?;
            if frozen_lock.is_some() {
                // If previous flush is still running, throttle writes by waiting (blocking)
                return Ok(());
            }

            // Swap active out and transition it to frozen
            let new_mem = Arc::new(MemTable::new());
            *frozen_lock = Some(mem);
            *active_lock = new_mem;

            // Spawn background writer thread
            let self_clone = self.clone();
            std::thread::spawn(move || {
                if let Err(e) = self_clone.flush_frozen() {
                    log_error!("platform-nodes::lsm", "Background flush error: {:?}", e);
                }
            });
        }
        Ok(())
    }

    /// Background task: Drains and flushes the frozen memtable to disk as an SSTable file.
    fn flush_frozen(&self) -> Result<(), String> {
        let mem = {
            let frozen = self
                .inner
                .frozen_memtable
                .read()
                .map_err(|_| "Lock poisoned during background read")?;
            match &*frozen {
                Some(m) => m.clone(),
                None => return Ok(()),
            }
        };

        let entries = mem.drain().map_err(|e| e.to_string())?;
        if entries.is_empty() {
            let mut frozen = self
                .inner
                .frozen_memtable
                .write()
                .map_err(|_| "Lock poisoned during empty freeze")?;
            *frozen = None;
            return Ok(());
        }

        let sstable_id = self.inner.next_sstable_id.fetch_add(1, Ordering::SeqCst);
        let filename = format!("{:05}.db", sstable_id);
        let path = self.inner.config.data_dir.join(filename);

        // Serialize to disk
        SstableWriter::write_to_file(&path, entries).map_err(|e| e.to_string())?;

        let reader = SstableReader::open(&path).map_err(|e| e.to_string())?;
        let reader_arc = Arc::new(reader);

        // Update active SSTable lists (prepend as it is the newest disk layer)
        {
            let mut sst = self
                .inner
                .sstables
                .write()
                .map_err(|_| "Lock poisoned during SSTable add")?;
            sst.insert(0, reader_arc);
        }

        // Evict frozen memtable
        {
            let mut frozen = self
                .inner
                .frozen_memtable
                .write()
                .map_err(|_| "Lock poisoned during frozen clear")?;
            *frozen = None;
        }

        log_info!(
            "platform-nodes::lsm",
            "Flushed memory buffers to disk-backed SSTable: {:?}",
            path
        );

        // Trigger compaction if file limits are exceeded
        self.trigger_compaction()?;

        Ok(())
    }

    /// Evaluates if active files exceed thresholds and triggers background compaction.
    fn trigger_compaction(&self) -> Result<(), String> {
        let count = {
            let sst = self
                .inner
                .sstables
                .read()
                .map_err(|_| "Lock poisoned during compaction check")?;
            sst.len()
        };

        if count >= self.inner.config.compaction_trigger_files {
            let self_clone = self.clone();
            std::thread::spawn(move || {
                if let Err(e) = self_clone.run_compaction() {
                    log_error!(
                        "platform-nodes::lsm",
                        "Background compaction error: {:?}",
                        e
                    );
                }
            });
        }
        Ok(())
    }

    /// Background task: Merges existing SSTables, builds a consolidated replacement, and swaps files.
    fn run_compaction(&self) -> Result<(), String> {
        let to_compact = {
            let sst = self
                .inner
                .sstables
                .read()
                .map_err(|_| "Lock poisoned during compaction start")?;
            if sst.len() < self.inner.config.compaction_trigger_files {
                return Ok(());
            }
            sst.clone()
        };

        log_info!(
            "platform-nodes::lsm",
            "Executing background merge compaction for {} SSTables...",
            to_compact.len()
        );

        let merged_id = self.inner.next_sstable_id.fetch_add(1, Ordering::SeqCst);
        let merged_filename = format!("{:05}.db", merged_id);
        let merged_path = self.inner.config.data_dir.join(merged_filename);

        // Perform major compaction, merging and stripping tombstones
        compactor::compact(&to_compact, &merged_path, true).map_err(|e| e.to_string())?;

        let new_reader = SstableReader::open(&merged_path).map_err(|e| e.to_string())?;
        let new_reader_arc = Arc::new(new_reader);

        // Atomically replace compacted files in engine state
        {
            let mut sst = self
                .inner
                .sstables
                .write()
                .map_err(|_| "Lock poisoned during compaction swap")?;

            let compacted_paths: std::collections::HashSet<PathBuf> =
                to_compact.iter().map(|r| r.path().to_path_buf()).collect();

            // Keep newly flushed tables that completed during compaction
            sst.retain(|r| !compacted_paths.contains(r.path()));
            // Append compacted merged reader to the end (oldest table)
            sst.push(new_reader_arc);
        }

        // Delete compacted files from disk
        for reader in to_compact {
            let path = reader.path();
            if let Err(e) = fs::remove_file(path) {
                log_error!(
                    "platform-nodes::lsm",
                    "Failed to remove compacted file {:?}: {:?}",
                    path,
                    e
                );
            }
        }

        log_info!(
            "platform-nodes::lsm",
            "Compaction complete. Swapped file: {:?}",
            merged_path
        );

        Ok(())
    }

    /// Returns the active number of SSTables tracked.
    pub fn sstable_count(&self) -> Result<usize, String> {
        let sst = self
            .inner
            .sstables
            .read()
            .map_err(|_| "Lock poisoned during sstable_count query")?;
        Ok(sst.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_memtable_basic() {
        let mem = MemTable::new();
        assert!(mem.is_empty());
        assert_eq!(mem.size(), 0);

        mem.put(b"key1".to_vec(), b"value1".to_vec()).unwrap();
        assert!(!mem.is_empty());
        assert_eq!(mem.size(), 10); // 4 + 6

        // Overwrite
        mem.put(b"key1".to_vec(), b"val2".to_vec()).unwrap();
        assert_eq!(mem.size(), 8); // 4 + 4

        // Get
        let res = mem.get(b"key1").unwrap();
        assert_eq!(res, Some(Some(b"val2".to_vec())));

        // Delete
        mem.delete(b"key1".to_vec()).unwrap();
        assert_eq!(mem.size(), 4); // key size only (tombstone)

        let res = mem.get(b"key1").unwrap();
        assert_eq!(res, Some(None)); // Tombstone marker

        let res_nonexistent = mem.get(b"key2").unwrap();
        assert_eq!(res_nonexistent, None);
    }

    #[test]
    fn test_sstable_write_read() {
        let test_dir = Path::new("data/sstable_test");
        let _ = fs::create_dir_all(test_dir);
        let sstable_path = test_dir.join("00001.db");

        let entries = vec![
            (b"key1".to_vec(), Some(b"value1".to_vec())),
            (b"key2".to_vec(), None), // Tombstone
            (b"key3".to_vec(), Some(b"value3".to_vec())),
        ];

        SstableWriter::write_to_file(&sstable_path, entries.clone()).unwrap();

        let reader = SstableReader::open(&sstable_path).unwrap();
        assert_eq!(reader.len(), 3);
        assert_eq!(reader.min_key(), Some(b"key1".as_slice()));
        assert_eq!(reader.max_key(), Some(b"key3".as_slice()));

        // Check gets
        assert_eq!(reader.get(b"key1").unwrap(), Some(Some(b"value1".to_vec())));
        assert_eq!(reader.get(b"key2").unwrap(), Some(None));
        assert_eq!(reader.get(b"key3").unwrap(), Some(Some(b"value3".to_vec())));
        assert_eq!(reader.get(b"key4").unwrap(), None);

        // Check sequential streaming
        let read_entries = reader.all_entries().unwrap();
        assert_eq!(read_entries, entries);

        let _ = fs::remove_dir_all(test_dir);
    }

    #[test]
    fn test_lsm_engine_flow_and_compaction() {
        let test_dir = PathBuf::from("data/lsm_engine_test");
        if test_dir.exists() {
            let _ = fs::remove_dir_all(&test_dir);
        }

        // Configure tiny threshold (30 bytes) to force memtable flush on every few puts
        let config = LsmConfig {
            data_dir: test_dir.clone(),
            flush_threshold_bytes: 30,
            compaction_trigger_files: 4,
        };

        let engine = LsmEngine::open(config).unwrap();

        // 1. Write records (should trigger multiple flushes)
        engine.put(b"aaa".to_vec(), b"111".to_vec()).unwrap(); // size ~ 6
        engine.put(b"bbb".to_vec(), b"222".to_vec()).unwrap(); // size ~ 6
        engine.put(b"ccc".to_vec(), b"333".to_vec()).unwrap(); // size ~ 6

        // Write large records to force size-limit flushes
        engine
            .put(b"ddd_large_key".to_vec(), b"ddd_large_val".to_vec())
            .unwrap(); // size 26
        engine
            .put(b"eee_large_key".to_vec(), b"eee_large_val".to_vec())
            .unwrap(); // size 26
        engine
            .put(b"fff_large_key".to_vec(), b"fff_large_val".to_vec())
            .unwrap(); // size 26
        engine
            .put(b"ggg_large_key".to_vec(), b"ggg_large_val".to_vec())
            .unwrap(); // size 26

        // Give background flushes/compactions a moment to run
        std::thread::sleep(std::time::Duration::from_millis(200));

        // 2. Put tombstones
        engine.delete(b"aaa".to_vec()).unwrap();
        engine.delete(b"bbb".to_vec()).unwrap();

        // Put another large entry to flush tombstones to disk
        engine
            .put(b"hhh_large_key".to_vec(), b"hhh_large_val".to_vec())
            .unwrap();

        std::thread::sleep(std::time::Duration::from_millis(200));

        // 3. Verify retrievals
        assert_eq!(engine.get(b"aaa").unwrap(), None); // Deleted
        assert_eq!(engine.get(b"bbb").unwrap(), None); // Deleted
        assert_eq!(engine.get(b"ccc").unwrap(), Some(b"333".to_vec()));
        assert_eq!(
            engine.get(b"ddd_large_key").unwrap(),
            Some(b"ddd_large_val".to_vec())
        );

        // 4. Assert that compaction merged SSTables (we triggered many flushes, so at least 4 tables should have compacted into fewer)
        let sst_count = engine.sstable_count().unwrap();
        assert!(sst_count > 0);
        // Compaction triggers at 4 and merges them. The total should be small.
        assert!(sst_count < 4);

        // 5. Open new engine instance on the same data directory to verify state recovery
        let recovered_engine = LsmEngine::open(LsmConfig {
            data_dir: test_dir.clone(),
            flush_threshold_bytes: 30,
            compaction_trigger_files: 4,
        })
        .unwrap();

        assert_eq!(recovered_engine.get(b"aaa").unwrap(), None);
        assert_eq!(recovered_engine.get(b"ccc").unwrap(), Some(b"333".to_vec()));

        let _ = fs::remove_dir_all(&test_dir);
    }
}
