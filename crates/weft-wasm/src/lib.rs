/// Original ontology Subject/Record/Key source selection only; no authority.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn inspect_security_mapping_subject_key_json(request: js_sys::JsString, selection: js_sys::JsString) -> String {
    let Ok(selection)=strict_text(&selection,16_384) else {return weft_runtime::inspect_security_mapping_subject_key_json("","");};
    match strict_text(&request,4_000_000) {
        Ok(request)=>weft_runtime::inspect_security_mapping_subject_key_json(&request,&selection),
        Err(_)=>weft_runtime::inspect_security_mapping_subject_key_json("",&selection),
    }
}
/// Pure logical preparation; never native execution admission.
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn prepare_security_mapping_json(request: js_sys::JsString) -> String {
    match strict_text(&request,4_000_000) {
        Ok(request)=>weft_runtime::prepare_security_mapping_json(&request),
        Err(_)=>weft_runtime::prepare_security_mapping_json(""),
    }
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
pub fn prepare_application_lineage_json(request:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{
 weft_runtime::prepare_application_lineage_json(&strict_text(&request,4_000_000)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
}

/// Source-bound designation only. It cannot authorize a native operation.
#[cfg(feature="application-lineage-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn prepare_current_query_designations_json(request:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{
 weft_runtime::prepare_current_query_designations_json(&strict_text(&request,4_000_000)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
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

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_raw_read_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue> { let original=strict_text(request,4_194_304)?;Ok(weft_runtime::compile_raw_read_json(&original)) }

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_native_action_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue> { let original=strict_text(request,4_194_304)?;Ok(weft_runtime::compile_native_action_json(&original)) }

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_native_bulk_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue> { let original=strict_text(request,4_194_304)?;Ok(weft_runtime::compile_native_bulk_json(&original)) }

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_search_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue> { let original=strict_text(request,4_194_304)?;Ok(weft_runtime::compile_search_json(&original)) }
/// Validate the primitive JS string before wasm-bindgen can replace unpaired
/// UTF-16 units. Work and allocation are bounded by the actual framed input.
fn strict_text(value:&js_sys::JsString,maximum:usize)->Result<String,wasm_bindgen::JsValue>{
    let fail=||wasm_bindgen::JsValue::from_str("WFT-CURRENT-QUERY-TEXT: Original Unicode text refused");
    let raw:&wasm_bindgen::JsValue=value.as_ref();if !raw.is_string(){return Err(fail());}
    let length=value.length();if length as usize>maximum{return Err(fail());}
    let mut i=0;let mut bytes=0usize;
    while i<length {
        let unit=value.char_code_at(i) as u16;
        let width=if (0xd800..=0xdbff).contains(&unit){
            if i+1>=length || !(0xdc00..=0xdfff).contains(&(value.char_code_at(i+1) as u16)){return Err(fail());}
            i+=1;4
        }else if (0xdc00..=0xdfff).contains(&unit){return Err(fail());}
        else if unit<=0x7f {1}else if unit<=0x7ff {2}else {3};
        bytes=bytes.checked_add(width).ok_or_else(fail)?;if bytes>maximum{return Err(fail());}i+=1;
    }
    let text=raw.as_string().ok_or_else(fail)?;if text.len()!=bytes{return Err(fail());}Ok(text)
}

/// Original source preparation object; metadata never confers native authority.
#[cfg(feature="bounded-current-string-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub struct ApplicationLineagePreparation {inner:weft_runtime::ApplicationLineagePreparation}
#[cfg(feature="bounded-current-string-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn prepare_application_lineage(request:js_sys::JsString)->Result<ApplicationLineagePreparation,wasm_bindgen::JsValue>{
    let text=strict_text(&request,4_000_000)?;
    weft_runtime::prepare_application_lineage(&text).map(|inner|ApplicationLineagePreparation{inner}).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))
}
#[cfg(feature="bounded-current-string-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl ApplicationLineagePreparation {
    pub fn json(&self)->String{self.inner.json().to_owned()}
    pub fn prepare_current_query(&self)->Result<CurrentQueryPreparation,wasm_bindgen::JsValue>{self.inner.prepare_current_query().map(|inner|CurrentQueryPreparation{inner}).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
}
#[cfg(feature="bounded-current-string-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub struct CurrentQueryPreparation {inner:weft_runtime::CurrentQueryPreparation}
#[cfg(feature="bounded-current-string-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
impl CurrentQueryPreparation {
    pub fn lower_postgresql_bounded_current_rows_candidate_json(&self,binding:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{self.inner.lower_postgresql_bounded_current_rows_candidate_json(&strict_text(&binding,4_000_000)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
    pub fn check_postgresql_bounded_current_rows_candidate_json(&self,binding:js_sys::JsString,input:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{self.inner.check_postgresql_bounded_current_rows_candidate_json(&strict_text(&binding,4_000_000)?,&strict_text(&input,16_777_216)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
    pub fn lower_postgresql_bounded_current_contributors_candidate_json(&self,binding:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{self.inner.lower_postgresql_bounded_current_contributors_candidate_json(&strict_text(&binding,4_000_000)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
    pub fn check_postgresql_bounded_current_contributors_candidate_json(&self,binding:js_sys::JsString,input:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{self.inner.check_postgresql_bounded_current_contributors_candidate_json(&strict_text(&binding,4_000_000)?,&strict_text(&input,16_777_216)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
    pub fn lower_databricks_bounded_current_contributors_candidate_json(&self,binding:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{self.inner.lower_databricks_bounded_current_contributors_candidate_json(&strict_text(&binding,4_000_000)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
    pub fn check_databricks_bounded_current_contributors_candidate_json(&self,binding:js_sys::JsString,input:js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{self.inner.check_databricks_bounded_current_contributors_candidate_json(&strict_text(&binding,4_000_000)?,&strict_text(&input,16_777_216)?).map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
    pub fn is_from(&self,original:&ApplicationLineagePreparation)->bool{self.inner.is_from(&original.inner)}
    pub fn designation_json(&self)->String{self.inner.designation_json().to_owned()}
    pub fn native_request_json(&self)->Result<String,wasm_bindgen::JsValue>{self.inner.native_request_json().map_err(|d|wasm_bindgen::JsValue::from_str(&format!("{}: {}",d.code,d.message)))}
}

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_engine_read_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{let original=strict_text(request,4_194_304)?;Ok(weft_runtime::engine_read::compile_json(&original))}
#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn check_engine_read_json(program:&js_sys::JsString,current_request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{let program=strict_text(program,4_194_304)?;let request=strict_text(current_request,4_194_304)?;Ok(weft_runtime::engine_read::check_json(&program,&request))}

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_engine_page_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{Ok(weft_runtime::engine_read::compile_page_json(&strict_text(request,4_194_304)?))}
#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn check_engine_page_json(program:&js_sys::JsString,current_request:&js_sys::JsString,after_json:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{Ok(weft_runtime::engine_read::check_page_json(&strict_text(program,4_194_304)?,&strict_text(current_request,4_194_304)?,&strict_text(after_json,1_048_576)?))}

#[cfg(feature="raw-postgresql-reads")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_engine_export_json(program:&js_sys::JsString,current_request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{Ok(weft_runtime::engine_export::compile_json(&strict_text(program,4_194_304)?,&strict_text(current_request,4_194_304)?))}

#[cfg(feature="managed-delta-mutation-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_managed_action_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{let original=strict_text(request,4_194_304)?;Ok(weft_runtime::compile_managed_action_json(&original))}

#[cfg(feature="managed-delta-mutation-candidate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn compile_managed_bulk_json(request:&js_sys::JsString)->Result<String,wasm_bindgen::JsValue>{let original=strict_text(request,4_194_304)?;Ok(weft_runtime::compile_managed_bulk_json(&original))}
