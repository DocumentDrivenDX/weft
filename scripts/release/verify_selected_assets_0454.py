#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Verify fresh Weft 0.4.54 release artifacts before draft reconciliation."""
import argparse
import hashlib
import json
import pathlib
import re
import sys
import tomllib
import zipfile

VERSION = '0.4.54'
MAC_CASES = [
    'original-sales-join-truss.postgresql', 'original-empty-global-truss.postgresql',
    'original-empty-grouped-truss.postgresql', 'original-sales-join-ashlar.databricks',
    'original-empty-global-ashlar.databricks', 'original-empty-grouped-ashlar.databricks',
    'original-string-exact-0', 'original-string-exact-1', 'original-string-exact-2',
]
RELEASE_WORKFLOW = '.github/workflows/release.yml'
MAC_WORKFLOW = '.github/workflows/release-selected-macos-0.4.54.yml'
RELEASE_EXCLUSION = "github.ref_name != 'v0.4.54' && inputs.tag != 'v0.4.54'"


def require(condition, message):
    if not condition:
        raise ValueError(message)


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha(path):
    return sha_bytes(path.read_bytes())


def read_json(path):
    return json.loads(path.read_text())


def validate_run(run, *, repository, workflow, source, run_id=None, conclusion='success'):
    require(run.get('repository', {}).get('full_name') == repository, 'artifact run repository mismatch')
    require(run.get('path') == workflow, 'artifact run workflow path mismatch')
    require(run.get('conclusion') == conclusion and run.get('status') == 'completed', 'artifact run did not complete successfully')
    require(run.get('head_sha') == source, 'artifact run source SHA mismatch')
    require(run.get('event') == 'workflow_dispatch', 'artifact run was not a manual artifact-only dispatch')
    require(run.get('head_branch') == 'main', 'artifact run was not dispatched from main')
    if run_id is not None:
        require(str(run.get('id')) == str(run_id), 'artifact run ID mismatch')


def validate_release_workflow(source_text, metadata_text):
    require(source_text.count(RELEASE_EXCLUSION) == 2, 'frozen S release workflow must contain both exact v0.4.54 exclusions')
    require(metadata_text.count(RELEASE_EXCLUSION) == 2, 'metadata release workflow must retain both exact v0.4.54 exclusions')
    require(metadata_text == source_text, 'legacy release workflow changed after frozen source S')


def validate_linux_jobs(jobs):
    values = jobs.get('jobs', [])
    require(len(values) == 3, 'Linux artifact run must contain two wheel jobs and the skipped publisher job')
    wheel_jobs = [j for j in values if 'x86_64-unknown-linux-gnu' in j.get('name', '') or 'aarch64-unknown-linux-gnu' in j.get('name', '')]
    publisher = [j for j in values if j.get('name') == 'publish']
    require(len(wheel_jobs) == 2 and len(publisher) == 1, 'Linux artifact run has an unexpected job shape')
    names = [j.get('name', '') for j in wheel_jobs]
    require(sum('x86_64-unknown-linux-gnu' in n for n in names) == 1 and sum('aarch64-unknown-linux-gnu' in n for n in names) == 1, 'Linux run must complete each required wheel architecture exactly once')
    require(all(j.get('status') == 'completed' and j.get('conclusion') == 'success' for j in wheel_jobs), 'both Linux wheel jobs must succeed')
    require(publisher[0].get('status') == 'completed' and publisher[0].get('conclusion') == 'skipped', 'Linux publisher job must be skipped for artifact-only dispatch')


