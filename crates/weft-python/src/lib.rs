// SPDX-License-Identifier: MIT OR Apache-2.0
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
#[cfg(feature = "truss-postgresql-pg18")]
#[pyfunction]
fn compile_truss_postgresql_json(request: &str, composition: &str) -> String {
    weft_runtime::compile_truss_postgresql_json(request, composition)
}
#[pymodule]
fn weft(m: &Bound<'_, PyModule>) -> PyResult<()> {
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_raw_read_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_table_operation_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(prepare_table_operation_module_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_class::<PreparedTableOperationModule>()?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_class::<PreparedTableOperation>()?;
    #[cfg(feature="raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_table_seed_json, m)?)?;
    #[cfg(feature="raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(check_table_seed_json, m)?)?;
    #[cfg(feature="raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(check_table_operation_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(prepare_table_query_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_class::<PreparedTableQuery>()?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(prepare_table_query_module_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_class::<PreparedTableQueryModule>()?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_native_privilege_probes_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(check_native_privilege_probes_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_related_read_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_native_action_json, m)?)?;
    #[cfg(feature = "managed-delta-mutation-candidate")]
    m.add_function(wrap_pyfunction!(compile_managed_action_json, m)?)?;
    #[cfg(feature = "managed-delta-mutation-candidate")]
    m.add_function(wrap_pyfunction!(compile_managed_bulk_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_native_bulk_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_search_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_unity_search_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_engine_read_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_engine_related_read_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_engine_related_page_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(check_engine_read_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_engine_page_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(check_engine_page_json, m)?)?;
    #[cfg(feature = "raw-postgresql-reads")]
    m.add_function(wrap_pyfunction!(compile_engine_export_json, m)?)?;
    m.add_class::<MutationCompiler>()?;
    #[cfg(feature = "truss-security-relationship-candidate")]
    {
        m.add_function(wrap_pyfunction!(
            prepare_truss_security_relationship_candidate_json,
            m
        )?)?;
        m.add_function(wrap_pyfunction!(
            inspect_truss_security_relationship_result_json,
            m
        )?)?;
    }
    m.add_function(wrap_pyfunction!(
        inspect_security_mapping_subject_key_json,
        m
    )?)?;
    m.add_function(wrap_pyfunction!(prepare_security_mapping_json, m)?)?;
    m.add_function(wrap_pyfunction!(compile_json, m)?)?;
    #[cfg(feature = "application-lineage-candidate")]
    {
        m.add_function(wrap_pyfunction!(prepare_application_lineage_json, m)?)?;
        m.add_function(wrap_pyfunction!(prepare_application_lineage, m)?)?;
        m.add_class::<ApplicationLineagePreparation>()?;
        m.add_class::<CurrentQueryPreparation>()?;
    }
    #[cfg(feature = "current-presence-candidate")]
    m.add_class::<CurrentPresencePreparation>()?;
    #[cfg(feature = "truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_runtime_json, m)?)?;
    #[cfg(feature = "truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_runtime_page_json, m)?)?;
    #[cfg(feature = "truss-postgresql-pg18")]
    m.add_function(wrap_pyfunction!(compile_truss_postgresql_json, m)?)?;
    #[cfg(feature = "ashlar-databricks-paths-keys")]
    m.add_function(wrap_pyfunction!(compile_paths_keys_json, m)?)?;
    #[cfg(feature = "test-original")]
    m.add_function(wrap_pyfunction!(
        compile_json_with_conformance_configuration,
        m
    )?)?;
    #[cfg(feature = "ashlar-databricks-implicit-arithmetic")]
    m.add_function(wrap_pyfunction!(compile_implicit_arithmetic_json, m)?)?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}

#[cfg(feature = "ashlar-databricks-implicit-arithmetic")]
#[pyfunction]
fn compile_implicit_arithmetic_json(request: &str) -> String {
    weft_runtime::implicit_arithmetic::compile_json(request)
}

#[cfg(feature = "truss-postgresql-pg18")]
#[pyfunction]
fn compile_truss_runtime_json(request: &str) -> String {
    weft_runtime::compile_truss_runtime_json(request)
}

#[cfg(feature = "truss-postgresql-pg18")]
#[pyfunction]
#[pyo3(signature=(request,page_size,after=None))]
fn compile_truss_runtime_page_json(
    request: &str,
    page_size: u16,
    after: Option<Vec<String>>,
) -> String {
    weft_runtime::compile_truss_runtime_page_json(request, page_size, after)
}

/// Host-owned original mutation/action registry. This class executes no database effects.
#[pyclass]
struct MutationCompiler {
    registry: weft_runtime::MutationRegistry,
}
fn mutation_error(e: impl std::fmt::Display) -> PyErr {
    pyo3::exceptions::PyValueError::new_err(e.to_string())
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyclass(frozen, module = "weft")]
struct PreparedTableQuery {
    inner: weft_runtime::PreparedTableQuery,
}
#[cfg(feature = "raw-postgresql-reads")]
#[pymethods]
impl PreparedTableQuery {
    fn bind_json(&self, call: &str) -> String {
        self.inner.bind_json(call)
    }
    fn program_json(&self) -> String { self.inner.program_json() }
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
fn prepare_table_query_json(request: &str) -> PyResult<PreparedTableQuery> {
    let inner=weft_runtime::PreparedTableQuery::prepare(request).map_err(mutation_error)?;
    Ok(PreparedTableQuery { inner })
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyclass(frozen, module = "weft")]
struct PreparedTableQueryModule { inner: weft_runtime::PreparedTableQueryModule }
#[cfg(feature = "raw-postgresql-reads")]
#[pymethods]
impl PreparedTableQueryModule {
    fn prepare(&self, operation: &str) -> PyResult<PreparedTableQuery> {
        let inner = self.inner.prepare_operation(operation).map_err(mutation_error)?;
        Ok(PreparedTableQuery { inner })
    }
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
fn prepare_table_query_module_json(request: &str) -> PyResult<PreparedTableQueryModule> {
    let inner = weft_runtime::PreparedTableQueryModule::prepare(request).map_err(mutation_error)?;
    Ok(PreparedTableQueryModule { inner })
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyclass(frozen, module = "weft")]
struct PreparedTableOperationModule { inner: weft_runtime::PreparedTableOperationModule }
#[cfg(feature = "raw-postgresql-reads")]
#[pymethods]
impl PreparedTableOperationModule {
    fn prepare(&self, operation: &str) -> PyResult<PreparedTableOperation> {
        let inner = self.inner.prepare_operation(operation).map_err(mutation_error)?;
        Ok(PreparedTableOperation { inner })
    }
    fn prepare_query(&self, operation: &str) -> PyResult<PreparedTableQuery> {
        let inner = self.inner.prepare_query(operation).map_err(mutation_error)?;
        Ok(PreparedTableQuery { inner })
    }
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyclass(frozen, module = "weft")]
struct PreparedTableOperation { inner: weft_runtime::PreparedTableOperation }
#[cfg(feature = "raw-postgresql-reads")]
#[pymethods]
impl PreparedTableOperation {
    fn bind_json(&self, call: &str) -> String { self.inner.bind_json(call) }
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
fn prepare_table_operation_module_json(request: &str) -> PyResult<PreparedTableOperationModule> {
    let inner = weft_runtime::PreparedTableOperationModule::prepare(request).map_err(mutation_error)?;
    Ok(PreparedTableOperationModule { inner })
}
#[pymethods]
impl MutationCompiler {
    #[new]
    fn new(configuration: &str) -> PyResult<Self> {
        let registry = weft_runtime::MutationRegistry::from_json(configuration)
            .map_err(|e| mutation_error(format!("{}: {}", e.code, e.message)))?;
        Ok(Self { registry })
    }
    fn compile_json(&self, request: &str) -> String {
        self.registry.compile_json(request)
    }
    #[cfg(feature = "truss-postgresql-pg18")]
    fn register_truss_selection(&mut self, configuration: &str) -> PyResult<usize> {
        weft_runtime::install_truss_mutation_backend(&mut self.registry, configuration)
            .map_err(|e| mutation_error(format!("{}: {}", e.code, e.message)))
    }
    fn lower_selection_json(&self, handle: usize, request: &str) -> PyResult<String> {
        weft_runtime::lower_mutation_selection_json(&self.registry, handle, request)
            .map_err(|e| mutation_error(format!("{}: {}", e.code, e.message)))
    }
}

#[cfg(feature = "truss-security-relationship-candidate")]
#[pyfunction]
fn prepare_truss_security_relationship_candidate_json(request: &str) -> PyResult<String> {
    weft_runtime::prepare_truss_security_relationship_candidate_json(request)
        .map_err(|d| pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message)))
}
#[cfg(feature = "truss-security-relationship-candidate")]
#[pyfunction]
fn inspect_truss_security_relationship_result_json(request: &str) -> PyResult<String> {
    weft_runtime::inspect_truss_security_relationship_result_json(request)
        .map_err(|d| pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message)))
}

/// Opt-in source lineage candidate; no native authorization or execution.
#[cfg(feature = "application-lineage-candidate")]
#[pyfunction]
fn prepare_application_lineage_json(request: &str) -> PyResult<String> {
    weft_runtime::prepare_application_lineage_json(request)
        .map_err(|d| pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message)))
}

