#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
# Includes all original/application frontend checks and real-browser runs.
sh scripts/run-b002a.sh
probe_python=${WEFT_PYTHON:-python3}
cargo build -p weft-backend-probe --locked
"$probe_python" tests/register-backend/oracle.py
bun tests/register-backend/schema-check.ts
cargo build -p weft-backend-probe --lib --target wasm32-unknown-unknown --locked
wasm-bindgen target/wasm32-unknown-unknown/debug/weft_backend_probe.wasm --target web --out-dir target/b003/web
WEFT_PROBE_API=backend_json \
WEFT_PROBE_JS=target/b003/web/weft_backend_probe.js \
WEFT_PROBE_WASM=target/b003/web/weft_backend_probe_bg.wasm \
WEFT_FRONTEND_CORPUS=tests/register-backend/fixtures/cases.json \
WEFT_FRONTEND_REPORTS=target/b003/reports.json \
WEFT_FRONTEND_BROWSER_SUMMARY=target/b003/browser-summary.json \
node tests/frontend/browser-check.mjs
