//! Pure build-time composition. Hosts opt into trusted candidate adapters.
pub fn compile_json(request: &str) -> String {
    #[cfg(feature = "test-third")]
    let registry = weft_backend_probe::compile_fixture_registry();
    #[cfg(not(feature = "test-third"))]
    let registry = weft_core::backend::Registry::default();
    #[cfg(feature = "truss-postgresql-candidate")]
    let registry = {
        let mut registry = registry;
        registry
            .register(weft_postgresql::candidate::Candidate)
            .expect("build-time candidate backend registration must be unique");
        registry
    };
    weft_core::compile::Compiler { registry }.compile_json(request)
}
