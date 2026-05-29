use columnar_engine::{
    ColumnarChunk, EngineMemoryManager,
    execute_bbox_scan,
    ingest_raw_block,
};

fn build_test_block(rows: u32) -> Vec<u8> {
    let ts_size = rows as usize * 8;
    let f32_size = rows as usize * 4;
    let u32_size = rows as usize * 4;

    let ts_off = 24usize;
    let lat_off = ts_off + ts_size;
    let lon_off = lat_off + f32_size;
    let eid_off = lon_off + f32_size;
    let total = eid_off + u32_size;

    let mut data = vec![0u8; total];
    data[0..4].copy_from_slice(b"SPAT");
    data[4..8].copy_from_slice(&rows.to_le_bytes());
    data[8..12].copy_from_slice(&(ts_off as u32).to_le_bytes());
    data[12..16].copy_from_slice(&(lat_off as u32).to_le_bytes());
    data[16..20].copy_from_slice(&(lon_off as u32).to_le_bytes());
    data[20..24].copy_from_slice(&(eid_off as u32).to_le_bytes());

    for i in 0..rows as usize {
        data[ts_off + i*8..ts_off + i*8 + 8].copy_from_slice(&(i as f64).to_le_bytes());
        data[lat_off + i*4..lat_off + i*4 + 4].copy_from_slice(&(i as f32).to_le_bytes());
        data[lon_off + i*4..lon_off + i*4 + 4].copy_from_slice(&((i * 2) as f32).to_le_bytes());
        data[eid_off + i*4..eid_off + i*4 + 4].copy_from_slice(&(i as u32).to_le_bytes());
    }

    data
}

#[test]
fn test_memory_eviction_on_large_dataset() {
    let mut manager = EngineMemoryManager::new(10);
    let test_block = build_test_block(1000);

    let mut total_ingested = 0u32;
    for _ in 0..50 {
        let mut chunk = manager.alloc_chunk().unwrap();
        let rows = unsafe { ingest_raw_block(&mut chunk, &test_block).unwrap() };
        total_ingested += rows;
    }

    assert!(total_ingested > 10000);
    assert!(manager.evicted_count() > 0);
    assert!(manager.heap_used() <= manager.max_capacity());
}

#[test]
fn test_lru_eviction_order() {
    let mut manager = EngineMemoryManager::new(15);
    let test_block = build_test_block(500);

    let (chunk0_id, _) = {
        let c0 = manager.alloc_chunk().unwrap();
        let id0 = c0.chunk_id;
        unsafe { ingest_raw_block(c0, &test_block).unwrap(); }
        let c1 = manager.alloc_chunk().unwrap();
        let id1 = c1.chunk_id;
        unsafe { ingest_raw_block(c1, &test_block).unwrap(); }
        (id0, id1)
    };

    manager.touch_chunk(chunk0_id);

    for _ in 0..5 {
        let mut c = manager.alloc_chunk().unwrap();
        unsafe { ingest_raw_block(&mut c, &test_block).ok(); }
    }

    let ids: Vec<u32> = manager.chunks().iter().map(|c| c.chunk_id).collect();
    assert!(ids.contains(&chunk0_id), "Chunk 0 should survive because it was touched (MRU)");
}

#[test]
fn test_bbox_query_accuracy() {
    let mut chunk = ColumnarChunk::new(0);
    let data = build_test_block(1000);
    unsafe { ingest_raw_block(&mut chunk, &data).unwrap(); }

    let mut output = vec![0u32; 1000];
    let count = execute_bbox_scan(
        &chunk, 100.0, 200.0, 200.0, 400.0,
        output.as_mut_ptr(), 1000,
    ).unwrap();

    for i in 0..count {
        let idx = output[i] as usize;
        let lat = chunk.latitudes[idx];
        let lon = chunk.longitudes[idx];
        assert!(lat >= 100.0 && lat <= 200.0);
        assert!(lon >= 200.0 && lon <= 400.0);
    }
}

#[test]
fn test_zero_copy_ingestion_rejects_invalid_magic() {
    let mut chunk = ColumnarChunk::new(0);
    let data = vec![0u8; 100];
    let result = unsafe { ingest_raw_block(&mut chunk, &data) };
    assert!(result.is_err());
}

#[test]
fn test_column_alignment() {
    let chunk = ColumnarChunk::new(0);
    assert_eq!(chunk.timestamps.capacity() % 8, 0);
    assert_eq!(chunk.latitudes.capacity() % 4, 0);
    assert_eq!(chunk.longitudes.capacity() % 4, 0);
    assert_eq!(chunk.entity_ids.capacity() % 4, 0);
}

#[test]
fn test_no_memory_leak_on_repeated_alloc_free() {
    let mut manager = EngineMemoryManager::new(20);
    let test_block = build_test_block(4000);

    for _ in 0..20 {
        let mut chunk = manager.alloc_chunk().unwrap();
        unsafe { ingest_raw_block(&mut chunk, &test_block).ok(); }
    }

    assert!(manager.heap_used() <= manager.max_capacity());
}
