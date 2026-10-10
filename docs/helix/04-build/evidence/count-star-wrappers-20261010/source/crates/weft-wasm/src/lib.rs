#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_json(request: &str) -> String {
    weft_runtime::compile_json(request)
}

#[cfg(feature = "test-original")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_json_with_conformance_configuration(request: &str, configuration: &str) -> String {
    weft_runtime::compile_json_with_conformance_configuration(request, configuration)
}

/// Explicit PathsKeys transport; generic compile_json retains its original composition.
#[cfg(feature = "ashlar-databricks-paths-keys")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_paths_keys_json(request: &str) -> String {
    weft_runtime::paths_keys::compile_json(request)
}
