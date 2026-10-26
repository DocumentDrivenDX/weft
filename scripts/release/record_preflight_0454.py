#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Record the selected hosted macOS runner before tool acquisition."""
import json
import os
import pathlib
import platform
import subprocess
import tomllib


root = pathlib.Path(os.environ['GITHUB_WORKSPACE'])
version = tomllib.loads((root / 'crates/weft-python/pyproject.toml').read_text())['project']['version']
crate = tomllib.loads((root / 'crates/weft-python/Cargo.toml').read_text())['package']['version']
if version != crate or version != os.environ['SOURCE_VERSION']:
    raise SystemExit('selected release version files do not agree')
if platform.system() != 'Darwin' or platform.machine() != 'arm64':
    raise SystemExit('selected macOS label did not provide a Darwin ARM64 runner')
payload = {
    'status': 'passed', 'sourceCommit': os.environ['SOURCE_SHA'],
    'workflowCommit': os.environ['GITHUB_SHA'], 'version': version,
    'runner': {
        'label': 'macos-14',
        'architectureSource': 'https://docs.github.com/en/actions/reference/runners/github-hosted-runners',
        'name': os.environ.get('ImageOS', 'unknown'), 'imageVersion': os.environ.get('ImageVersion', 'unknown'),
        'os': platform.system(), 'platform': platform.platform(), 'architecture': platform.machine(),
        'target': 'aarch64-apple-darwin',
    },
    'sdk': subprocess.check_output(['xcrun', '--sdk', 'macosx', '--show-sdk-version'], text=True).strip(),
    'python': subprocess.check_output(['python3', '--version'], text=True).strip(),
    'bun': subprocess.check_output(['bun', '--version'], text=True).strip(),
    'target': 'aarch64-apple-darwin', 'rustExpected': '1.90.0',
    'maturinExpected': '1.9.6', 'wasmBindgenExpected': '0.2.105',
    'compilerFlags': {'CARGO_PROFILE': 'release', 'featuresFrom': 'crates/weft-python/pyproject.toml [tool.maturin].features'},
}
pathlib.Path(os.environ['EVIDENCE_ROOT'], 'runner-preflight.json').write_text(json.dumps(payload, indent=2) + '\n')
