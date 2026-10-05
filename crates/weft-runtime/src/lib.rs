//! Pure build-time composition. Production adapters are registered in their owned slices.
pub fn compile_json(request: &str) -> String {
    #[cfg(feature = "test-third")]
    let compiler = weft_core::compile::Compiler {
        registry: weft_backend_probe::compile_fixture_registry(),
    };
    #[cfg(not(feature = "test-third"))]
    let compiler = weft_core::compile::Compiler::default();
    compiler.compile_json(request)
}
