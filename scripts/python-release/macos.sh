#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/../.."
wheel=(artifacts/python-release-final/*macosx_11_0_arm64.whl)
[[ ${#wheel[@]} == 1 ]]
: "${WEFT_PYTHON_INSTALL_DIR:?Set the managed Python installation directory}"
: "${WEFT_RELEASE_VENV_DIR:?Set the temporary venv directory}"
for python in "$WEFT_PYTHON_INSTALL_DIR"/*/bin/python3; do
  version=$($python -c 'import sys;print("%d%d"%sys.version_info[:2])')
  venv=$WEFT_RELEASE_VENV_DIR/macos-py$version
  "$python" -m venv "$venv"
  "$venv/bin/python" -m pip install --no-index "${wheel[0]}"
  "$venv/bin/python" scripts/python-release/validate.py --wheel "${wheel[0]}" --out "artifacts/python-release-final/macos-arm64-py$version.json"
done
