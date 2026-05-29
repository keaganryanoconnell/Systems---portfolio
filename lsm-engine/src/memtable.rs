//! MemTable - In-Memory Sorted Write Buffer
//!
//! Stores key-value updates in sorted order using a standard BTreeMap,
//! protected by a read-write lock for concurrent thread access.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::RwLock;

/// Thread-safe in-memory sorted write buffer tracking memory weight.
pub struct MemTable {
    map: RwLock<BTreeMap<Vec<u8>, Option<Vec<u8>>>>,
    size_in_bytes: AtomicUsize,
}

impl MemTable {
    /// Creates a new empty MemTable instance.
    pub fn new() -> Self {
        Self {
            map: RwLock::new(BTreeMap::new()),
            size_in_bytes: AtomicUsize::new(0),
        }
    }

    /// Inserts or updates a key-value pair in the MemTable.
    pub fn put(&self, key: Vec<u8>, value: Vec<u8>) -> Result<(), &'static str> {
        let key_len = key.len();
        let val_len = value.len();

        let mut map = self.map.write().map_err(|_| "Lock poisoned during put")?;

        let entry_weight = key_len + val_len;

        if let Some(old_opt) = map.insert(key, Some(value)) {
            let old_weight = key_len + old_opt.map_or(0, |v| v.len());
            self.size_in_bytes.fetch_sub(old_weight, Ordering::SeqCst);
        }

        self.size_in_bytes.fetch_add(entry_weight, Ordering::SeqCst);
        Ok(())
    }

    /// Deletes a key by placing a tombstone marker (None) in the MemTable.
    pub fn delete(&self, key: Vec<u8>) -> Result<(), &'static str> {
        let key_len = key.len();

        let mut map = self
            .map
            .write()
            .map_err(|_| "Lock poisoned during delete")?;

        if let Some(old_opt) = map.insert(key, None) {
            let old_weight = key_len + old_opt.map_or(0, |v| v.len());
            self.size_in_bytes.fetch_sub(old_weight, Ordering::SeqCst);
        }

        self.size_in_bytes.fetch_add(key_len, Ordering::SeqCst);
        Ok(())
    }

    /// Retrieves a key from the MemTable.
    ///
    /// Returns:
    /// * `Ok(Some(Some(value)))` if the key exists with a value.
    /// * `Ok(Some(None))` if the key exists as a tombstone (deleted).
    /// * `Ok(None)` if the key does not exist in this MemTable.
    pub fn get(&self, key: &[u8]) -> Result<Option<Option<Vec<u8>>>, &'static str> {
        let map = self.map.read().map_err(|_| "Lock poisoned during get")?;
        Ok(map.get(key).cloned())
    }

    /// Returns the approximate memory consumption of the MemTable in bytes.
    pub fn size(&self) -> usize {
        self.size_in_bytes.load(Ordering::Relaxed)
    }

    /// Returns true if the MemTable contains no elements.
    pub fn is_empty(&self) -> bool {
        if let Ok(map) = self.map.read() {
            map.is_empty()
        } else {
            true
        }
    }

    /// Drains all entries and returns them as a sorted vector.
    /// Resets the memory consumption tracking to zero.
    pub fn drain(&self) -> Result<Vec<crate::engine::KeyValuePair>, &'static str> {
        let mut map = self.map.write().map_err(|_| "Lock poisoned during drain")?;
        let entries = std::mem::take(&mut *map);
        self.size_in_bytes.store(0, Ordering::SeqCst);
        Ok(entries.into_iter().collect())
    }
}

impl Default for MemTable {
    fn default() -> Self {
        Self::new()
    }
}
