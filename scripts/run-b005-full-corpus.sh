#!/bin/sh
# Run after fresh native application reports and run-b005-embeddings.sh.
set -eu
cd "$(dirname "$0")/.."
probe_python=${WEFT_PYTHON:-python3}
cargo build -p weft-postgresql --example compile_probe --locked
"$probe_python" tests/truss-postgresql/full-corpus.py
WEFT_CORPUS=target/b005/full-corpus-reports.json \
WEFT_REPORTS=target/b005/full-corpus-reports.json \
WEFT_SUMMARY=target/b005/full-corpus-python-summary.json \
"$probe_python" tests/truss-postgresql/python-check.py
WEFT_PROBE_JS=target/b005/web/weft_wasm.js \
WEFT_PROBE_WASM=target/b005/web/weft_wasm_bg.wasm \
WEFT_FRONTEND_CORPUS=target/b005/full-corpus-reports.json \
WEFT_FRONTEND_REPORTS=target/b005/full-corpus-reports.json \
WEFT_FRONTEND_BROWSER_SUMMARY=target/b005/full-corpus-browser-summary.json \
node tests/compile/browser-check.mjs
