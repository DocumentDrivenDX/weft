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
#[pymodule]
fn weft(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_json, m)?)?;
    #[cfg(feature = "ashlar-databricks-paths-keys")]
    m.add_function(wrap_pyfunction!(compile_paths_keys_json, m)?)?;
    #[cfg(feature = "test-original")]
    m.add_function(wrap_pyfunction!(
        compile_json_with_conformance_configuration,
        m
    )?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
