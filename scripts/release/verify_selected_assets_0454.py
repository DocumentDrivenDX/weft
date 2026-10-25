#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Verify fresh Weft 0.4.54 release artifacts before draft reconciliation."""
import argparse
import hashlib
import json
import pathlib
import re
import sys
import zipfile

VERSION = '0.4.54'
MAC_CASES = [
    'original-sales-join-truss.postgresql', 'original-empty-global-truss.postgresql',
    'original-empty-grouped-truss.postgresql', 'original-sales-join-ashlar.databricks',
    'original-empty-global-ashlar.databricks', 'original-empty-grouped-ashlar.databricks',
    'original-string-exact-0', 'original-string-exact-1', 'original-string-exact-2',
]

def require(condition, message):
    if not condition:
        raise ValueError(message)

def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def read_json(path):
    return json.loads(path.read_text())

def validate_run(run, *, repository, workflow, source, conclusion='success'):
    require(run.get('repository', {}).get('full_name') == repository, 'artifact run repository mismatch')
    require(run.get('path') == workflow, 'artifact run workflow path mismatch')
    require(run.get('conclusion') == conclusion and run.get('status') == 'completed', 'artifact run did not complete successfully')
    require(run.get('head_sha') == source, 'artifact run source SHA mismatch')
    require(run.get('event') == 'workflow_dispatch', 'artifact run was not a manual artifact-only dispatch')
    require(run.get('head_branch') == 'main', 'artifact run was not dispatched from main')

def validate_mac(receipt, *, source):
    require(receipt.get('schema') == 'weft-selected-macos-host/1', 'wrong macOS receipt schema')
    require(receipt.get('status') == 'passed', 'macOS hosted checks did not all pass')
    require(receipt.get('version') == VERSION and receipt.get('sourceCommit') == source, 'macOS receipt version/source mismatch')
    require(re.fullmatch(r'[0-9a-f]{40}', receipt.get('workflowCommit', '')) is not None, 'macOS receipt workflow commit is missing')
    require(receipt.get('functionalQualification') != 'local-only', 'legacy local-only receipt is not eligible')
    checks = receipt.get('checks')
    require(type(checks) is dict and set(checks) == {'PREFLIGHT', 'WHEEL', 'VECTORS', 'PARITY'} and all(v == 'success' for v in checks.values()), 'macOS producer is missing required checks')
    wheel = receipt.get('wheel')
    require(type(wheel) is dict and wheel.get('name', '').startswith('weft_sql-0.4.54-') and wheel['name'].endswith('-macosx_11_0_arm64.whl'), 'macOS wheel identity mismatch')
    require(re.fullmatch(r'[0-9a-f]{64}', wheel.get('sha256', '')) is not None and re.fullmatch(r'[0-9a-f]{64}', wheel.get('nativeModuleSha256', '')) is not None, 'macOS wheel/module hash missing')
    roster = receipt.get('compilerRoster')
    require(roster == MAC_CASES, 'macOS compiler roster differs from the frozen nine-vector selection')
    require(receipt.get('parity', {}).get('rust') == 'passed' and receipt.get('parity', {}).get('python') == 'passed' and receipt.get('parity', {}).get('chromiumWasm') == 'passed', 'required installed-wheel/compiler parity receipt missing')

def validate_linux_jobs(jobs):
    values = jobs.get('jobs', [])
    require(len(values) == 2 and all(j.get('status') == 'completed' and j.get('conclusion') == 'success' for j in values), 'Linux artifact run must contain exactly two successful wheel jobs')
    names = [j.get('name', '') for j in values]
    require(sum('x86_64-unknown-linux-gnu' in n for n in names) == 1 and sum('aarch64-unknown-linux-gnu' in n for n in names) == 1, 'Linux run did not complete each required architecture exactly once')

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

