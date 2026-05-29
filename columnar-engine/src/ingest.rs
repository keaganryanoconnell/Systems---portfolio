use crate::chunk::ColumnarChunk;
use crate::error::{EngineError, EngineResult};

const MAGIC: &[u8; 4] = b"SPAT";

pub fn parse_header(raw_bytes: &[u8]) -> EngineResult<(u32, [u32; 4])> {
    if raw_bytes.len() < 24 {
        return Err(EngineError::InsufficientData { expected: 24, actual: raw_bytes.len() });
    }

    let magic: [u8; 4] = [raw_bytes[0], raw_bytes[1], raw_bytes[2], raw_bytes[3]];
    if &magic != MAGIC {
        return Err(EngineError::InvalidMagic(magic));
    }

    let row_count = u32::from_le_bytes([raw_bytes[4], raw_bytes[5], raw_bytes[6], raw_bytes[7]]);

    let offsets = [
        u32::from_le_bytes([raw_bytes[8],  raw_bytes[9],  raw_bytes[10], raw_bytes[11]]),
        u32::from_le_bytes([raw_bytes[12], raw_bytes[13], raw_bytes[14], raw_bytes[15]]),
        u32::from_le_bytes([raw_bytes[16], raw_bytes[17], raw_bytes[18], raw_bytes[19]]),
        u32::from_le_bytes([raw_bytes[20], raw_bytes[21], raw_bytes[22], raw_bytes[23]]),
    ];

    for (i, &offset) in offsets.iter().enumerate() {
        if offset as usize > raw_bytes.len() {
            let fields = ["timestamps", "latitudes", "longitudes", "entity_ids"];
            return Err(EngineError::InvalidOffset {
                field: fields[i],
                offset,
                total: raw_bytes.len(),
            });
        }
    }

    Ok((row_count, offsets))
}

/// Ingests raw binary data into a columnar chunk via zero-copy pointer casts.
///
/// # Safety
///
/// Caller must ensure `raw_bytes` points to valid memory matching the SPAT header
/// format and contains complete column arrays for the declared row count.
pub unsafe fn ingest_raw_block(chunk: &mut ColumnarChunk, raw_bytes: &[u8]) -> EngineResult<u32> {
    let (row_count, offsets) = parse_header(raw_bytes)?;

    let ts_off = offsets[0] as usize;
    let lat_off = offsets[1] as usize;
    let lon_off = offsets[2] as usize;
    let eid_off = offsets[3] as usize;

    let ts_bytes = row_count as usize * 8;
    let f32_bytes = row_count as usize * 4;
    let u32_bytes = row_count as usize * 4;

    if ts_off + ts_bytes > raw_bytes.len()
        || lat_off + f32_bytes > raw_bytes.len()
        || lon_off + f32_bytes > raw_bytes.len()
        || eid_off + u32_bytes > raw_bytes.len()
    {
        return Err(EngineError::InsufficientData {
            expected: ts_off.max(lat_off).max(lon_off).max(eid_off) + ts_bytes,
            actual: raw_bytes.len(),
        });
    }

    let timestamps: &[f64] = bytemuck::cast_slice(&raw_bytes[ts_off..ts_off + ts_bytes]);
    let latitudes:  &[f32] = bytemuck::cast_slice(&raw_bytes[lat_off..lat_off + f32_bytes]);
    let longitudes: &[f32] = bytemuck::cast_slice(&raw_bytes[lon_off..lon_off + f32_bytes]);
    let entity_ids: &[u32] = bytemuck::cast_slice(&raw_bytes[eid_off..eid_off + u32_bytes]);

    chunk.extend_from(timestamps, latitudes, longitudes, entity_ids)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_validation() {
        let data = vec![0u8; 24];
        let result = parse_header(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_header_valid() {
        let mut data = vec![0u8; 2000];
        data[0..4].copy_from_slice(b"SPAT");
        data[4..8].copy_from_slice(&100u32.to_le_bytes());
        data[8..12].copy_from_slice(&24u32.to_le_bytes());
        data[12..16].copy_from_slice(&824u32.to_le_bytes());
        data[16..20].copy_from_slice(&1224u32.to_le_bytes());
        data[20..24].copy_from_slice(&1624u32.to_le_bytes());

        let (rows, offsets) = parse_header(&data).unwrap();
        assert_eq!(rows, 100);
        assert_eq!(offsets[0], 24);
        assert_eq!(offsets[1], 824);
        assert_eq!(offsets[2], 1224);
        assert_eq!(offsets[3], 1624);
    }

    #[test]
    fn test_ingest_and_verify() {
        let row_count: u32 = 100;
        let ts_size = row_count as usize * 8;
        let f32_size = row_count as usize * 4;
        let u32_size = row_count as usize * 4;

        let ts_off = 24usize;
        let lat_off = ts_off + ts_size;
        let lon_off = lat_off + f32_size;
        let eid_off = lon_off + f32_size;
        let total = eid_off + u32_size;

        let mut data = vec![0u8; total];
        data[0..4].copy_from_slice(b"SPAT");
        data[4..8].copy_from_slice(&row_count.to_le_bytes());
        data[8..12].copy_from_slice(&(ts_off as u32).to_le_bytes());
        data[12..16].copy_from_slice(&(lat_off as u32).to_le_bytes());
        data[16..20].copy_from_slice(&(lon_off as u32).to_le_bytes());
        data[20..24].copy_from_slice(&(eid_off as u32).to_le_bytes());

        for i in 0..row_count as usize {
            data[ts_off + i*8..ts_off + i*8 + 8].copy_from_slice(&(i as f64).to_le_bytes());
            data[lat_off + i*4..lat_off + i*4 + 4].copy_from_slice(&(i as f32).to_le_bytes());
            data[lon_off + i*4..lon_off + i*4 + 4].copy_from_slice(&((i * 2) as f32).to_le_bytes());
            data[eid_off + i*4..eid_off + i*4 + 4].copy_from_slice(&(i as u32).to_le_bytes());
        }

        let mut chunk = ColumnarChunk::new(0);
        let ingested = unsafe { ingest_raw_block(&mut chunk, &data).unwrap() };
        assert_eq!(ingested, 100);
        assert_eq!(chunk.timestamps[0], 0.0);
        assert_eq!(chunk.timestamps[99], 99.0);
        assert_eq!(chunk.latitudes[0], 0.0);
        assert_eq!(chunk.longitudes[99], 198.0);
    }
}
