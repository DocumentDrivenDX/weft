#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Finalize a version-parameterized hosted ARM64 producer receipt."""
import argparse
import hashlib
import json
import os
import pathlib
import shutil
import zipfile

CASE_IDS = [
    'original-sales-join-truss.postgresql',
    'original-empty-global-truss.postgresql',
    'original-empty-grouped-truss.postgresql',
    'original-sales-join-ashlar.databricks',
    'original-empty-global-ashlar.databricks',
    'original-empty-grouped-ashlar.databricks',
    'original-string-exact-0',
    'original-string-exact-1',
    'original-string-exact-2',
]


def validate_refusal_rows(cases, rows):
    """Check the selected wheel-feature profile, not the corpus's default registry.

    These older Databricks requests select an unavailable backend version in the
    wheel composition; the generic corpus instead expects an absent backend.
    Retain the source fixtures and assert the qualified composition explicitly.
    """
    if [case.get('id') for case in cases] != CASE_IDS or [row.get('id') for row in rows] != CASE_IDS:
        raise ValueError('Rust comparator response roster is incomplete')
    for case, row in zip(cases, rows, strict=True):
        response = json.loads(row['raw'])
        code = 'WFT-BACKEND-VERSION' if case['id'].endswith('ashlar.databricks') else 'WFT-BACKEND-MISSING'
        codes = [diagnostic.get('code') for diagnostic in response.get('diagnostics', [])]
        if response.get('status') != 'blocked' or codes != [code] or any(key in response for key in ('sql', 'parameters', 'logicalPlan')):
            raise ValueError('Rust comparator did not preserve selected refusal: ' + case['id'])


