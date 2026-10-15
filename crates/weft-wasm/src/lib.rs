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

/// Opt-in source lineage candidate; no native authorization or execution.
#[cfg(feature="application-lineage-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn prepare_application_lineage_json(request:&str)->Result<String,wasm_bindgen::JsValue>{
 weft_runtime::prepare_application_lineage_json(request).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
}

/// Source-bound designation only. It cannot authorize a native operation.
#[cfg(feature="application-lineage-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn prepare_current_query_designations_json(request:&str)->Result<String,wasm_bindgen::JsValue>{
 weft_runtime::prepare_current_query_designations_json(request).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
}

/// Host-installed action preparation. Compiles plans; grants no execution authority.
#[wasm_bindgen::prelude::wasm_bindgen]
pub struct MutationCompiler {
    registry: weft_runtime::MutationRegistry,
}
#[wasm_bindgen::prelude::wasm_bindgen]
impl MutationCompiler {
    #[wasm_bindgen::prelude::wasm_bindgen(constructor)]
    pub fn new(configuration: &str) -> Result<MutationCompiler, wasm_bindgen::JsValue> {
        let registry = weft_runtime::MutationRegistry::from_json(configuration)
            .map_err(|d| wasm_bindgen::JsValue::from_str(&format!("{}: {}", d.code, d.message)))?;
        Ok(Self { registry })
    }
    #[cfg(feature="truss-browser-local")]
    pub fn register_browser_selection(&mut self,configuration:&str)->Result<usize,wasm_bindgen::JsValue>{
        weft_runtime::install_browser_mutation_backend(&mut self.registry,configuration)
            .map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
    }
    pub fn lower_selection_json(&self,handle:usize,request:&str)->Result<String,wasm_bindgen::JsValue>{
        weft_runtime::lower_mutation_selection_json(&self.registry,handle,request)
            .map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
    }
    pub fn compile_json(&self, request: &str) -> String {
        self.registry.compile_json(request)
    }
}
