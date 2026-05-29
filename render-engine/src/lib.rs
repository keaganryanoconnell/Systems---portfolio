pub mod pipeline;

#[cfg(target_arch = "wasm32")]
mod wasm {
    use wasm_bindgen::prelude::*;

    use crate::pipeline::RenderPipeline;

    #[wasm_bindgen]
    pub struct WasmRenderer {
        pipeline: Option<RenderPipeline>,
    }

    #[wasm_bindgen]
    impl WasmRenderer {
        pub fn new() -> Self {
            console_error_panic_hook::set_once();
            Self { pipeline: None }
        }

        pub async fn init(&mut self) -> Result<(), JsValue> {
            let pipeline = RenderPipeline::new().await
                .map_err(|e| JsValue::from_str(&e.to_string()))?;
            self.pipeline = Some(pipeline);
            Ok(())
        }

        pub fn update_buffers(&mut self, ptr: *const u8, len: usize) -> Result<(), JsValue> {
            if let Some(ref mut p) = self.pipeline {
                let data = unsafe { std::slice::from_raw_parts(ptr, len) };
                p.update_buffers(data).map_err(|e| JsValue::from_str(&e.to_string()))?;
            }
            Ok(())
        }

        pub fn render(&mut self) -> Result<(), JsValue> {
            if let Some(ref mut p) = self.pipeline {
                p.render().map_err(|e| JsValue::from_str(&e.to_string()))?;
            }
            Ok(())
        }

        pub fn resize(&mut self, width: u32, height: u32) {
            if let Some(ref mut p) = self.pipeline {
                p.resize(width, height);
            }
        }
    }
}

#[cfg(target_arch = "wasm32")]
pub use wasm::WasmRenderer;
