use wasm_bindgen::prelude::*;
#[wasm_bindgen]
pub fn compile_json(request: &str) -> String {
    weft_spike_core::compile_json(request)
}
