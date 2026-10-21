# Hohfeld compiler performance evidence — 2026-10-21

The final source freeze is recorded in `../../releases/local-macos-0.4.49.json`.
`rust-tests.log` records the 372-test release-feature campaign with Rust1.90.0,
test opt-level1, debug assertions and overflow checks enabled. Packaging identity
is `weft-hp-packaging.json`; browser identity and per-case SHA-256 comparisons are
`browser-summary.json` and `browser-receipts.json`. The shared source has no
connection, credentials or database executor.

Reproduce native artifacts with the installed release wheel:

```sh
python tests/truss-facts/check.py --reports /tmp/reports.json
cargo build --locked -p weft-wasm --features truss-postgresql-pg18 --target wasm32-unknown-unknown
wasm-bindgen --target web --out-dir /tmp/weft-web target/wasm32-unknown-unknown/debug/weft_wasm.wasm
bun build packages/weft-browser/src/index.ts --target browser --outdir /tmp/wrapper
WEFT_PROBE_JS=/tmp/weft-web/weft_wasm.js WEFT_PROBE_WASM=/tmp/weft-web/weft_wasm_bg.wasm WEFT_BROWSER_WRAPPER=/tmp/wrapper/index.js WEFT_FRONTEND_CORPUS=tests/truss-facts/cases.json WEFT_FRONTEND_REPORTS=/tmp/reports.json WEFT_PROBE_API=compile_truss_runtime_json WEFT_FRONTEND_BROWSER_SUMMARY=/tmp/browser-summary.json WEFT_BROWSER_RECEIPTS=/tmp/browser-receipts.json node tests/compile/browser-check.mjs
```

The checked-in workflow `truss-facts.yml` repeats this regression with pinned
toolchains and real Chromium. Five cases cover complete facts, a filtered
complete entity, plain fields, count and a wrong binding pin. Byte parity
includes original diagnostics and execution obligations. It establishes
compilation parity, not native execution in the browser.

Native value/authorization checks live in Truss's
`test_compiled_query_native.CompiledQueryTests`, including
`test_complete_entity_materializes_facts_and_retains_nested_proofs` and
`test_repeated_query_compiles_once_but_revoked_reader_still_refuses`.
