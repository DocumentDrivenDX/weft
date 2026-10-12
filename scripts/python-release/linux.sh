#!/usr/bin/env bash
set -euo pipefail
cd /io
export RUSTUP_TOOLCHAIN=1.90.0
export CARGO_TARGET_DIR=/tmp/weft-release-target
maturin build --release --locked --manifest-path crates/weft-python/Cargo.toml --out artifacts/python-release-final --compatibility manylinux2014 --interpreter /opt/python/cp39-cp39/bin/python
wheel=(artifacts/python-release-final/*manylinux*aarch64.whl)
if [[ $(uname -m) == x86_64 ]]; then wheel=(artifacts/python-release-final/*manylinux*x86_64.whl); fi
[[ ${#wheel[@]} == 1 ]]
for abi in cp39-cp39 cp310-cp310 cp311-cp311 cp312-cp312 cp313-cp313 cp314-cp314; do
  /opt/python/$abi/bin/python -m venv /tmp/weft-$abi
  /tmp/weft-$abi/bin/python -m pip install --no-index "${wheel[0]}"
  /tmp/weft-$abi/bin/python scripts/python-release/validate.py --wheel "${wheel[0]}" --out "artifacts/python-release-final/linux-$(uname -m)-$abi.json"
done
