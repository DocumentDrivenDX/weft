#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_json(request: &str) -> String {
    weft_runtime::compile_json(request)
}

#[cfg(feature = "test-original")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_json_with_conformance_configuration(request: &str, configuration: &str) -> String {
    weft_runtime::compile_json_with_conformance_configuration(request, configuration)
}
