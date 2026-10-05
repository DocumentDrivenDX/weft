#!/bin/sh
set -eu
cd "$(dirname "$0")/.."
spike_python=${WEFT_PYTHON:-python3}
[ "$(rustc --version | cut -d ' ' -f 2)" = 1.90.0 ]
[ "$(wasm-bindgen --version)" = 'wasm-bindgen 0.2.105' ]
cargo fmt --all -- --check
cargo test -p weft-spike-core --locked
cargo build -p weft-spike-core --locked
"$spike_python" -m maturin build --manifest-path spikes/b001/python/Cargo.toml --release --locked
"$spike_python" -c 'import glob,subprocess,sys; wheels=glob.glob("target/wheels/weft_spike-0.1.0-*.whl"); assert len(wheels)==1; subprocess.check_call([sys.executable,"-m","pip","install","--force-reinstall","--no-index",wheels[0]])'
cargo build -p weft-spike-wasm --target wasm32-unknown-unknown --release --locked
wasm-bindgen target/wasm32-unknown-unknown/release/weft_spike_wasm.wasm --target web --out-dir target/b001/web
"$spike_python" spikes/b001/check_native.py
PATH=/no-executables "$spike_python" spikes/b001/python_only.py
node spikes/b001/check_browser.mjs