def validate_mac(receipt, preflight, run, *, source, run_id):
    require(preflight.get('status') == 'passed', 'macOS runner preflight did not pass')
    require(receipt.get('schema') == 'weft-selected-macos-host/1', 'wrong macOS receipt schema')
    require(receipt.get('status') == 'passed', 'macOS hosted checks did not all pass')
    require(receipt.get('version') == VERSION and receipt.get('sourceCommit') == source, 'macOS receipt version/source mismatch')
    require(receipt.get('workflowCommit') == source and run.get('head_sha') == source, 'macOS workflow head must equal frozen source S')
    require(preflight.get('sourceCommit') == source and preflight.get('workflowCommit') == source, 'macOS preflight is not bound to frozen source S')
    require(str(receipt.get('workflowRunId')) == str(run_id) == str(run.get('id')), 'macOS receipt/run ID mismatch')
    require(receipt.get('functionalQualification') == 'hosted-installed-wheel-compiler-parity-only', 'macOS hosted qualification claim is absent or unexpected')
    checks = receipt.get('checks')
    require(type(checks) is dict and set(checks) == {'PREFLIGHT', 'WHEEL', 'VECTORS', 'PARITY'} and all(v == 'success' for v in checks.values()), 'macOS producer is missing required checks')
    wheel = receipt.get('wheel')
    require(type(wheel) is dict and wheel.get('name', '').startswith('weft_sql-0.4.54-') and wheel['name'].endswith('-macosx_11_0_arm64.whl'), 'macOS wheel identity mismatch')
    require(re.fullmatch(r'[0-9a-f]{64}', wheel.get('sha256', '')) is not None and re.fullmatch(r'[0-9a-f]{64}', wheel.get('nativeModuleSha256', '')) is not None, 'macOS wheel/module hash missing')
    require(receipt.get('compilerRoster') == MAC_CASES, 'macOS compiler roster differs from the frozen nine-vector selection')
    parity = receipt.get('parity', {})
    require(parity.get('rust') == 'passed' and parity.get('python') == 'passed' and parity.get('chromiumWasm') == 'passed', 'required installed-wheel/compiler parity receipt missing')
    require(receipt.get('runnerPreflightSha256') == sha_bytes(json.dumps(preflight, indent=2).encode() + b'\n'), 'macOS runner preflight evidence digest mismatch')
    runner = preflight.get('runner', {})
    require(runner.get('label') == 'macos-14' and runner.get('architectureSource') == 'https://docs.github.com/en/actions/reference/runners/github-hosted-runners', 'macOS runner label/architecture source is absent')
    require(runner.get('os') == 'Darwin' and runner.get('architecture') == 'arm64' and runner.get('target') == 'aarch64-apple-darwin', 'macOS producer did not run on the selected ARM64 target')
    require(preflight.get('python', '').startswith('Python 3.12.'), 'macOS Python version is outside the selected 3.12 line')
    require(preflight.get('bun') == '1.4.2' and preflight.get('node', '').startswith('v24.') and preflight.get('maturin') == 'maturin 1.9.6' and preflight.get('rustc', '').startswith('rustc 1.90.0 ') and preflight.get('cargo', '').startswith('cargo 1.90.0 ') and preflight.get('wasmBindgen') == 'wasm-bindgen 0.2.105', 'macOS tool versions differ from the frozen release profile')
    features = preflight.get('maturinFeatures')
    require(type(features) is list and features and preflight.get('maturinFeaturesSha256') == sha_bytes(json.dumps(features, separators=(',', ':')).encode()), 'macOS Maturin feature set is missing or unbound')
    return features


def validate_module_binding(receipt, python_report, actual_module_sha256):
    expected = receipt.get('wheel', {}).get('nativeModuleSha256')
    require(expected == actual_module_sha256 == python_report.get('nativeModuleSha256'), 'installed Python module digest differs from the accepted wheel module')


def classify_wheels(wheels):
    require(len(wheels) == 3, 'release must contain exactly three wheels')
    targets = {}
    for path in wheels:
        name = path.name
        require(name.startswith('weft_sql-0.4.54-'), f'wrong wheel version: {name}')
        if 'manylinux_2_17_x86_64.manylinux2014_x86_64' in name:
            target = 'x86_64-unknown-linux-gnu'
        elif 'manylinux_2_17_aarch64.manylinux2014_aarch64' in name:
            target = 'aarch64-unknown-linux-gnu'
        elif name.endswith('-macosx_11_0_arm64.whl'):
            target = 'aarch64-apple-darwin'
        else:
            raise ValueError(f'unexpected platform wheel: {name}')
        require(target not in targets, f'duplicate wheel target: {target}')
        targets[target] = path
        with zipfile.ZipFile(path) as archive:
            names = archive.namelist()
            require(len(names) == len(set(names)), f'duplicate archive path: {name}')
            native = [n for n in names if n.endswith(('.so', '.dylib', '.pyd'))]
            require(len(native) == 1, f'wheel must contain exactly one native module: {name}')
            metadata = archive.read(next(n for n in names if n.endswith('.dist-info/METADATA'))).decode()
            wheelmeta = archive.read(next(n for n in names if n.endswith('.dist-info/WHEEL'))).decode()
            require('Name: weft-sql\nVersion: 0.4.54\n' in metadata, f'wheel metadata mismatch: {name}')
            require('Tag: cp39-abi3-' in wheelmeta, f'not a CPython abi3 wheel: {name}')
    require(set(targets) == {'x86_64-unknown-linux-gnu', 'aarch64-unknown-linux-gnu', 'aarch64-apple-darwin'}, 'wheel target set mismatch')
    return targets


