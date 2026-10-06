#!/bin/sh
# Requires fresh target/b005/application-native-reports.json from application-native.py.
set -eu
cd "$(dirname "$0")/.."
probe_python=${WEFT_PYTHON:-python3}
maturin build --manifest-path crates/weft-python/Cargo.toml --features truss-postgresql-candidate --locked --out target/b005/test-wheels
"$probe_python" -m pip install --force-reinstall --no-deps target/b005/test-wheels/*.whl
"$probe_python" tests/truss-postgresql/python-check.py
cargo build -p weft-wasm --features truss-postgresql-candidate --target wasm32-unknown-unknown --locked
wasm-bindgen target/wasm32-unknown-unknown/debug/weft_wasm.wasm --target web --out-dir target/b005/web
bun build packages/weft-browser/src/index.ts --target browser --outdir target/b004/wrapper
WEFT_PROBE_JS=target/b005/web/weft_wasm.js \
WEFT_PROBE_WASM=target/b005/web/weft_wasm_bg.wasm \
WEFT_FRONTEND_CORPUS=tests/truss-postgresql/fixtures/application-cases.json \
WEFT_FRONTEND_REPORTS=target/b005/application-native-reports.json \
WEFT_FRONTEND_BROWSER_SUMMARY=target/b005/browser-summary.json \
node tests/compile/browser-check.mjs