def sha(path: pathlib.Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--version', required=True)
    parser.add_argument('--distribution', required=True)
    args = parser.parse_args()
    out = pathlib.Path(os.environ['DIST_ROOT'])
    root = pathlib.Path(os.environ['EVIDENCE_ROOT'])
    upload = pathlib.Path(os.environ['UPLOAD_ROOT'])
    root.mkdir(parents=True, exist_ok=True)
    upload.mkdir(parents=True, exist_ok=True)
    checks = {key: os.environ.get(key + '_OUTCOME', 'skipped') for key in ['PREFLIGHT', 'WHEEL', 'VECTORS', 'PARITY']}
    status = 'passed' if all(value == 'success' for value in checks.values()) and os.environ.get('JOB_OUTCOME') == 'success' else 'failed'
    payload = {
        'schema': 'weft-hosted-macos-arm64-producer/1', 'status': status,
        'version': args.version, 'distribution': args.distribution,
        'sourceCommit': os.environ.get('SOURCE_SHA', ''),
        'workflowCommit': os.environ.get('GITHUB_SHA', ''),
        'workflowRunId': os.environ.get('GITHUB_RUN_ID', ''),
        'runner': 'macos-14-arm64', 'target': 'aarch64-apple-darwin',
        'checks': checks,
        'compilerProfile': 'nine-frozen-refusal-responses-rust-python-chromium/1',
        'limits': [
            'Exact installed ARM64 wheel packaging and nine frozen refusal-response compiler vectors only.',
            'Vectors are refusal parity; no positive-query or native PostgreSQL execution is claimed.',
            'No publication is performed by this producer.',
        ],
    }
    for name, field in [('runner-preflight.json', 'runnerPreflightSha256'), ('tool-binary-sha256.txt', 'toolBinaryHashesSha256')]:
        path = root / name
        if path.is_file():
            payload[field] = sha(path)
    artifacts = []
    withheld = []
    for path in sorted(out.rglob('*')) if out.exists() else []:
        if not path.is_file():
            continue
        relative = path.relative_to(out).as_posix()
        size = path.stat().st_size
        limit = 60 * 1024 * 1024 if path.suffix == '.whl' else (20 * 1024 * 1024 if relative == 'web/weft_wasm_bg.wasm' else 5 * 1024 * 1024 if relative == 'web/weft_wasm.js' else 2 * 1024 * 1024)
        if size > limit:
            withheld.append({'name': relative, 'originalBytes': size, 'limitBytes': limit, 'reason': 'per-file producer artifact cap exceeded'})
        else:
            artifacts.append({'name': relative, 'bytes': size, 'sha256': sha(path)})
    total = sum(item['bytes'] for item in artifacts)
    if total > 70 * 1024 * 1024:
        withheld.extend({'name': item['name'], 'originalBytes': item['bytes'], 'limitBytes': 70 * 1024 * 1024, 'reason': 'aggregate producer artifact cap exceeded'} for item in artifacts)
        artifacts = []
    required = [
        out / 'compiler-roster.json', out / 'compiler-cases.json', out / 'compiler-reports.json',
        out / 'python-compiler-parity.json', out / 'browser-compiler-parity.json',
        out / 'browser-compiler-receipts.json', out / 'packaging-macos-arm64.json',
    ]
    try:
        if not all(path.is_file() for path in required):
            raise ValueError('compiler or packaging evidence is missing')
        roster = json.loads(required[0].read_text())
        cases = json.loads(required[1].read_text())
        rust_rows = json.loads(required[2].read_text())
        python_result = json.loads(required[3].read_text())
        browser = json.loads(required[4].read_text())
        packaging = json.loads(required[6].read_text())
        ids = [case.get('id') for case in cases]
        payload['compilerRoster'] = ids
        if ids != CASE_IDS or roster.get('cases') != CASE_IDS or roster.get('profile') != 'nine-frozen-refusal-responses/1':
            raise ValueError('compiler vector roster differs from the frozen refusal profile')
        for case in cases:
            expected = case.get('expected', {})
            if expected.get('status') != 'blocked' or expected.get('code') not in {'WFT-BACKEND-MISSING', 'WFT-BACKEND-VERSION'}:
                raise ValueError('compiler vector is outside the refusal-only profile')
        validate_refusal_rows(cases, rust_rows)
        if python_result.get('status') != 'passed' or [row.get('id') for row in python_result.get('cases', [])] != CASE_IDS:
            raise ValueError('installed Python wheel parity did not pass')
        if python_result.get('distribution') != args.distribution or python_result.get('distributionVersion') != args.version:
            raise ValueError('installed Python wheel identity differs')
        if browser.get('byteParity') is not True or browser.get('cases') != 9:
            raise ValueError('Chromium WASM parity did not pass')
        if packaging.get('status') != 'passed' or packaging.get('version') != args.version or packaging.get('distribution') != args.distribution:
            raise ValueError('ARM64 packaging report identity differs')
        wheels = list(out.glob('*.whl'))
        if len(wheels) != 1 or packaging.get('wheel') != wheels[0].name or packaging.get('wheelSha256') != sha(wheels[0]):
            raise ValueError('ARM64 wheel and packaging receipt do not match')
        with zipfile.ZipFile(wheels[0]) as archive:
            native = [name for name in archive.namelist() if name.endswith(('.so', '.dylib', '.pyd'))]
            if len(native) != 1:
                raise ValueError('ARM64 wheel native module inventory is not unique')
            module_sha = hashlib.sha256(archive.read(native[0])).hexdigest()
        if packaging.get('nativeModule') != native[0] or packaging.get('nativeModuleSha256') != module_sha or python_result.get('nativeModuleSha256') != module_sha:
            raise ValueError('ARM64 native module hashes differ across evidence')
        payload['wheel'] = {'name': wheels[0].name, 'sha256': sha(wheels[0]), 'nativeModule': native[0], 'nativeModuleSha256': module_sha}
        payload['parity'] = {'rust': 'passed', 'installedPython': 'passed', 'chromiumWasm': 'passed'}
        payload['functionalQualification'] = 'hosted-installed-wheel-refusal-parity-only'
    except (OSError, ValueError, KeyError, TypeError, zipfile.BadZipFile, json.JSONDecodeError) as error:
        status = 'failed'
        payload['status'] = 'failed'
        payload['failure'] = str(error)[:400]
        payload['parity'] = {'rust': 'failed', 'installedPython': 'failed', 'chromiumWasm': 'failed'}
    payload['artifacts'] = artifacts
    if withheld:
        payload['status'] = 'failed'
        payload['withheldEvidence'] = withheld
    if not artifacts:
        payload['status'] = 'failed'
    receipt = root / 'producer-receipt.json'
    receipt.write_text(json.dumps(payload, indent=2) + '\n')
    if out.exists():
        for path in sorted(out.rglob('*')):
            if path.is_file() and path.stat().st_size <= (60 * 1024 * 1024 if path.suffix == '.whl' else 20 * 1024 * 1024):
                destination = upload / path.relative_to(out)
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(path, destination)
    for path in sorted(root.iterdir()):
        if path.is_file() and path.stat().st_size <= 2 * 1024 * 1024:
            shutil.copyfile(path, upload / path.name)
    if sum(path.stat().st_size for path in upload.rglob('*') if path.is_file()) > 70 * 1024 * 1024:
        payload['status'] = 'failed'
        payload['artifacts'] = []
        payload['failure'] = 'aggregate artifact upload cap exceeded'
        # Retain only the bounded producer receipt. Do not upload an artifact
        # that violates the declared aggregate limit, even as a failed run.
        for path in sorted(upload.rglob('*'), reverse=True):
            if path.is_file() and path != upload / receipt.name:
                path.unlink()
            elif path.is_dir():
                try:
                    path.rmdir()
                except OSError:
                    pass
        receipt.write_text(json.dumps(payload, indent=2) + '\n')
        shutil.copyfile(receipt, upload / receipt.name)
    if payload['status'] != 'passed':
        raise SystemExit('hosted macOS producer did not pass every required check')


if __name__ == '__main__':
    main()
