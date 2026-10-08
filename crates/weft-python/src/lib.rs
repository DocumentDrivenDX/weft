use pyo3::prelude::*;
#[pyfunction]
fn compile_json(request: &str) -> String {
    weft_runtime::compile_json(request)
}
#[cfg(feature = "test-original")]
#[pyfunction]
fn compile_json_with_conformance_configuration(request: &str, configuration: &str) -> String {
    weft_runtime::compile_json_with_conformance_configuration(request, configuration)
}
#[pymodule]
fn weft(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_json, m)?)?;
    #[cfg(feature = "test-original")]
    m.add_function(wrap_pyfunction!(
        compile_json_with_conformance_configuration,
        m
    )?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