/// Core-issued immutable preparation. Actual source/current/native admission remains external.
#[cfg(feature = "application-lineage-candidate")]
#[pyclass(frozen, module = "weft")]
struct ApplicationLineagePreparation {
    inner: weft_runtime::ApplicationLineagePreparation,
}
#[cfg(feature = "application-lineage-candidate")]
#[pymethods]
impl ApplicationLineagePreparation {
    fn json(&self) -> &str {
        self.inner.json()
    }
    fn source_request_json(&self) -> &str {
        self.inner.source_request_json()
    }
    fn current_query_designations_json(&self) -> PyResult<String> {
        self.inner.current_query_designations_json().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })
    }
    #[cfg(feature = "current-presence-candidate")]
    fn prepare_current_presence(slf: PyRef<'_, Self>) -> PyResult<CurrentPresencePreparation> {
        let inner = slf.inner.prepare_current_presence().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })?;
        Ok(CurrentPresencePreparation {
            original: slf.into(),
            inner,
        })
    }
    fn prepare_current_query(slf: PyRef<'_, Self>) -> PyResult<CurrentQueryPreparation> {
        let inner = slf.inner.prepare_current_query().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })?;
        Ok(CurrentQueryPreparation {
            original: slf.into(),
            inner,
        })
    }
}
#[cfg(feature = "application-lineage-candidate")]
#[pyfunction]
fn prepare_application_lineage(request: &str) -> PyResult<ApplicationLineagePreparation> {
    weft_runtime::prepare_application_lineage(request)
        .map(|inner| ApplicationLineagePreparation { inner })
        .map_err(|d| pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message)))
}

