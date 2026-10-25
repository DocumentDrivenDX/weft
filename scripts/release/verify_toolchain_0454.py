#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Assert the release toolchain and bind exact features and binary digests."""
import hashlib
import json
import os
import pathlib
import shutil
import subprocess
import sys
import tomllib


def command(argv):
    return subprocess.check_output(argv, text=True).strip()


def binary_hash(path):
    resolved = pathlib.Path(path).resolve()
    return {'path': str(resolved), 'sha256': hashlib.sha256(resolved.read_bytes()).hexdigest()}


if sys.version_info[:2] != (3, 12):
    raise SystemExit('Python 3.12 is required for the selected macOS producer')
versions = {
    'python': command(['python3', '--version']), 'bun': command(['bun', '--version']),
    'node': command(['node', '--version']), 'npm': command(['npm', '--version']),
    'maturin': command(['maturin', '--version']), 'rustc': command(['rustc', '+1.90.0', '--version']),
    'cargo': command(['cargo', '+1.90.0', '--version']), 'wasmBindgen': command(['wasm-bindgen', '--version']),
}
checks = {
    'bun': versions['bun'] == '1.4.2', 'maturin': versions['maturin'] == 'maturin 1.9.6',
    'node': versions['node'].startswith('v24.'),
    'rustc': versions['rustc'].startswith('rustc 1.90.0 '), 'wasmBindgen': versions['wasmBindgen'] == 'wasm-bindgen 0.2.105',
    'cargo': versions['cargo'].startswith('cargo 1.90.0 '),
}
if not all(checks.values()):
    raise SystemExit('selected release tool version assertion failed: ' + json.dumps(versions, sort_keys=True))
root = pathlib.Path(os.environ['GITHUB_WORKSPACE'])
preflight_path = pathlib.Path(os.environ['EVIDENCE_ROOT'], 'runner-preflight.json')
preflight = json.loads(preflight_path.read_text())
features = tomllib.loads((root / 'crates/weft-python/pyproject.toml').read_text())['tool']['maturin']['features']
preflight.update(versions)
preflight['maturinFeatures'] = features
preflight['maturinFeaturesSha256'] = hashlib.sha256(json.dumps(features, separators=(',', ':')).encode()).hexdigest()
preflight_path.write_text(json.dumps(preflight, indent=2) + '\n')
rustc = command(['rustup', 'which', '--toolchain', '1.90.0', 'rustc'])
cargo = command(['rustup', 'which', '--toolchain', '1.90.0', 'cargo'])
binaries = {
    'python3': binary_hash(sys.executable), 'bun': binary_hash(shutil.which('bun')),
    'node': binary_hash(shutil.which('node')), 'npm': binary_hash(shutil.which('npm')),
    'maturin': binary_hash(shutil.which('maturin')), 'rustc': binary_hash(rustc),
    'cargo': binary_hash(cargo), 'wasm-bindgen': binary_hash(shutil.which('wasm-bindgen')),
}
pathlib.Path(os.environ['EVIDENCE_ROOT'], 'rustc-version.txt').write_text(command(['rustc', '+1.90.0', '-vV']) + '\n')
pathlib.Path(os.environ['EVIDENCE_ROOT'], 'tool-binary-sha256.txt').write_text(''.join(f"{value['sha256']}  {name}  {value['path']}\n" for name, value in binaries.items()))
