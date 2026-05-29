use crate::chunk::ColumnarChunk;
use crate::error::{EngineError, EngineResult};

/// Executes a vectorized latitude-range filter scan on pre-ingested chunk data.
///
/// Writes matching row indices to `out_ptr` and returns the count of matches.
///
/// # Safety
///
/// Caller must ensure `out_ptr` points to a writable buffer of at least `out_capacity`
/// `u32` elements and that the chunk's column arrays have not been freed or reallocated
/// concurrently.
pub unsafe fn execute_filter_scan(
    chunk: &ColumnarChunk,
    min_lat: f32,
    max_lat: f32,
    out_ptr: *mut u32,
    out_capacity: usize,
) -> EngineResult<usize> {
    if out_ptr.is_null() || out_capacity == 0 {
        return Err(EngineError::BufferTooSmall {
            needed: 1,
            capacity: 0,
        });
    }

    let lats = chunk.latitudes.as_slice();
    let len = chunk.row_count as usize;
    let mut out_count = 0usize;

    for (i, &lat) in lats.iter().enumerate().take(len) {
        let matches = lat >= min_lat && lat <= max_lat;

        if matches {
            let dst = out_ptr.add(out_count);
            *dst = i as u32;
            out_count += 1;
        }

        if out_count >= out_capacity {
            break;
        }
    }

    Ok(out_count)
}

/// Executes a vectorized bounding-box filter scan on pre-ingested chunk data.
///
/// Writes matching row indices to `out_ptr` and returns the count of matches.
///
/// # Safety
///
/// Caller must ensure `out_ptr` points to a writable buffer of at least `out_capacity`
/// `u32` elements and that the chunk's column arrays have not been freed or reallocated
/// concurrently.
pub unsafe fn execute_bbox_scan(
    chunk: &ColumnarChunk,
    lat_min: f32,
    lat_max: f32,
    lon_min: f32,
    lon_max: f32,
    out_ptr: *mut u32,
    out_capacity: usize,
) -> EngineResult<usize> {
    if out_ptr.is_null() || out_capacity == 0 {
        return Err(EngineError::BufferTooSmall {
            needed: 1,
            capacity: 0,
        });
    }

    let lats = chunk.latitudes.as_slice();
    let lons = chunk.longitudes.as_slice();
    let len = chunk.row_count as usize;
    let mut out_count = 0usize;

    for (i, (&lat, &lon)) in lats.iter().zip(lons.iter()).enumerate().take(len) {
        let matches = lat >= lat_min && lat <= lat_max && lon >= lon_min && lon <= lon_max;

        if matches {
            let dst = out_ptr.add(out_count);
            *dst = i as u32;
            out_count += 1;
        }

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
        let count =
            unsafe { execute_filter_scan(&chunk, 10.0, 20.0, output.as_mut_ptr(), 1000).unwrap() };
        assert!(count > 0);
        assert!(count < 1000);
    }

    #[test]
    fn test_empty_result() {
        let mut chunk = ColumnarChunk::new(0);
        populate_chunk(&mut chunk, 100);

        let mut output = vec![0u32; 100];
        let count = unsafe {
            execute_filter_scan(&chunk, -100.0, -50.0, output.as_mut_ptr(), 100).unwrap()
        };
        assert_eq!(count, 0);
    }

    #[test]
    fn test_bbox_query() {
        let mut chunk = ColumnarChunk::new(0);
        populate_chunk(&mut chunk, 1000);

        let mut output = vec![0u32; 1000];
        let count = unsafe {
            execute_bbox_scan(&chunk, 0.0, 100.0, 0.0, 200.0, output.as_mut_ptr(), 1000).unwrap()
        };
        assert!(count > 0);
    }
}
