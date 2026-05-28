use crate::error::BrokerResult;

const INDEX_CHUNK_SIZE: usize = 10240;

pub struct IndexEntry {
    pub offset: u64,
    pub position: u64,
    pub msg_size: u32,
}

pub struct SegmentIndex {
    pub base_offset: u64,
    entries: Vec<IndexEntry>,
    next_offset: u64,
}

impl SegmentIndex {
    pub fn new(base_offset: u64) -> Self {
        Self {
            base_offset,
            entries: Vec::with_capacity(INDEX_CHUNK_SIZE),
            next_offset: base_offset,
        }
    }

    pub fn with_capacity(base_offset: u64, capacity: usize) -> Self {
        Self {
            base_offset,
            entries: Vec::with_capacity(capacity),
            next_offset: base_offset,
        }
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn next_offset(&self) -> u64 {
        self.next_offset
    }

    pub fn earliest_offset(&self) -> Option<u64> {
        self.entries.first().map(|e| e.offset)
    }

    pub fn latest_offset(&self) -> Option<u64> {
        self.entries.last().map(|e| e.offset)
    }

    pub fn add(&mut self, offset: u64, position: u64, key_len: u32, value_len: u32) {
        let msg_size = 20u32 + key_len + value_len;

        self.entries.push(IndexEntry {
            offset,
            position,
            msg_size,
        });

        self.next_offset = offset + 1;
    }

    pub fn batch_add(&mut self, items: impl Iterator<Item = (u64, u64, u32)>) {
        for (offset, position, msg_size) in items {
            self.entries.push(IndexEntry {
                offset,
                position,
                msg_size,
            });
            self.next_offset = offset + 1;
        }
    }

    pub fn find_position(&self, offset: u64) -> BrokerResult<u64> {
        if self.entries.is_empty() {
            if let (Some(earliest), Some(latest)) = (self.earliest_offset(), self.latest_offset()) {
                return Err(crate::error::BrokerError::OffsetOutOfRange {
                    requested: offset,
                    earliest,
                    latest,
                });
            }
            return Err(crate::error::BrokerError::OffsetOutOfRange {
                requested: offset,
                earliest: 0,
                latest: 0,
            });
        }

        if offset < self.entries[0].offset {
            let earliest = self.entries[0].offset;
            let latest = self.entries[self.entries.len() - 1].offset;
            return Err(crate::error::BrokerError::OffsetOutOfRange {
                requested: offset,
                earliest,
                latest,
            });
        }

        if offset > self.entries[self.entries.len() - 1].offset {
            return Err(crate::error::BrokerError::OffsetOutOfRange {
                requested: offset,
                earliest: self.entries[0].offset,
                latest: self.entries[self.entries.len() - 1].offset,
            });
        }

        match self.binary_search(offset) {
            Ok(idx) => Ok(self.entries[idx].position),
            Err(idx) => {
                if idx == 0 {
                    Ok(self.entries[0].position)
                } else {
                    Ok(self.entries[idx - 1].position)
                }
            }
        }
    }

    fn binary_search(&self, target: u64) -> Result<usize, usize> {
        self.entries
            .binary_search_by(|entry| entry.offset.cmp(&target))
    }

    pub fn scan_from(&self, start_offset: u64) -> impl Iterator<Item = &IndexEntry> {
        let start_idx = match self.binary_search(start_offset) {
            Ok(idx) => idx,
            Err(idx) => idx,
        };
        self.entries[start_idx..].iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_index_add_and_find() {
        let mut idx = SegmentIndex::new(0);

        idx.add(0, 0, 3, 5);
        idx.add(1, 28, 3, 5);
        idx.add(2, 56, 4, 6);

        assert_eq!(idx.find_position(0).unwrap(), 0);
        assert_eq!(idx.find_position(1).unwrap(), 28);
        assert_eq!(idx.find_position(2).unwrap(), 56);
        assert_eq!(idx.next_offset(), 3);
    }

    #[test]
    fn test_offset_out_of_range() {
        let mut idx = SegmentIndex::new(100);
        idx.add(100, 0, 1, 1);

        let result = idx.find_position(50);
        assert!(result.is_err());
    }

    #[test]
    fn test_scan_from() {
        let mut idx = SegmentIndex::new(0);
        idx.add(0, 0, 1, 1);
        idx.add(1, 22, 1, 1);
        idx.add(2, 44, 1, 1);
        idx.add(3, 66, 1, 1);

        let from_1: Vec<u64> = idx.scan_from(1).map(|e| e.offset).collect();
        assert_eq!(from_1, vec![1, 2, 3]);
    }
}
