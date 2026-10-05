use pyo3::prelude::*;
#[pyfunction]
fn compile_json(request: &str) -> String {
    weft_spike_core::compile_json(request)
}
#[pymodule]
fn weft_spike(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(compile_json, m)?)?;
    Ok(())
}
