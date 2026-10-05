#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
spike_python=${WEFT_PYTHON:-python3}
cargo fmt --all -- --check
cargo test -p weft-core --locked
cargo build -p weft-core --locked
"$spike_python" tests/frontend/oracle.py
bun tests/frontend/schema-check.ts
cargo build -p weft-frontend-probe --target wasm32-unknown-unknown --locked
wasm-bindgen target/wasm32-unknown-unknown/debug/weft_frontend_probe.wasm --target web --out-dir target/b002/web
node tests/frontend/browser-check.mjs
bun run specs:check
