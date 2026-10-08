#!/bin/sh
# Saved native-tested artifact parity; database execution is a separate harness.
set -eu
cd "$(dirname "$0")/.."
probe_python=${WEFT_PYTHON:-python3}
cargo test -p weft-runtime --locked
cargo test -p weft-runtime --features ashlar-databricks-candidate,truss-postgresql-candidate --locked
maturin build --manifest-path crates/weft-python/Cargo.toml --features ashlar-databricks-candidate --locked --out target/b006/test-wheels
"$probe_python" -m pip install --force-reinstall --no-deps target/b006/test-wheels/*.whl
"$probe_python" tests/ashlar-databricks/python-check.py
cargo build -p weft-wasm --features ashlar-databricks-candidate --target wasm32-unknown-unknown --locked
wasm-bindgen target/wasm32-unknown-unknown/debug/weft_wasm.wasm --target web --out-dir target/b006/web
bun build packages/weft-browser/src/index.ts --target browser --outdir target/b004/wrapper
WEFT_PROBE_API=compile_json \
WEFT_PROBE_JS=target/b006/web/weft_wasm.js \
WEFT_PROBE_WASM=target/b006/web/weft_wasm_bg.wasm \
WEFT_FRONTEND_CORPUS=target/b006/embeddings/cases.json \
WEFT_FRONTEND_REPORTS=target/b006/embeddings/reports.json \
WEFT_FRONTEND_BROWSER_SUMMARY=target/b006/embeddings/browser-summary.json \
node tests/compile/browser-check.mjs
