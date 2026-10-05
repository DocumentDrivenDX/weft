#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
sh scripts/run-b003.sh
probe_python=${WEFT_PYTHON:-python3}
cargo build -p weft-runtime --features test-third --locked
"$probe_python" tests/compile/reports.py
bun tests/compile/schema-check.ts
maturin build --manifest-path crates/weft-python/Cargo.toml --features test-third --locked --out target/b004/test-wheels
"$probe_python" -m pip install --force-reinstall --no-deps target/b004/test-wheels/*.whl
"$probe_python" tests/compile/python-check.py
cargo build -p weft-wasm --features test-third --target wasm32-unknown-unknown --locked
wasm-bindgen target/wasm32-unknown-unknown/debug/weft_wasm.wasm --target web --out-dir target/b004/web
bun build packages/weft-browser/src/index.ts --target browser --outdir target/b004/wrapper
WEFT_PROBE_API=compile_json \
WEFT_PROBE_JS=target/b004/web/weft_wasm.js \
WEFT_PROBE_WASM=target/b004/web/weft_wasm_bg.wasm \
WEFT_FRONTEND_CORPUS=tests/compile/fixtures/cases.json \
WEFT_FRONTEND_REPORTS=target/b004/reports.json \
WEFT_FRONTEND_BROWSER_SUMMARY=target/b004/browser-summary.json \
node tests/compile/browser-check.mjs
