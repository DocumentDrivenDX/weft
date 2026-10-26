#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Validate and safely extract an exact hosted macOS producer artifact."""
import argparse
from email.parser import BytesParser
import hashlib
import io
import json
import os
import pathlib
import re
import stat
import urllib.error
import urllib.parse
import urllib.request
import zipfile
from finalize_hosted_macos import validate_refusal_rows

MAX_ARCHIVE = 70 * 1024 * 1024
MAX_FILES = 160
MAX_WHEEL = 60 * 1024 * 1024
MAC_WORKFLOW = '.github/workflows/release-hosted-macos.yml'
CASE_IDS = [
    'original-sales-join-truss.postgresql',
    'original-empty-global-truss.postgresql',
    'original-empty-grouped-truss.postgresql',
    'original-sales-join-ashlar.databricks',
    'original-empty-global-ashlar.databricks',
    'original-empty-grouped-ashlar.databricks',
    'original-string-exact-0', 'original-string-exact-1', 'original-string-exact-2',
]


class VerificationError(ValueError):
    pass


class _SafeRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        old = urllib.parse.urlparse(req.full_url)
        new = urllib.parse.urlparse(newurl)
        require(new.scheme == 'https', 'GitHub artifact download redirected away from HTTPS')
        result = super().redirect_request(req, fp, code, msg, headers, newurl)
        if result is not None and (new.hostname, new.port) != (old.hostname, old.port):
            result.remove_header('Authorization')
            result.remove_header('Proxy-Authorization')
        return result


def _github_request(url: str, token: str, accept: str, limit: int) -> bytes:
    request = urllib.request.Request(url, headers={
        'Authorization': f'Bearer {token}', 'Accept': accept,
        'X-GitHub-Api-Version': '2022-11-28', 'User-Agent': 'weft-release-verifier',
    })
    opener = urllib.request.build_opener(_SafeRedirect())
    try:
        with opener.open(request, timeout=60) as response:
            data = response.read(limit + 1)
    except (urllib.error.URLError, TimeoutError) as exc:
        raise VerificationError('GitHub API or artifact download failed') from exc
    require(len(data) <= limit, 'GitHub response exceeds its bounded download limit')
    return data


def require(ok: bool, message: str) -> None:
    if not ok:
        raise VerificationError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def wheel_prefix(distribution: str, version: str) -> str:
    normalized = re.sub(r'[-_.]+', '_', distribution)
    return f'{normalized}-{version}-'


