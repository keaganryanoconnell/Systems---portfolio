pub mod chunk;
pub mod error;
pub mod ingest;
pub mod pool;
pub mod query;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::chunk::ColumnarChunk;
    use crate::error::EngineError;
    use crate::ingest::ingest_raw_block;
    use crate::pool::EngineMemoryManager;
    use crate::query::{execute_bbox_scan, execute_filter_scan};

    #[wasm_bindgen]
    pub struct WasmEngine {
        manager: EngineMemoryManager,
    }

    #[wasm_bindgen]
    impl WasmEngine {
        pub fn new(memory_cap_mb: u32) -> WasmEngine {
            WasmEngine {
                manager: EngineMemoryManager::new(memory_cap_mb),
            }
        }

        pub fn ingest(&mut self, ptr: *const u8, len: usize) -> Result<u32, JsValue> {
            let bytes = unsafe { std::slice::from_raw_parts(ptr, len) };
            let mut chunk = none_or_error(self.manager.alloc_chunk())?;
            let rows = unsafe {
                ingest_raw_block(&mut chunk, bytes)
                    .map_err(|e| JsValue::from_str(&e.to_string()))?
            };
            Ok(rows)
        }

        pub fn query_lat_range(
            &mut self,
            chunk_id: u32,
            min_lat: f32,
            max_lat: f32,
        ) -> Result<Vec<u32>, JsValue> {
            for chunk in self.manager.chunks().iter() {
                if chunk.chunk_id == chunk_id {
                    self.manager.touch_chunk(chunk_id);
                    let mut out = vec![0u32; chunk.row_count as usize];
                    let count =
                        execute_filter_scan(chunk, min_lat, max_lat, out.as_mut_ptr(), out.len())
                            .map_err(|e| JsValue::from_str(&e.to_string()))?;
                    out.truncate(count);
                    return Ok(out);
                }
            }
            Ok(vec![])
        }

        pub fn query_bbox(
            &mut self,
            chunk_id: u32,
            lat_min: f32,
            lat_max: f32,
            lon_min: f32,
            lon_max: f32,
        ) -> Result<Vec<u32>, JsValue> {
            for chunk in self.manager.chunks().iter() {
                if chunk.chunk_id == chunk_id {
                    self.manager.touch_chunk(chunk_id);
                    let mut out = vec![0u32; chunk.row_count as usize];
                    let count = execute_bbox_scan(
                        chunk,
                        lat_min,
                        lat_max,
                        lon_min,
                        lon_max,
                        out.as_mut_ptr(),
                        out.len(),
                    )
                    .map_err(|e| JsValue::from_str(&e.to_string()))?;
                    out.truncate(count);
                    return Ok(out);
                }
            }
            Ok(vec![])
        }

        pub fn heap_used(&self) -> usize {
            self.manager.heap_used()
        }

        pub fn evicted_count(&self) -> u64 {
            self.manager.evicted_count()
        }
    }

    fn none_or_error<T>(result: crate::error::EngineResult<T>) -> Result<T, JsValue> {
        result.map_err(|e| JsValue::from_str(&e.to_string()))
    }
}

pub use chunk::ColumnarChunk;
pub use error::{EngineError, EngineResult};
pub use ingest::ingest_raw_block;
pub use pool::EngineMemoryManager;
pub use query::{execute_bbox_scan, execute_filter_scan};

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmEngine;
