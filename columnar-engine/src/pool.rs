use crate::chunk::ColumnarChunk;
use crate::error::EngineResult;

const DEFAULT_MEMORY_CAP: usize = 256 * 1024 * 1024;

pub struct EngineMemoryManager {
    chunks: Vec<ColumnarChunk>,
    lru_seq: Vec<u64>,
    global_seq: u64,
    current_heap_usage: usize,
    max_heap_capacity: usize,
    evicted_count: u64,
    next_chunk_id: u32,
}

impl EngineMemoryManager {
    pub fn new(max_heap_capacity_mb: u32) -> Self {
        let cap = if max_heap_capacity_mb == 0 {
            DEFAULT_MEMORY_CAP
        } else {
            (max_heap_capacity_mb as usize) * 1024 * 1024
        };

        Self {
            chunks: Vec::new(),
            lru_seq: Vec::new(),
            global_seq: 0,
            current_heap_usage: 0,
            max_heap_capacity: cap,
            evicted_count: 0,
            next_chunk_id: 0,
        }
    }

    pub fn alloc_chunk(&mut self) -> EngineResult<&mut ColumnarChunk> {
        let new_size = {
            let probe = ColumnarChunk::new(0);
            probe.memory_used()
        };

        while self.current_heap_usage + new_size > self.max_heap_capacity {
            if self.chunks.is_empty() {
                break;
            }

            let oldest_idx = self.find_lru_idx();
            self.evict_chunk(oldest_idx);
        }

        let chunk = ColumnarChunk::new(self.next_chunk_id);
        self.next_chunk_id += 1;
        self.current_heap_usage += chunk.memory_used();
        self.chunks.push(chunk);
        self.global_seq += 1;
        self.lru_seq.push(self.global_seq);

        let idx = self.chunks.len() - 1;
        Ok(&mut self.chunks[idx])
    }

    pub fn touch_chunk(&mut self, chunk_id: u32) {
        for (i, chunk) in self.chunks.iter().enumerate() {
            if chunk.chunk_id == chunk_id {
                self.global_seq += 1;
                self.lru_seq[i] = self.global_seq;
                return;
            }
        }
    }

    pub fn heap_used(&self) -> usize {
        self.current_heap_usage
    }

    pub fn max_capacity(&self) -> usize {
        self.max_heap_capacity
    }

    pub fn evicted_count(&self) -> u64 {
        self.evicted_count
    }

    pub fn chunk_count(&self) -> usize {
        self.chunks.len()
    }

    pub fn chunks(&self) -> &[ColumnarChunk] {
        &self.chunks
    }

    fn find_lru_idx(&self) -> usize {
        let mut min_seq = u64::MAX;
        let mut min_idx = 0;

        for (i, &seq) in self.lru_seq.iter().enumerate() {
            if seq < min_seq {
                min_seq = seq;
                min_idx = i;
            }
        }

        min_idx
    }

    fn evict_chunk(&mut self, idx: usize) {
        let freed = self.chunks[idx].memory_used();
        self.current_heap_usage = self.current_heap_usage.saturating_sub(freed);
        self.chunks.swap_remove(idx);
        self.lru_seq.swap_remove(idx);
        self.evicted_count += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_tracking() {
        let mut mgr = EngineMemoryManager::new(0);
        let initial = mgr.heap_used();
        assert_eq!(initial, 0);

        let _chunk = mgr.alloc_chunk().unwrap();
        assert!(mgr.heap_used() > 0);
    }

    #[test]
    fn test_lru_eviction_triggers_at_threshold() {
        let mut mgr = EngineMemoryManager::new(1);
        let single = ColumnarChunk::new(0).memory_used();

        for _ in 0..3 {
            mgr.alloc_chunk().unwrap();
        }

        assert!(
            mgr.evicted_count() > 0,
            "LRU eviction should fire with 1MB cap (chunks are {} bytes each)",
            single
        );
    }

    #[test]
    fn test_lru_touch_moves_to_mru() {
        let mut mgr = EngineMemoryManager::new(20);
        let _c0 = mgr.alloc_chunk().unwrap();
        let _c1 = mgr.alloc_chunk().unwrap();

        mgr.touch_chunk(0);

        let lru_idx = mgr.find_lru_idx();
        assert_eq!(
            mgr.chunks[lru_idx].chunk_id, 1,
            "Chunk 1 should be LRU after touching chunk 0"
        );
    }
}
