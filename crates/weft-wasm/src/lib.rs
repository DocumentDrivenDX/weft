/// Original ontology Subject/Record/Key source selection only; no authority.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn inspect_security_mapping_subject_key_json(request: &str, selection: &str) -> String {
    weft_runtime::inspect_security_mapping_subject_key_json(request, selection)
}
/// Pure logical preparation; never native execution admission.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn prepare_security_mapping_json(request: &str) -> String {
    weft_runtime::prepare_security_mapping_json(request)
}

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

#[cfg(feature="ashlar-databricks-implicit-arithmetic")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_implicit_arithmetic_json(request: &str) -> String {
    weft_runtime::implicit_arithmetic::compile_json(request)
}
