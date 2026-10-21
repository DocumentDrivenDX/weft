#!/usr/bin/env bash
set -euo pipefail

expected_sha="${1:?usage: build-selected-macos-0.4.49.sh <reviewed-source-sha> [output-dir]}"
output_dir="${2:-dist-selected-0.4.49}"
python_bin="${PYTHON_BIN:-python3.12}"
root="$(git rev-parse --show-toplevel)"
cd "$root"
test "$(git rev-parse HEAD)" = "$expected_sha"
git symbolic-ref --short -q HEAD >/dev/null
test -z "$(git status --porcelain)"
test "$(uname -s)" = Darwin
test "$(uname -m)" = arm64
test "$(rustc --version | awk '{print $2}')" = 1.90.0
test "$(maturin --version | awk '{print $2}')" = 1.9.6
command -v "$python_bin" >/dev/null
"$python_bin" - <<'PYTHON'
import tomllib
assert tomllib.load(open('crates/weft-python/pyproject.toml','rb'))['project']['version']=='0.4.49'
assert tomllib.load(open('crates/weft-python/Cargo.toml','rb'))['package']['version']=='0.4.49'
PYTHON

test ! -e "$output_dir"
mkdir -p "$output_dir"
maturin build --manifest-path crates/weft-python/Cargo.toml --release --locked --out "$output_dir"
check_dir="$(mktemp -d)"
trap 'rm -rf "$check_dir"' EXIT
"$python_bin" -m venv "$check_dir/venv"
wheel="$(find "$output_dir" -maxdepth 1 -name 'weft_sql-0.4.49-*-macosx_*_arm64.whl' -print -quit)"
test -n "$wheel"
test "$(find "$output_dir" -maxdepth 1 -name 'weft_sql-0.4.49-*-macosx_*_arm64.whl' | wc -l | tr -d ' ')" = 1
"$check_dir/venv/bin/python" -m pip install --no-deps "$wheel"
"$check_dir/venv/bin/python" scripts/release/verify-wheel-packaging.py --wheel "$wheel" --version 0.4.49 --output "$output_dir/packaging-macos-arm64.json"
"$python_bin" scripts/release/enrich-selected-wheel-report.py \
  --wheel "$wheel" \
  --packaging-report "$output_dir/packaging-macos-arm64.json" \
  --build-report "$output_dir/selected-macos-build-0.4.49.json" \
  --source-commit "$expected_sha" --target aarch64-apple-darwin
printf 'Wheel and selected-macos-build-0.4.49.json are in %s\n' "$output_dir"
