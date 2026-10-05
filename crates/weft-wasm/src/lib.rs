#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_json(request: &str) -> String {
    weft_runtime::compile_json(request)
}
