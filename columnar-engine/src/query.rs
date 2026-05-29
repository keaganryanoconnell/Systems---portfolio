use crate::chunk::ColumnarChunk;
use crate::error::{EngineError, EngineResult};

pub fn execute_filter_scan(
    chunk: &ColumnarChunk,
    min_lat: f32,
    max_lat: f32,
    out_ptr: *mut u32,
    out_capacity: usize,
) -> EngineResult<usize> {
    if out_ptr.is_null() || out_capacity == 0 {
        return Err(EngineError::BufferTooSmall { needed: 1, capacity: 0 });
    }

    let lats = chunk.latitudes.as_slice();
    let len = chunk.row_count as usize;
    let mut out_count = 0usize;

    for i in 0..len {
        let lat = lats[i];
        let matches = lat >= min_lat && lat <= max_lat;

        unsafe {
            let dst = out_ptr.add(out_count);
            *dst = i as u32;
        }

        out_count += matches as usize;

        if out_count >= out_capacity {
            break;
        }
    }

    Ok(out_count)
}

pub fn execute_bbox_scan(
    chunk: &ColumnarChunk,
    lat_min: f32, lat_max: f32,
    lon_min: f32, lon_max: f32,
    out_ptr: *mut u32,
    out_capacity: usize,
) -> EngineResult<usize> {
    if out_ptr.is_null() || out_capacity == 0 {
        return Err(EngineError::BufferTooSmall { needed: 1, capacity: 0 });
    }

    let lats = chunk.latitudes.as_slice();
    let lons = chunk.longitudes.as_slice();
    let len = chunk.row_count as usize;
    let mut out_count = 0usize;

    for i in 0..len {
        let lat = lats[i];
        let lon = lons[i];
        let matches = lat >= lat_min && lat <= lat_max && lon >= lon_min && lon <= lon_max;

        unsafe {
            let dst = out_ptr.add(out_count);
            *dst = i as u32;
        }

        out_count += matches as usize;

        if out_count >= out_capacity {
            break;
        }
    }

    Ok(out_count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chunk::ColumnarChunk;

    fn populate_chunk(chunk: &mut ColumnarChunk, rows: usize) {
        let ts: Vec<f64> = (0..rows).map(|i| i as f64).collect();
        let lats: Vec<f32> = (0..rows).map(|i| (i as f32) * 0.5).collect();
        let lons: Vec<f32> = (0..rows).map(|i| (i as f32) * 1.0).collect();
        let eids: Vec<u32> = (0..rows).map(|i| i as u32).collect();
        chunk.extend_from(&ts, &lats, &lons, &eids).unwrap();
    }

    #[test]
    fn test_filter_lat_range() {
        let mut chunk = ColumnarChunk::new(0);
        populate_chunk(&mut chunk, 1000);

        let mut output = vec![0u32; 1000];
        let count = execute_filter_scan(&chunk, 10.0, 20.0, output.as_mut_ptr(), 1000).unwrap();
        assert!(count > 0);
        assert!(count < 1000);
    }

    #[test]
    fn test_empty_result() {
        let mut chunk = ColumnarChunk::new(0);
        populate_chunk(&mut chunk, 100);

        let mut output = vec![0u32; 100];
        let count = execute_filter_scan(&chunk, -100.0, -50.0, output.as_mut_ptr(), 100).unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_bbox_query() {
        let mut chunk = ColumnarChunk::new(0);
        populate_chunk(&mut chunk, 1000);

        let mut output = vec![0u32; 1000];
        let count = execute_bbox_scan(
            &chunk, 0.0, 100.0, 0.0, 200.0, output.as_mut_ptr(), 1000,
        ).unwrap();
        assert!(count > 0);
    }
}
