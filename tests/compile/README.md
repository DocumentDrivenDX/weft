# Public compile-envelope development (B-004)

B-004 component scope is complete; see its execution evidence. `envelope.rs` exercises the real pure Rust Compiler API:
strict versioned transport, atomic no-SQL refusals, exact pins/limits and compiled
0.1/0.2 artifacts through an explicitly registered fixture backend.
`generate_schemas.py` finalizes the draft response structures around B-003's
backend interface and typed 0.2 column representations.

The public Python distribution is `weft-sql`, importing `weft`; the browser WASM
crate is `weft-wasm`. Both call `weft-runtime`, which calls the same core Compiler.
Default composition currently registers no production adapter: selecting Truss
or Ashlar returns WFT-BACKEND-MISSING until their owned implementation slices.
The explicit `test-third` build feature links the fixture registry solely for
conformance testing. Request/model content cannot enable that feature or load code.

A macOS arm64 abi3 wheel built with Maturin 1.9.6 imports and calls directly on
CPython 3.12.14 with PATH empty and subprocess calls disabled. The default WASM
crate also builds. Full component evidence follows below; production and release qualification remain later slices.

The full 1,273-case public corpus now passes authored outcomes, versioned response
schemas, native CPython and Chromium byte parity. Seven emitted fixture queries
pass independent SQLite row checks. The browser transport rejects non-string and
unpaired-surrogate inputs before encoding, and a real WebAssembly `unreachable`
fixture verifies trap normalization and prevention of re-entry. A trapped wrapper
returns a fixed 0.1 blocked host diagnostic; recovery requires a fresh module/realm.
This fatal envelope cannot safely infer a request version after an engine trap.
Production adapter qualification and other wheel platforms remain unclaimed.
