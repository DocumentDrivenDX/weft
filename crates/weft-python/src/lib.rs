use pyo3::prelude::*;
/// Original ontology Subject/Record/Key source selection only; no authority.
#[pyfunction]
fn inspect_security_mapping_subject_key_json(request: &str, selection: &str) -> String {
    weft_runtime::inspect_security_mapping_subject_key_json(request, selection)
}
#[pyfunction]
fn prepare_security_mapping_json(request: &str) -> String {
    weft_runtime::prepare_security_mapping_json(request)
}
#[pyfunction]
fn compile_json(request: &str) -> String {
    weft_runtime::compile_json(request)
}
/// Explicit PathsKeys transport; generic compile_json retains its original composition.
#[cfg(feature = "ashlar-databricks-paths-keys")]
#[pyfunction]
fn compile_paths_keys_json(request: &str) -> String {
    weft_runtime::paths_keys::compile_json(request)
}
#[cfg(feature = "test-original")]
#[pyfunction]
fn compile_json_with_conformance_configuration(request: &str, configuration: &str) -> String {
    weft_runtime::compile_json_with_conformance_configuration(request, configuration)
}
/// Host-selected production Truss PG18 composition, not a conformance feature.
#[cfg(feature="truss-postgresql-pg18")]
#[pyfunction]
fn compile_truss_postgresql_json(request:&str,composition:&str)->String{
    weft_runtime::compile_truss_postgresql_json(request,composition)
}
#[pymodule]
fn weft(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<MutationCompiler>()?;
    #[cfg(feature="truss-security-relationship-candidate")]
    {
        m.add_function(wrap_pyfunction!(prepare_truss_security_relationship_candidate_json,m)?)?;
        m.add_function(wrap_pyfunction!(inspect_truss_security_relationship_result_json,m)?)?;
    }
    m.add_function(wrap_pyfunction!(inspect_security_mapping_subject_key_json, m)?)?;
    m.add_function(wrap_pyfunction!(prepare_security_mapping_json, m)?)?;
    m.add_function(wrap_pyfunction!(compile_json, m)?)?;
    #[cfg(feature="application-lineage-candidate")]
    {
        m.add_function(wrap_pyfunction!(prepare_application_lineage_json,m)?)?;
        m.add_function(wrap_pyfunction!(prepare_application_lineage,m)?)?;
        m.add_class::<ApplicationLineagePreparation>()?;
    }
    #[cfg(feature="truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_runtime_json,m)?)?;
    #[cfg(feature="truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_runtime_page_json,m)?)?;
    #[cfg(feature="truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_postgresql_json,m)?)?;
    #[cfg(feature = "ashlar-databricks-paths-keys")]
    m.add_function(wrap_pyfunction!(compile_paths_keys_json, m)?)?;
    #[cfg(feature = "test-original")]
    m.add_function(wrap_pyfunction!(
        compile_json_with_conformance_configuration,
        m
    )?)?;
    #[cfg(feature="ashlar-databricks-implicit-arithmetic")]
    m.add_function(wrap_pyfunction!(compile_implicit_arithmetic_json, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

#[cfg(feature="ashlar-databricks-implicit-arithmetic")]
#[pyfunction]
fn compile_implicit_arithmetic_json(request: &str) -> String {
    weft_runtime::implicit_arithmetic::compile_json(request)
}

#[cfg(feature="truss-postgresql-pg18")]
#[pyfunction]
fn compile_truss_runtime_json(request:&str)->String{weft_runtime::compile_truss_runtime_json(request)}

#[cfg(feature="truss-postgresql-pg18")]
#[pyfunction]
#[pyo3(signature=(request,page_size,after=None))]
fn compile_truss_runtime_page_json(request:&str,page_size:u16,after:Option<Vec<String>>)->String{
    weft_runtime::compile_truss_runtime_page_json(request,page_size,after)
}

/// Host-owned original mutation/action registry. This class executes no database effects.
#[pyclass]
struct MutationCompiler {registry:weft_runtime::MutationRegistry}
fn mutation_error(e:impl std::fmt::Display)->PyErr{pyo3::exceptions::PyValueError::new_err(e.to_string())}
#[pymethods]
impl MutationCompiler {
 #[new]
 fn new(configuration:&str)->PyResult<Self>{
  let registry=weft_runtime::MutationRegistry::from_json(configuration).map_err(|e|mutation_error(format!("{}: {}",e.code,e.message)))?;Ok(Self{registry})
 }
 fn compile_json(&self,request:&str)->String{self.registry.compile_json(request)}
 #[cfg(feature="truss-postgresql-pg18")]
 fn register_truss_selection(&mut self,configuration:&str)->PyResult<usize>{weft_runtime::install_truss_mutation_backend(&mut self.registry,configuration).map_err(|e|mutation_error(format!("{}: {}",e.code,e.message)))}
 fn lower_selection_json(&self,handle:usize,request:&str)->PyResult<String>{weft_runtime::lower_mutation_selection_json(&self.registry,handle,request).map_err(|e|mutation_error(format!("{}: {}",e.code,e.message)))}
}

#[cfg(feature="truss-security-relationship-candidate")]
#[pyfunction]
fn prepare_truss_security_relationship_candidate_json(request:&str)->PyResult<String>{
 weft_runtime::prepare_truss_security_relationship_candidate_json(request).map_err(|d|pyo3::exceptions::PyValueError::new_err(format!("{}: {}",d.code,d.message)))
}
#[cfg(feature="truss-security-relationship-candidate")]
#[pyfunction]
fn inspect_truss_security_relationship_result_json(request:&str)->PyResult<String>{
 weft_runtime::inspect_truss_security_relationship_result_json(request).map_err(|d|pyo3::exceptions::PyValueError::new_err(format!("{}: {}",d.code,d.message)))
}

/// Opt-in source lineage candidate; no native authorization or execution.
#[cfg(feature="application-lineage-candidate")]
#[pyfunction]
fn prepare_application_lineage_json(request:&str)->PyResult<String>{
 weft_runtime::prepare_application_lineage_json(request).map_err(|d|pyo3::exceptions::PyValueError::new_err(format!("{}: {}",d.code,d.message)))
}

/// Core-issued immutable preparation. Actual source/current/native admission remains external.
#[cfg(feature="application-lineage-candidate")]
#[pyclass(frozen,module="weft")]
struct ApplicationLineagePreparation {inner:weft_runtime::ApplicationLineagePreparation}
#[cfg(feature="application-lineage-candidate")]
#[pymethods]
impl ApplicationLineagePreparation {
 fn json(&self)->&str{self.inner.json()}
 fn source_request_json(&self)->&str{self.inner.source_request_json()}
}
#[cfg(feature="application-lineage-candidate")]
#[pyfunction]
fn prepare_application_lineage(request:&str)->PyResult<ApplicationLineagePreparation>{
 weft_runtime::prepare_application_lineage(request)
  .map(|inner|ApplicationLineagePreparation{inner})
  .map_err(|d|pyo3::exceptions::PyValueError::new_err(format!("{}: {}",d.code,d.message)))
}