/// Non-importable original-object custody. Possession establishes no native authority.
#[cfg(feature = "application-lineage-candidate")]
#[pyclass(frozen, module = "weft")]
struct CurrentQueryPreparation {
    original: Py<ApplicationLineagePreparation>,
    inner: weft_runtime::CurrentQueryPreparation,
}
#[cfg(feature = "application-lineage-candidate")]
#[pymethods]
impl CurrentQueryPreparation {
    fn source_preparation(&self, py: Python<'_>) -> PyResult<Py<ApplicationLineagePreparation>> {
        self.check_original(py)?;
        Ok(self.original.clone_ref(py))
    }
    fn designation_json(&self, py: Python<'_>) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .with_context(|_, designation, _| Ok(designation.json().to_owned()))
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    fn lower_postgresql_current_rows_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_postgresql_current_rows_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    fn lower_postgresql_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_postgresql_current_contributors_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-postgresql-scalar-candidate")]
    fn lower_postgresql_current_sum_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_postgresql_current_sum_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-databricks-scalar-candidate")]
    fn lower_databricks_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_databricks_current_contributors_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-databricks-scalar-candidate")]
    fn lower_databricks_current_sum_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_databricks_current_sum_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-spark-scalar-candidate")]
    fn lower_spark_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_spark_current_contributors_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    #[cfg(feature = "current-spark-scalar-candidate")]
    fn check_spark_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_spark_current_contributors_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "current-query-refinement-candidate")]
    fn check_postgresql_current_rows_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_postgresql_current_rows_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "current-query-refinement-candidate")]
    fn check_postgresql_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_postgresql_current_contributors_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "current-query-refinement-candidate")]
    fn check_postgresql_current_sum_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_postgresql_current_sum_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "current-query-refinement-candidate")]
    fn check_databricks_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_databricks_current_contributors_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "current-query-refinement-candidate")]
    fn check_databricks_current_sum_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_databricks_current_sum_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    fn lower_postgresql_bounded_current_rows_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        self.inner
            .lower_postgresql_bounded_current_rows_candidate_json(&binding)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    fn check_postgresql_bounded_current_rows_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_postgresql_bounded_current_rows_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    fn lower_postgresql_bounded_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        self.inner
            .lower_postgresql_bounded_current_contributors_candidate_json(&binding)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    fn check_postgresql_bounded_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_postgresql_bounded_current_contributors_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    fn lower_databricks_bounded_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        self.inner
            .lower_databricks_bounded_current_contributors_candidate_json(&binding)
            .map_err(|_| refinement_python_failure())
    }
    #[cfg(feature = "bounded-current-string-candidate")]
    fn check_databricks_bounded_current_contributors_candidate_json(
        &self,
        py: Python<'_>,
        original_binding_json: &Bound<'_, PyAny>,
        refinement_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)
            .map_err(|_| refinement_python_failure())?;
        let binding = refinement_text(original_binding_json, 4_000_000)?;
        let input = refinement_text(refinement_input_json, 16_777_216)?;
        self.inner
            .check_databricks_bounded_current_contributors_candidate_json(&binding, &input)
            .map_err(|_| refinement_python_failure())
    }
    fn operand_count(&self, py: Python<'_>) -> PyResult<usize> {
        self.check_original(py)?;
        Ok(self.inner.operand_count())
    }
    fn native_request_json(&self, py: Python<'_>) -> PyResult<String> {
        self.check_original(py)?;
        self.inner.native_request_json().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })
    }

    #[cfg(feature = "current-postgresql-scalar-candidate")]
    fn lower_postgresql_scalar_candidate_json(
        &self,
        py: Python<'_>,
        binding: &str,
    ) -> PyResult<String> {
        self.check_original(py)?;
        self.inner
            .lower_postgresql_scalar_candidate_json(binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
}

#[cfg(feature = "application-lineage-candidate")]
impl CurrentQueryPreparation {
    fn check_original(&self, py: Python<'_>) -> PyResult<()> {
        if !self.inner.is_from(&self.original.bind(py).borrow().inner) {
            return Err(pyo3::exceptions::PyValueError::new_err(
                "WFT-CURRENT-QUERY-CONTEXT: Original current query context refused",
            ));
        }
        Ok(())
    }
}

#[cfg(feature = "current-query-refinement-candidate")]
fn refinement_python_failure() -> PyErr {
    pyo3::exceptions::PyValueError::new_err(
        "WFT-CURRENT-QUERY-REFINEMENT: Conditional current-query refinement refused",
    )
}
#[cfg(feature = "current-query-refinement-candidate")]
fn refinement_text(value: &Bound<'_, PyAny>, maximum: usize) -> PyResult<String> {
    if !value.is_exact_instance_of::<pyo3::types::PyString>()
        || value.len().map_err(|_| refinement_python_failure())? > maximum
    {
        return Err(refinement_python_failure());
    }
    let text = value
        .extract::<String>()
        .map_err(|_| refinement_python_failure())?;
    if text.len() > maximum {
        return Err(refinement_python_failure());
    }
    Ok(text)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_raw_read_json(request: &str) -> String {
    weft_runtime::compile_raw_read_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_native_privilege_probes_json(request: &str) -> String {
    weft_runtime::compile_native_privilege_probes_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn check_native_privilege_probes_json(program: &str) -> String {
    weft_runtime::check_native_privilege_probes_json(program)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_native_action_json(request: &str) -> String {
    weft_runtime::compile_native_action_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_native_bulk_json(request: &str) -> String {
    weft_runtime::compile_native_bulk_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_search_json(request: &str) -> String {
    weft_runtime::compile_search_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_engine_read_json(request: &str) -> String {
    weft_runtime::engine_read::compile_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_engine_related_read_json(request: &str) -> String {
    weft_runtime::engine_related_read::compile_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_engine_related_page_json(request: &str) -> String {
    weft_runtime::engine_related_read::compile_page_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn check_engine_read_json(program: &str, current_request: &str) -> String {
    weft_runtime::engine_read::check_json(program, current_request)
}
#[cfg(feature = "current-presence-candidate")]
fn presence_failure() -> PyErr {
    pyo3::exceptions::PyValueError::new_err(
        "WFT-CURRENT-PRESENCE-BRIDGE: Current presence bridge refused",
    )
}
#[cfg(feature = "current-presence-candidate")]
fn presence_text(value: &Bound<'_, PyAny>, maximum: usize) -> PyResult<String> {
    if !value.is_exact_instance_of::<pyo3::types::PyString>()
        || value.len().map_err(|_| presence_failure())? > maximum
    {
        return Err(presence_failure());
    }
    let text = value.extract::<String>().map_err(|_| presence_failure())?;
    if text.len() > maximum {
        return Err(presence_failure());
    }
    Ok(text)
}
/// Distinct issued type. No constructor, subclassing, report import or old methods.
#[cfg(feature = "current-presence-candidate")]
#[pyclass(frozen, module = "weft")]
struct CurrentPresencePreparation {
    original: Py<ApplicationLineagePreparation>,
    inner: weft_runtime::CurrentPresencePreparation,
}
#[cfg(feature = "current-presence-candidate")]
#[pymethods]
impl CurrentPresencePreparation {
    fn source_preparation(&self, py: Python<'_>) -> PyResult<Py<ApplicationLineagePreparation>> {
        self.check_original(py)?;
        Ok(self.original.clone_ref(py))
    }
    fn json(&self, py: Python<'_>) -> PyResult<String> {
        self.check_original(py)?;
        Ok(self.inner.json().to_owned())
    }
    fn native_request_json(&self, py: Python<'_>) -> PyResult<String> {
        self.check_original(py)?;
        self.inner.native_request_json().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })
    }
    #[cfg(feature = "current-presence-model-requirements-candidate")]
    fn model_requirements_json(&self, py: Python<'_>) -> PyResult<String> {
        self.check_original(py)?;
        self.inner.model_requirements_json().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })
    }
    #[cfg(feature = "current-presence-graph-requirements-candidate")]
    fn graph_requirements_json(&self, py: Python<'_>) -> PyResult<String> {
        self.check_original(py)?;
        self.inner.graph_requirements_json().map_err(|d| {
            pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
        })
    }
    fn lower_postgresql_current_presence_candidate_json(
        &self,
        py: Python<'_>,
        binding_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)?;
        let binding = presence_text(binding_json, 4_000_000)?;
        self.inner
            .lower_postgresql_current_presence_candidate_json(&binding)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
    fn check_postgresql_current_presence_candidate_json(
        &self,
        py: Python<'_>,
        binding_json: &Bound<'_, PyAny>,
        transport_input_json: &Bound<'_, PyAny>,
    ) -> PyResult<String> {
        self.check_original(py)?;
        let binding = presence_text(binding_json, 4_000_000)?;
        let input = presence_text(transport_input_json, 16_000_000)?;
        self.inner
            .check_postgresql_current_presence_candidate_json(&binding, &input)
            .map_err(|d| {
                pyo3::exceptions::PyValueError::new_err(format!("{}: {}", d.code, d.message))
            })
    }
}
#[cfg(feature = "current-presence-candidate")]
impl CurrentPresencePreparation {
    fn check_original(&self, py: Python<'_>) -> PyResult<()> {
        if !self.inner.is_from(&self.original.bind(py).borrow().inner) {
            return Err(presence_failure());
        }
        Ok(())
    }
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_engine_page_json(request: &str) -> String {
    weft_runtime::engine_read::compile_page_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn check_engine_page_json(program: &str, current_request: &str, after_json: &str) -> String {
    weft_runtime::engine_read::check_page_json(program, current_request, after_json)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_engine_export_json(program: &str, current_request: &str) -> String {
    weft_runtime::engine_export::compile_json(program, current_request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
fn compile_unity_search_json(request: &str) -> String {
    weft_runtime::compile_unity_search_json(request)
}

#[cfg(feature = "managed-delta-mutation-candidate")]
#[pyfunction]
pub fn compile_managed_action_json(request: &str) -> String {
    weft_runtime::compile_managed_action_json(request)
}
#[cfg(feature = "managed-delta-mutation-candidate")]
#[pyfunction]
pub fn compile_managed_bulk_json(request: &str) -> String {
    weft_runtime::compile_managed_bulk_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_related_read_json(request: &str) -> String {
    weft_runtime::compile_related_read_json(request)
}

#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn compile_table_operation_json(request: &str) -> String {
    weft_runtime::compile_table_operation_json(request)
}
#[cfg(feature = "raw-postgresql-reads")]
#[pyfunction]
pub fn check_table_operation_json(program:&str)->String{weft_runtime::check_table_operation_json(program)}

#[cfg(feature="raw-postgresql-reads")]
#[pyfunction]
pub fn compile_table_seed_json(request:&str)->String{weft_runtime::compile_table_seed_json(request)}
#[cfg(feature="raw-postgresql-reads")]
#[pyfunction]
pub fn check_table_seed_json(program:&str)->String{weft_runtime::check_table_seed_json(program)}
