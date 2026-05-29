//! Compactor - SSTable Merge Sort Compaction
//!
//! Merges multiple immutable SSTable files on disk into a single consolidated
//! SSTable, resolving duplicate keys and purging tombstones.

use crate::sstable::{SstableReader, SstableWriter};
use std::collections::BTreeMap;
use std::io;
use std::path::Path;
use std::sync::Arc;

/// Compacts multiple SSTables into a single new file.
///
/// Merges records from oldest to newest so newer modifications overwrite older ones.
/// If `is_major` is true, deletes tombstones (None values) to save disk space.
pub fn compact(
    sstables: &[Arc<SstableReader>],
    output_path: &Path,
    is_major: bool,
) -> io::Result<()> {
    let mut merged = BTreeMap::new();

    // Iterate in reverse (oldest to newest) to let newer values overwrite older ones
    for reader in sstables.iter().rev() {
        let entries = reader.all_entries()?;
        for (key, val) in entries {
            merged.insert(key, val);
        }
    }

    // Filter out tombstones if this is a major compaction
    let final_entries: Vec<crate::engine::KeyValuePair> = if is_major {
        merged
            .into_iter()
            .filter(|(_, val)| val.is_some())
            .collect()
    } else {
        merged.into_iter().collect()
    };

    SstableWriter::write_to_file(output_path, final_entries)?;
    Ok(())
}