def main():
    p = argparse.ArgumentParser()
    p.add_argument('--source-sha', required=True); p.add_argument('--metadata-sha', required=True)
    p.add_argument('--linux-run', type=pathlib.Path, required=True); p.add_argument('--macos-run', type=pathlib.Path, required=True)
    p.add_argument('--linux-jobs', type=pathlib.Path, required=True)
    p.add_argument('--linux-dir', type=pathlib.Path, required=True); p.add_argument('--macos-dir', type=pathlib.Path, required=True)
    p.add_argument('--macos-receipt', type=pathlib.Path, required=True); p.add_argument('--output', type=pathlib.Path, required=True)
    a = p.parse_args()
    require(re.fullmatch(r'[0-9a-f]{40}', a.source_sha) is not None and re.fullmatch(r'[0-9a-f]{40}', a.metadata_sha) is not None, 'invalid source or metadata SHA')
    repository = 'DocumentDrivenDX/weft'
    linux_run, macos_run = read_json(a.linux_run), read_json(a.macos_run)
    validate_run(linux_run, repository=repository, workflow='.github/workflows/release.yml', source=a.source_sha)
    validate_linux_jobs(read_json(a.linux_jobs))
    validate_run(macos_run, repository=repository, workflow='.github/workflows/release-selected-macos-0.4.54.yml', source=macos_run.get('head_sha'))
    mac_receipt = read_json(a.macos_receipt)
    validate_mac(mac_receipt, source=a.source_sha)
    require(macos_run['head_sha'] == mac_receipt.get('workflowCommit'), 'macOS workflow commit does not match producer run')
    macdir=a.macos_dir
    roster=read_json(macdir/'compiler-roster.json'); py=read_json(macdir/'python-compiler-parity.json')
    browser=read_json(macdir/'browser-compiler-parity.json'); browser_receipts=read_json(macdir/'browser-compiler-receipts.json')
    rust=json.loads((macdir/'compiler-reports.json').read_text())
    require(roster.get('cases') == MAC_CASES and py.get('status') == 'passed', 'selected compiler roster or installed-wheel result is incomplete')
    require(browser.get('cases') == 9 and browser.get('byteParity') is True and browser.get('playwrightVersion') == '1.62.1', 'actual Chromium WASM parity did not pass')
    require([row.get('id') for row in rust] == MAC_CASES and [row.get('id') for row in py.get('cases', [])] == MAC_CASES, 'Rust/Python compiler receipts differ from the frozen roster')
    browser_cases=browser_receipts.get('cases', [])
    require([row.get('id') for row in browser_cases] == MAC_CASES, 'browser case receipt order differs from the frozen roster')
    for rust_row, py_row, browser_row in zip(rust, py['cases'], browser_cases, strict=True):
        raw_sha=hashlib.sha256(rust_row['raw'].encode()).hexdigest()
        require(py_row.get('rustResponseSha256') == raw_sha == py_row.get('pythonResponseSha256') == browser_row.get('actualSha256') == browser_row.get('expectedSha256'), f'compiler parity digest mismatch: {rust_row["id"]}')
    records={row.get('name'):row for row in mac_receipt.get('artifacts', [])}
    required_files=['compiler-roster.json','compiler-reports.json','python-compiler-parity.json','browser-compiler-parity.json','browser-compiler-receipts.json','packaging-macos-arm64.json','weft_wasm_bg.wasm','weft_wasm.js']
    for name in required_files:
        path=macdir/name;row=records.get(name)
        require(path.is_file() and row is not None and row.get('bytes') == path.stat().st_size and row.get('sha256') == sha(path), f'macOS artifact bytes differ from hosted producer receipt: {name}')
    wheels = list(a.linux_dir.glob('*.whl')) + list(a.macos_dir.glob('*.whl'))
    targets = classify_wheels(wheels)
    require(len(list(a.linux_dir.glob('*.whl'))) == 2 and len(list(a.macos_dir.glob('*.whl'))) == 1, 'wheel run placement mismatch')
    require(mac_receipt['wheel']['name'] == targets['aarch64-apple-darwin'].name and mac_receipt['wheel']['sha256'] == sha(targets['aarch64-apple-darwin']), 'macOS producer receipt wheel identity/digest mismatch')
    reports = list(a.linux_dir.glob('packaging-*.json')) + list(a.macos_dir.glob('packaging-macos-arm64.json'))
    require(len(reports) == 3, 'one packaging report per target is required')
    by_name = {w.name: w for w in wheels}
    for report_path in reports:
        report = read_json(report_path); wheel = by_name.get(report.get('wheel'))
        require(wheel is not None and report.get('status') == 'passed' and report.get('version') == VERSION, f'packaging report does not bind one passing wheel: {report_path.name}')
        require(report.get('wheelSha256') == sha(wheel), f'wheel hash differs from packaging report: {wheel.name}')
        if report_path.name == 'packaging-macos-arm64.json':
            require(report.get('sourceCommit') == a.source_sha and report.get('target') == 'aarch64-apple-darwin', 'macOS packaging report target/source mismatch')
            require(report.get('nativeModuleSha256') == mac_receipt['wheel']['nativeModuleSha256'], 'macOS native module digest differs across receipt and packaging report')
        else:
            require(report.get('sourceCommit') == a.source_sha, f'Linux packaging source mismatch: {report_path.name}')
        with zipfile.ZipFile(wheel) as archive:
            native = [n for n in archive.namelist() if n.endswith(('.so', '.dylib', '.pyd'))]
            require(report.get('nativeModuleSha256') == hashlib.sha256(archive.read(native[0])).hexdigest(), f'native module hash mismatch: {wheel.name}')
    require(mac_receipt['wheel']['sha256'] == sha(targets['aarch64-apple-darwin']), 'macOS receipt wheel digest mismatch')
    output = {'schema':'weft-selected-assets-verification/1','status':'passed','version':VERSION,'sourceCommit':a.source_sha,'releaseMetadataCommit':a.metadata_sha,'linuxRun':{'id':linux_run['id'],'headSha':linux_run['head_sha'],'workflow':linux_run['path']},'macosRun':{'id':macos_run['id'],'headSha':macos_run['head_sha'],'workflow':macos_run['path']},'wheels':[{'target':target,'name':path.name,'sha256':sha(path)} for target,path in sorted(targets.items())],'selectedMacCompilerParity':{'cases':MAC_CASES,'status':'passed'},'nativeDatabaseExecution':'not-qualified'}
    a.output.parent.mkdir(parents=True, exist_ok=True);a.output.write_text(json.dumps(output, indent=2)+'\n')
    print(json.dumps(output))

if __name__ == '__main__':
    try: main()
    except Exception as exc: print(f'0.4.54 asset verification failed: {exc}', file=sys.stderr); sys.exit(1)