def verify_archive(*, archive_path: pathlib.Path, run: dict, artifact_response: dict,
                   run_id: int, source_sha: str, version: str, distribution: str,
                   repository: str, output_dir: pathlib.Path) -> dict:
    require(re.fullmatch(r'[0-9a-f]{40}', source_sha) is not None, 'source SHA is malformed')
    require(str(run.get('id')) == str(run_id), 'run ID differs from the requested producer')
    require(run.get('repository', {}).get('full_name') == repository, 'producer repository differs')
    require(run.get('event') == 'workflow_dispatch' and run.get('head_branch') == 'main', 'producer must be a main-branch workflow_dispatch run')
    require(run.get('status') == 'completed' and run.get('conclusion') == 'success', 'producer run is not successful and complete')
    require(run.get('path') == MAC_WORKFLOW, 'producer workflow path differs')
    require(run.get('head_sha') == source_sha, 'producer source SHA differs')
    artifacts = artifact_response.get('artifacts')
    require(isinstance(artifacts, list), 'producer artifact API response is malformed')
    expected_name = f'weft-hosted-{version}-macos-arm64-{source_sha}'
    matches = [item for item in artifacts if item.get('name') == expected_name]
    require(len(matches) == 1, 'producer artifact is missing or duplicated')
    artifact = matches[0]
    require(artifact.get('expired') is False, 'producer artifact has expired')
    require(artifact.get('workflow_run', {}).get('id') == run_id, 'artifact belongs to another run')
    require(artifact.get('workflow_run', {}).get('head_sha') == source_sha, 'artifact source SHA differs')
    require(artifact.get('size_in_bytes', MAX_ARCHIVE + 1) <= MAX_ARCHIVE, 'artifact exceeds the 70 MiB archive cap')
    digest = artifact.get('digest')
    require(isinstance(digest, str) and re.fullmatch(r'sha256:[0-9a-f]{64}', digest) is not None, 'artifact API digest is absent or malformed')
    raw = archive_path.read_bytes()
    require(len(raw) <= MAX_ARCHIVE, 'downloaded artifact exceeds the 70 MiB archive cap')
    require(digest == 'sha256:' + sha(raw), 'downloaded ZIP digest differs from the artifact API')
    try:
        zf = zipfile.ZipFile(archive_path)
    except (OSError, zipfile.BadZipFile) as exc:
        raise VerificationError('artifact ZIP is invalid') from exc
    with zf:
        infos = zf.infolist()
        require(0 < len(infos) <= MAX_FILES, 'artifact member count is outside bounds')
        seen = set()
        expanded = 0
        members = {}
        for info in infos:
            name = info.filename
            path = pathlib.PurePosixPath(name)
            require(not path.is_absolute() and '..' not in path.parts and '\\' not in name, 'artifact contains an unsafe path')
            require(name not in seen, 'artifact contains a duplicate member')
            seen.add(name)
            mode = info.external_attr >> 16
            require(not stat.S_ISLNK(mode), 'artifact contains a symlink')
            if info.is_dir():
                continue
            require(len(path.parts) <= 2 and (len(path.parts) == 1 or path.parts[0] == 'web'), 'artifact member is outside the allowed root/web layout')
            limit = MAX_WHEEL if name.endswith('.whl') else 20 * 1024 * 1024 if name == 'web/weft_wasm_bg.wasm' else 5 * 1024 * 1024 if name == 'web/weft_wasm.js' else 2 * 1024 * 1024
            require(info.file_size <= limit, f'artifact member exceeds its size cap: {name}')
            expanded += info.file_size
            require(expanded <= MAX_ARCHIVE, 'expanded artifact exceeds the 70 MiB cap')
            members[name] = zf.read(info)
    required_names = {
        'producer-receipt.json', 'runner-preflight.json', 'tool-binary-sha256.txt',
        'compiler-roster.json', 'compiler-cases.json', 'compiler-reports.json',
        'python-compiler-parity.json', 'browser-compiler-parity.json',
        'browser-compiler-receipts.json', 'packaging-macos-arm64.json',
        'web/weft_wasm.js', 'web/weft_wasm_bg.wasm',
    }
    require(required_names <= members.keys(), 'artifact is missing required evidence members')
    receipt = json.loads(members['producer-receipt.json'])
    preflight = json.loads(members['runner-preflight.json'])
    require(receipt.get('schema') == 'weft-hosted-macos-arm64-producer/1' and receipt.get('status') == 'passed', 'producer receipt did not pass')
    require(receipt.get('sourceCommit') == source_sha and receipt.get('version') == version and receipt.get('distribution') == distribution, 'producer receipt identity differs')
    require(receipt.get('workflowCommit') == source_sha and str(receipt.get('workflowRunId')) == str(run_id), 'producer workflow identity differs')
    require(receipt.get('runner') == 'macos-14-arm64' and receipt.get('target') == 'aarch64-apple-darwin', 'producer target differs')
    require(receipt.get('compilerProfile') == 'nine-frozen-refusal-responses-rust-python-chromium/1', 'compiler profile differs')
    require(receipt.get('functionalQualification') == 'hosted-installed-wheel-refusal-parity-only', 'producer qualification scope differs')
    require(receipt.get('checks') == {'PREFLIGHT': 'success', 'WHEEL': 'success', 'VECTORS': 'success', 'PARITY': 'success'}, 'producer required checks are incomplete')
    require(receipt.get('parity') == {'rust': 'passed', 'installedPython': 'passed', 'chromiumWasm': 'passed'}, 'producer parity did not pass')
    require(preflight.get('sourceCommit') == source_sha and preflight.get('version') == version and preflight.get('distribution') == distribution, 'runner preflight identity differs')
    require(preflight.get('workflowCommit') == source_sha, 'runner preflight workflow identity differs')
    require(preflight.get('target') == 'aarch64-apple-darwin' and preflight.get('runner', {}).get('architecture') == 'arm64' and preflight.get('runner', {}).get('os') == 'Darwin', 'runner preflight is not Darwin ARM64')
    require(preflight.get('bun') == '1.4.2' and preflight.get('maturin') == 'maturin 1.9.6' and preflight.get('wasmBindgen') == 'wasm-bindgen 0.2.105', 'selected build tool versions differ')
    require(preflight.get('python', '').startswith('Python 3.12.') and preflight.get('node', '').startswith('v24.') and preflight.get('rustc', '').startswith('rustc 1.90.0 ') and preflight.get('cargo', '').startswith('cargo 1.90.0 '), 'selected runtime/compiler versions differ')
    require(receipt.get('runnerPreflightSha256') == sha(members['runner-preflight.json']) and receipt.get('toolBinaryHashesSha256') == sha(members['tool-binary-sha256.txt']), 'producer tool/preflight receipt hashes differ')
    features = preflight.get('maturinFeatures')
    require(isinstance(features, list) and bool(features), 'Maturin feature inputs are absent')
    require(preflight.get('maturinFeaturesSha256') == sha(json.dumps(features, separators=(',', ':')).encode()), 'Maturin feature input hash differs')
    artifact_rows = receipt.get('artifacts')
    require(isinstance(artifact_rows, list), 'producer artifact inventory is absent')
    listed = [row.get('name') for row in artifact_rows]
    require(len(listed) == len(set(listed)), 'producer artifact inventory contains duplicates')
    require((required_names - {'producer-receipt.json', 'runner-preflight.json', 'tool-binary-sha256.txt'}) <= set(listed), 'producer inventory omits required evidence')
    for row in artifact_rows:
        name = row.get('name')
        require(name in members, 'receipt names a missing artifact member')
        payload = members[name]
        require(row.get('bytes') == len(payload) and row.get('sha256') == sha(payload), 'producer member bytes differ from its receipt')
    prefix = wheel_prefix(distribution, version)
    wheels = [name for name in members if name.endswith('.whl')]
    require(len(wheels) == 1 and pathlib.PurePosixPath(wheels[0]).name.startswith(prefix) and '-cp39-abi3-macosx_' in wheels[0] and wheels[0].endswith('_arm64.whl'), 'artifact does not contain exactly the selected ARM64 ABI3 wheel')
    wheel_info = receipt.get('wheel', {})
    require(wheel_info.get('name') == wheels[0] and wheel_info.get('sha256') == sha(members[wheels[0]]), 'wheel bytes differ from the producer receipt')
    packaging = json.loads(members['packaging-macos-arm64.json'])
    require(packaging.get('status') == 'passed' and packaging.get('version') == version and packaging.get('distribution') == distribution, 'Mac packaging report identity differs')
    require(packaging.get('wheel') == wheels[0] and packaging.get('wheelSha256') == wheel_info.get('sha256'), 'Mac packaging report wheel differs')
    require(packaging.get('nativeModuleSha256') == wheel_info.get('nativeModuleSha256') and packaging.get('nativeModule') == wheel_info.get('nativeModule'), 'Mac native module hash/path differs')
    try:
        with zipfile.ZipFile(io.BytesIO(members[wheels[0]])) as wheel_zip:
            metadata_paths = [name for name in wheel_zip.namelist() if name.endswith('.dist-info/METADATA')]
            native_paths = [name for name in wheel_zip.namelist() if name.endswith(('.so', '.dylib', '.pyd'))]
            require(len(metadata_paths) == 1 and len(native_paths) == 1, 'ARM64 wheel metadata or module inventory is not unique')
            metadata = BytesParser().parsebytes(wheel_zip.read(metadata_paths[0]))
            require(metadata.get('Name') == distribution and metadata.get('Version') == version, 'ARM64 wheel metadata identity differs')
            module_sha = sha(wheel_zip.read(native_paths[0]))
    except zipfile.BadZipFile as exc:
        raise VerificationError('ARM64 wheel ZIP is invalid') from exc
    require(native_paths[0] == wheel_info.get('nativeModule') and module_sha == wheel_info.get('nativeModuleSha256'), 'embedded ARM64 native module differs from its receipt')
    cases = json.loads(members['compiler-cases.json'])
    roster = json.loads(members['compiler-roster.json'])
    require([item.get('id') for item in cases] == CASE_IDS and roster.get('cases') == CASE_IDS, 'compiler case roster differs')
    require(roster.get('profile') == 'nine-frozen-refusal-responses/1' and roster.get('selectedCasesSha256') == sha(members['compiler-cases.json']), 'compiler selected-case custody differs')
    require(all(item.get('expected', {}).get('status') == 'blocked' for item in cases), 'profile includes a non-refusal compiler case')
    py = json.loads(members['python-compiler-parity.json'])
    browser = json.loads(members['browser-compiler-parity.json'])
    rust = json.loads(members['compiler-reports.json'])
    require(py.get('status') == 'passed', 'Python parity did not pass')
    require(py.get('distribution') == distribution and py.get('distributionVersion') == version, 'Python parity package identity differs')
    require(py.get('nativeModuleSha256') == wheel_info.get('nativeModuleSha256'), 'Python native module hash differs')
    require([item.get('id') for item in py.get('cases', [])] == CASE_IDS, 'Python parity roster differs')
    require(browser.get('byteParity') is True and browser.get('cases') == 9, 'Chromium parity did not pass')
    require(len(rust) == 9 and [item.get('id') for item in rust] == CASE_IDS, 'Rust parity roster differs')
    try:
        validate_refusal_rows(cases, rust)
    except (ValueError, KeyError, TypeError) as error:
        raise VerificationError('Rust refusal response differs from selected profile') from error
    output_dir.mkdir(parents=True, exist_ok=True)
    for name, payload in members.items():
        destination = output_dir / pathlib.PurePosixPath(name)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_bytes(payload)
    return {
        'schema': 'weft-hosted-macos-artifact-verification/1', 'status': 'passed',
        'producerRunId': run_id, 'producerRunAttempt': run.get('run_attempt'),
        'sourceCommit': source_sha, 'workflowCommit': run.get('head_sha'),
        'workflowPath': run.get('path'), 'artifactId': artifact.get('id'),
        'artifactName': expected_name, 'artifactZipSha256': sha(raw),
        'artifactBytes': len(raw), 'wheel': wheel_info,
        'nativeModuleSha256': wheel_info.get('nativeModuleSha256'),
        'compilerProfile': receipt['compilerProfile'],
        'scope': 'exact hosted ARM64 wheel packaging plus nine frozen refusal-response Rust/Python/Chromium parity vectors; no positive-query or native PostgreSQL execution',
    }


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument('--archive', type=pathlib.Path)
    parser.add_argument('--run-json', type=pathlib.Path)
    parser.add_argument('--artifacts-json', type=pathlib.Path)
    parser.add_argument('--github-token', default=os.environ.get('GH_TOKEN', ''))
    parser.add_argument('--api-base', default='https://api.github.com')
    parser.add_argument('--run-id', type=int, required=True)
    parser.add_argument('--source-sha', required=True)
    parser.add_argument('--version', required=True)
    parser.add_argument('--distribution', required=True)
    parser.add_argument('--repository', required=True)
    parser.add_argument('--output-dir', type=pathlib.Path, required=True)
    parser.add_argument('--receipt', type=pathlib.Path, required=True)
    args = parser.parse_args()
    offline = all((args.archive, args.run_json, args.artifacts_json))
    if not offline:
        require(bool(args.github_token), 'GitHub token is required for hosted artifact verification')
        require(args.api_base == 'https://api.github.com', 'only the canonical GitHub API origin is supported')
        run_url = f'{args.api_base}/repos/{args.repository}/actions/runs/{args.run_id}'
        artifacts_url = f'{run_url}/artifacts?per_page=100'
        run_raw = _github_request(run_url, args.github_token, 'application/vnd.github+json', 2 * 1024 * 1024)
        artifacts_raw = _github_request(artifacts_url, args.github_token, 'application/vnd.github+json', 2 * 1024 * 1024)
        run = json.loads(run_raw)
        artifact_response = json.loads(artifacts_raw)
        expected_name = f'weft-hosted-{args.version}-macos-arm64-{args.source_sha}'
        matches = [item for item in artifact_response.get('artifacts', []) if item.get('name') == expected_name]
        require(len(matches) == 1, 'producer artifact is missing or duplicated')
        artifact_id = matches[0].get('id')
        require(isinstance(artifact_id, int), 'producer artifact ID is missing')
        archive_url = f'{args.api_base}/repos/{args.repository}/actions/artifacts/{artifact_id}/zip'
        archive_bytes = _github_request(archive_url, args.github_token, 'application/vnd.github+json', MAX_ARCHIVE)
        args.archive = args.output_dir.parent / f'producer-artifact-{args.run_id}.zip'
        args.archive.parent.mkdir(parents=True, exist_ok=True)
        args.archive.write_bytes(archive_bytes)
        args.run_json = args.output_dir.parent / f'producer-run-{args.run_id}.json'
        args.artifacts_json = args.output_dir.parent / f'producer-artifacts-{args.run_id}.json'
        args.run_json.write_bytes(run_raw)
        args.artifacts_json.write_bytes(artifacts_raw)
    require(args.archive is not None and args.run_json is not None and args.artifacts_json is not None, 'run metadata and artifact ZIP are required')
    receipt = verify_archive(
        archive_path=args.archive,
        run=json.loads(args.run_json.read_text()),
        artifact_response=json.loads(args.artifacts_json.read_text()),
        run_id=args.run_id, source_sha=args.source_sha, version=args.version,
        distribution=args.distribution, repository=args.repository,
        output_dir=args.output_dir,
    )
    args.receipt.parent.mkdir(parents=True, exist_ok=True)
    args.receipt.write_text(json.dumps(receipt, indent=2) + '\n')


if __name__ == '__main__':
    main()
