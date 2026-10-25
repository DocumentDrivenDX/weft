#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Assemble bounded macOS producer evidence and preserve the fail-closed receipt."""
import hashlib
import json
import os
import pathlib
import shutil
import zipfile


def main():
    out = pathlib.Path(os.environ['DIST_ROOT']); root = pathlib.Path(os.environ['EVIDENCE_ROOT']); root.mkdir(parents=True, exist_ok=True)
    upload = pathlib.Path(os.environ['UPLOAD_ROOT']); upload.mkdir(parents=True, exist_ok=True)
    checks = {key: os.environ.get(key + '_OUTCOME', 'skipped') for key in ['PREFLIGHT', 'WHEEL', 'VECTORS', 'PARITY']}
    status = 'passed' if all(value == 'success' for value in checks.values()) and os.environ['JOB_OUTCOME'] == 'success' else 'failed'
    payload = {
        'schema': 'weft-selected-macos-host/1', 'status': status, 'version': '0.4.54',
        'sourceCommit': os.environ['SOURCE_SHA'], 'workflowCommit': os.environ['GITHUB_SHA'],
        'workflowRunId': os.environ['GITHUB_RUN_ID'], 'checks': checks,
        'limits': ['Exact installed wheel packaging and nine selected compiler vectors only. No native PostgreSQL execution or publication.']
    }
    preflight = root / 'runner-preflight.json'
    if preflight.is_file():
        payload['runnerPreflightSha256'] = hashlib.sha256(preflight.read_bytes()).hexdigest()
    tool_hashes = root / 'tool-binary-sha256.txt'
    if tool_hashes.is_file():
        payload['toolBinaryHashesSha256'] = hashlib.sha256(tool_hashes.read_bytes()).hexdigest()
    artifacts = []; withheld = []; total = 0
    for path in sorted(out.iterdir()) if out.exists() else []:
        if path.is_file():
            size = path.stat().st_size; limit = 60 * 1024 * 1024 if path.suffix == '.whl' else 2 * 1024 * 1024
            if size > limit:
                withheld.append({'name': path.name, 'originalBytes': size, 'limitBytes': limit, 'reason': 'per-file evidence cap exceeded'})
            else:
                artifacts.append({'name': path.name, 'bytes': size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}); total += size
    for path, limit in [(out / 'web/weft_wasm_bg.wasm', 20 * 1024 * 1024), (out / 'web/weft_wasm.js', 5 * 1024 * 1024)]:
        if path.is_file():
            size = path.stat().st_size
            if size > limit:
                withheld.append({'name': path.name, 'originalBytes': size, 'limitBytes': limit, 'reason': 'per-file browser artifact cap exceeded'})
            else:
                artifacts.append({'name': path.name, 'bytes': size, 'sha256': hashlib.sha256(path.read_bytes()).hexdigest()}); total += size
    if artifacts:
        payload['artifacts'] = artifacts
    if total > 70 * 1024 * 1024:
        withheld.extend({'name': item['name'], 'originalBytes': item['bytes'], 'limitBytes': 70 * 1024 * 1024, 'reason': 'aggregate upload evidence cap exceeded'} for item in artifacts)
        payload['artifacts'] = []
    roster = out / 'compiler-roster.json'; python_parity = out / 'python-compiler-parity.json'; browser = out / 'browser-compiler-parity.json'; reports = out / 'compiler-reports.json'
    if all(path.is_file() for path in (roster, python_parity, browser, reports)):
        roster_data = json.loads(roster.read_text()); py_data = json.loads(python_parity.read_text()); browser_data = json.loads(browser.read_text()); rust_data = json.loads(reports.read_text())
        ids = roster_data.get('cases', []); payload['compilerRoster'] = ids
        payload['parity'] = {
            'rust': 'passed' if len(rust_data) == 9 and [row.get('id') for row in rust_data] == ids else 'failed',
            'python': 'passed' if py_data.get('status') == 'passed' and [row.get('id') for row in py_data.get('cases', [])] == ids else 'failed',
            'chromiumWasm': 'passed' if browser_data.get('byteParity') is True and browser_data.get('cases') == 9 else 'failed'
        }
    else:
        payload['parity'] = {'rust': 'failed', 'python': 'failed', 'chromiumWasm': 'failed'}
    wheels = list(out.glob('*.whl')) if out.exists() else []
    if len(wheels) == 1:
        with zipfile.ZipFile(wheels[0]) as archive:
            native = [name for name in archive.namelist() if name.endswith(('.so', '.dylib', '.pyd'))]
            if len(native) == 1:
                payload['wheel'] = {'name': wheels[0].name, 'sha256': hashlib.sha256(wheels[0].read_bytes()).hexdigest(), 'nativeModule': native[0], 'nativeModuleSha256': hashlib.sha256(archive.read(native[0])).hexdigest()}
    packaging = out / 'packaging-macos-arm64.json'
    if packaging.is_file() and json.loads(packaging.read_text()).get('status') == 'passed':
        payload['functionalQualification'] = 'hosted-installed-wheel-compiler-parity-only'
    if status == 'passed' and (any(value != 'passed' for value in payload['parity'].values()) or 'wheel' not in payload or payload.get('functionalQualification') is None):
        payload['status'] = 'failed'; status = 'failed'
    if withheld:
        payload['status'] = 'failed'; status = 'failed'; payload['withheldEvidence'] = withheld
    receipt = root / 'producer-receipt.json'
    receipt.write_text(json.dumps(payload, indent=2) + '\n')
    if out.exists():
        for path in sorted(out.iterdir()):
            if path.is_file():
                limit = 60 * 1024 * 1024 if path.suffix == '.whl' else 2 * 1024 * 1024
                if path.stat().st_size <= limit:
                    shutil.copyfile(path, upload / path.name)
    for path, limit in [(out / 'web/weft_wasm_bg.wasm', 20 * 1024 * 1024), (out / 'web/weft_wasm.js', 5 * 1024 * 1024)]:
        if path.is_file() and path.stat().st_size <= limit:
            shutil.copyfile(path, upload / path.name)
    for path in sorted(root.iterdir()):
        if path.is_file() and path.stat().st_size <= 2 * 1024 * 1024:
            shutil.copyfile(path, upload / path.name)
    upload_files = [path for path in upload.iterdir() if path.is_file()]
    upload_bytes = sum(path.stat().st_size for path in upload_files)
    if upload_bytes > 70 * 1024 * 1024:
        originals = [{'name': path.name, 'originalBytes': path.stat().st_size, 'limitBytes': 70 * 1024 * 1024, 'reason': 'aggregate upload evidence cap exceeded'} for path in upload_files]
        for path in upload_files:
            if path.name != 'producer-receipt.json':
                path.unlink()
        payload['status'] = 'failed'; status = 'failed'; payload['artifacts'] = []; payload['withheldEvidence'] = originals
        receipt.write_text(json.dumps(payload, indent=2) + '\n')
        shutil.copyfile(receipt, upload / 'producer-receipt.json')
    if status != 'passed':
        raise SystemExit('hosted macOS producer did not pass every required check')


if __name__ == '__main__':
    main()
