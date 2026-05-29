use crate::error::EngineResult;

pub const CHUNK_ROWS: usize = 65536;
const _TIMESTAMPS_SIZE: usize = CHUNK_ROWS * 8;
const _LATITUDES_SIZE: usize = CHUNK_ROWS * 4;
const _LONGITUDES_SIZE: usize = CHUNK_ROWS * 4;
const _ENTITY_IDS_SIZE: usize = CHUNK_ROWS * 4;

pub struct ColumnarChunk {
    pub timestamps: Vec<f64>,
    pub latitudes: Vec<f32>,
    pub longitudes: Vec<f32>,
    pub entity_ids: Vec<u32>,
    pub row_count: u32,
    pub chunk_id: u32,
}

impl ColumnarChunk {
    pub fn new(chunk_id: u32) -> Self {
        Self {
            timestamps: Vec::with_capacity(CHUNK_ROWS),
            latitudes: Vec::with_capacity(CHUNK_ROWS),
            longitudes: Vec::with_capacity(CHUNK_ROWS),
            entity_ids: Vec::with_capacity(CHUNK_ROWS),
            row_count: 0,
            chunk_id,
        }
    }

    pub fn memory_used(&self) -> usize {
        self.timestamps.capacity() * 8
            + self.latitudes.capacity() * 4
            + self.longitudes.capacity() * 4
            + self.entity_ids.capacity() * 4
    }

    pub fn rows_remaining(&self) -> u32 {
        (CHUNK_ROWS as u32).saturating_sub(self.row_count)
    }

    pub fn extend_from(
        &mut self,
        timestamps: &[f64],
        latitudes: &[f32],
        longitudes: &[f32],
        entity_ids: &[u32],
    ) -> EngineResult<u32> {
        let count = timestamps
            .len()
            .min(latitudes.len())
            .min(longitudes.len())
            .min(entity_ids.len());
        let count = count.min(self.rows_remaining() as usize);

        if count == 0 {
            return Ok(0);
        }

        self.timestamps.extend_from_slice(&timestamps[..count]);
        self.latitudes.extend_from_slice(&latitudes[..count]);
        self.longitudes.extend_from_slice(&longitudes[..count]);
        self.entity_ids.extend_from_slice(&entity_ids[..count]);
        self.row_count += count as u32;

        Ok(count as u32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunk_creation() {
        let chunk = ColumnarChunk::new(0);
        assert_eq!(chunk.row_count, 0);
        assert_eq!(chunk.rows_remaining(), CHUNK_ROWS as u32);
        assert!(chunk.memory_used() > 0);
    }

    #[test]
    fn test_extend_from() {
        let mut chunk = ColumnarChunk::new(0);
        let data: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let f32_data: Vec<f32> = (0..1000).map(|i| i as f32).collect();
        let u32_data: Vec<u32> = (0..1000).map(|i| i as u32).collect();

        let count = chunk
            .extend_from(&data, &f32_data, &f32_data, &u32_data)
            .unwrap();
        assert_eq!(count, 1000);
        assert_eq!(chunk.row_count, 1000);
        assert_eq!(chunk.timestamps[0], 0.0);
        assert_eq!(chunk.timestamps[999], 999.0);
    }
}