def mac_wheel_module(path):
    with zipfile.ZipFile(path) as archive:
        native = [n for n in archive.namelist() if n.endswith(('.so', '.dylib', '.pyd'))]
        require(len(native) == 1, 'macOS wheel must contain exactly one native module')
        return native[0], sha_bytes(archive.read(native[0]))


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--source-sha', required=True); p.add_argument('--metadata-sha', required=True)
    p.add_argument('--linux-run-id', required=True); p.add_argument('--macos-run-id', required=True)
    p.add_argument('--linux-run', type=pathlib.Path, required=True); p.add_argument('--macos-run', type=pathlib.Path, required=True)
    p.add_argument('--linux-jobs', type=pathlib.Path, required=True); p.add_argument('--source-pyproject', type=pathlib.Path, required=True)
    p.add_argument('--source-release-workflow', type=pathlib.Path, required=True); p.add_argument('--metadata-release-workflow', type=pathlib.Path, required=True)
    p.add_argument('--linux-dir', type=pathlib.Path, required=True); p.add_argument('--macos-dir', type=pathlib.Path, required=True)
    p.add_argument('--macos-receipt', type=pathlib.Path, required=True); p.add_argument('--output', type=pathlib.Path, required=True)
    a = p.parse_args()
    require(re.fullmatch(r'[0-9a-f]{40}', a.source_sha) is not None and re.fullmatch(r'[0-9a-f]{40}', a.metadata_sha) is not None, 'invalid source or metadata SHA')
    validate_release_workflow(a.source_release_workflow.read_text(), a.metadata_release_workflow.read_text())
    repository = 'DocumentDrivenDX/weft'
    linux_run, macos_run = read_json(a.linux_run), read_json(a.macos_run)
    validate_run(linux_run, repository=repository, workflow=RELEASE_WORKFLOW, source=a.source_sha, run_id=a.linux_run_id)
    validate_linux_jobs(read_json(a.linux_jobs))
    validate_run(macos_run, repository=repository, workflow=MAC_WORKFLOW, source=a.source_sha, run_id=a.macos_run_id)
    mac_receipt = read_json(a.macos_receipt)
    macdir = a.macos_dir
    preflight = read_json(macdir / 'runner-preflight.json')
    features = validate_mac(mac_receipt, preflight, macos_run, source=a.source_sha, run_id=a.macos_run_id)
    tool_hash_path = macdir / 'tool-binary-sha256.txt'
    require(tool_hash_path.is_file() and mac_receipt.get('toolBinaryHashesSha256') == sha(tool_hash_path), 'macOS tool binary digest record is missing or mismatched')
    source_features = tomllib.loads(a.source_pyproject.read_text())['tool']['maturin']['features']
    require(features == source_features, 'hosted macOS Maturin feature list differs from frozen S pyproject')
    rust_path = macdir / 'compiler-reports.json'
    roster = read_json(macdir / 'compiler-roster.json'); py = read_json(macdir / 'python-compiler-parity.json')
    browser = read_json(macdir / 'browser-compiler-parity.json'); browser_receipts = read_json(macdir / 'browser-compiler-receipts.json')
    rust = read_json(rust_path)
    require(roster.get('cases') == MAC_CASES and py.get('status') == 'passed', 'selected compiler roster or installed-wheel result is incomplete')
    require(browser.get('cases') == 9 and browser.get('byteParity') is True and browser.get('playwrightVersion') == '1.62.1', 'actual Chromium WASM parity did not pass')
    require(isinstance(browser.get('browser'), str) and browser['browser'], 'actual Chromium version was not recorded')
    require([row.get('id') for row in rust] == MAC_CASES and [row.get('id') for row in py.get('cases', [])] == MAC_CASES, 'Rust/Python compiler receipts differ from the frozen roster')
    browser_cases = browser_receipts.get('cases', [])
    require([row.get('id') for row in browser_cases] == MAC_CASES, 'browser case receipt order differs from the frozen roster')
    for rust_row, py_row, browser_row in zip(rust, py['cases'], browser_cases, strict=True):
        raw_sha = sha_bytes(rust_row['raw'].encode())
        require(py_row.get('rustResponseSha256') == raw_sha == py_row.get('pythonResponseSha256') == browser_row.get('actualSha256') == browser_row.get('expectedSha256'), f'compiler parity digest mismatch: {rust_row["id"]}')
    records = {row.get('name'): row for row in mac_receipt.get('artifacts', [])}
    required_files = ['compiler-roster.json', 'compiler-reports.json', 'python-compiler-parity.json', 'browser-compiler-parity.json', 'browser-compiler-receipts.json', 'packaging-macos-arm64.json', 'weft_wasm_bg.wasm', 'weft_wasm.js']
    for name in required_files:
        path = macdir / name; row = records.get(name)
        require(path.is_file() and row is not None and row.get('bytes') == path.stat().st_size and row.get('sha256') == sha(path), f'macOS artifact bytes differ from hosted producer receipt: {name}')
    wheels = list(a.linux_dir.glob('*.whl')) + list(macdir.glob('*.whl'))
    targets = classify_wheels(wheels)
    require(len(list(a.linux_dir.glob('*.whl'))) == 2 and len(list(macdir.glob('*.whl'))) == 1, 'wheel run placement mismatch')
    macwheel = targets['aarch64-apple-darwin']
    require(mac_receipt['wheel']['name'] == macwheel.name and mac_receipt['wheel']['sha256'] == sha(macwheel), 'macOS producer receipt wheel identity/digest mismatch')
    native_name, native_digest = mac_wheel_module(macwheel)
    validate_module_binding(mac_receipt, py, native_digest)
    reports = list(a.linux_dir.glob('packaging-*.json')) + list(macdir.glob('packaging-macos-arm64.json'))
    require(len(reports) == 3, 'one packaging report per target is required')
    by_name = {w.name: w for w in wheels}
    for report_path in reports:
        report = read_json(report_path); wheel = by_name.get(report.get('wheel'))
        require(wheel is not None and report.get('status') == 'passed' and report.get('version') == VERSION, f'packaging report does not bind one passing wheel: {report_path.name}')
        require(report.get('wheelSha256') == sha(wheel), f'wheel hash differs from packaging report: {wheel.name}')
        if report_path.name == 'packaging-macos-arm64.json':
            require(report.get('sourceCommit') == a.source_sha and report.get('target') == 'aarch64-apple-darwin', 'macOS packaging report target/source mismatch')
            require(report.get('nativeModuleSha256') == native_digest, 'macOS module digest differs across receipt, package and wheel')
        else:
            require(report.get('sourceCommit') == a.source_sha, f'Linux packaging source mismatch: {report_path.name}')
        with zipfile.ZipFile(wheel) as archive:
            native = [n for n in archive.namelist() if n.endswith(('.so', '.dylib', '.pyd'))]
            require(report.get('nativeModuleSha256') == sha_bytes(archive.read(native[0])), f'native module hash mismatch: {wheel.name}')
    output = {
        'schema': 'weft-selected-assets-verification/1', 'status': 'passed', 'version': VERSION,
        'sourceCommit': a.source_sha, 'releaseMetadataCommit': a.metadata_sha,
        'linuxRun': {'id': linux_run['id'], 'headSha': linux_run['head_sha'], 'workflow': linux_run['path']},
        'macosRun': {'id': macos_run['id'], 'headSha': macos_run['head_sha'], 'workflow': macos_run['path']},
        'wheels': [{'target': target, 'name': path.name, 'sha256': sha(path)} for target, path in sorted(targets.items())],
        'macosMaturinFeatures': features, 'macosCompilerReportSha256': sha(rust_path),
        'macosToolBinaryHashesSha256': sha(tool_hash_path),
        'macosBrowserVersion': browser['browser'],
        'macosNativeModule': {'name': native_name, 'sha256': native_digest},
        'selectedMacCompilerParity': {'cases': MAC_CASES, 'status': 'passed'}, 'nativeDatabaseExecution': 'not-qualified'
    }
    a.output.parent.mkdir(parents=True, exist_ok=True); a.output.write_text(json.dumps(output, indent=2) + '\n')
    print(json.dumps(output))


if __name__ == '__main__':
    try:
        main()
    except Exception as exc:
        print(f'0.4.54 asset verification failed: {exc}', file=sys.stderr); sys.exit(1)
