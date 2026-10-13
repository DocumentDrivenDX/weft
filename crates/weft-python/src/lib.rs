use pyo3::prelude::*;
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
    m.add_function(wrap_pyfunction!(compile_json, m)?)?;
    #[cfg(feature="truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_runtime_json,m)?)?;
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
