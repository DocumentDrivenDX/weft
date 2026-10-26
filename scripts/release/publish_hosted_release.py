#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Assemble and publish a verified hosted-Mac three-platform wheel release."""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import tomllib
import zipfile
from email.parser import BytesParser

from publish_pypi_release import distribution_for_version, wheel_prefix_for_version

FROZEN_INPUTS = [
    'Cargo.toml', 'Cargo.lock', 'crates', 'spec', 'vendor', '.cargo',
    'rust-toolchain', 'rust-toolchain.toml', 'docs/helix/02-design/contracts',
    'tests/compile/fixtures/cases.json', 'packages/weft-browser',
    '.github/workflows/release-hosted-macos.yml',
    'scripts/release/compare_selected_python.py',
    'scripts/release/compare_selected_rust_0454.py',
    'scripts/release/finalize_hosted_macos.py',
    'scripts/release/record_preflight_0454.py',
    'scripts/release/run_bounded_0454.py',
    'scripts/release/verify-wheel-packaging.py',
    'scripts/release/verify_hosted_macos_artifact.py',
    'scripts/release/verify_toolchain_0454.py',
]
MAC_EVIDENCE = {
    'producer-receipt.json': 'mac-hosted-producer-receipt-{version}.json',
    'runner-preflight.json': 'mac-runner-preflight-{version}.json',
    'tool-binary-sha256.txt': 'mac-tool-binary-sha256-{version}.txt',
    'compiler-roster.json': 'compiler-roster-{version}.json',
    'compiler-cases.json': 'compiler-cases-{version}.json',
    'compiler-reports.json': 'rust-compiler-reports-{version}.json',
    'python-compiler-parity.json': 'python-compiler-parity-{version}.json',
    'browser-compiler-parity.json': 'browser-compiler-parity-{version}.json',
    'browser-compiler-receipts.json': 'browser-compiler-receipts-{version}.json',
    'packaging-macos-arm64.json': 'packaging-macos-arm64-{version}.json',
}


class ReleaseError(ValueError):
    pass


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ReleaseError(message)


def sha(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def command(args: list[str], *, check: bool = True, timeout: int = 300) -> subprocess.CompletedProcess[str]:
    try:
        result = subprocess.run(args, text=True, capture_output=True, timeout=timeout)
    except subprocess.TimeoutExpired as error:
        raise ReleaseError(f"command timed out after {timeout}s: {' '.join(args[:3])}") from error
    if check and result.returncode:
        raise ReleaseError(f"command failed: {' '.join(args[:3])}: {result.stderr[-500:]}")
    return result


def producer_intent(source_sha: str, workflow_commit: str, workflow_run_id: int) -> dict:
    """Fields consumed by the canonical main-dispatch PyPI release validator."""
    return {
        'commit': source_sha,
        'workflowCommit': workflow_commit,
        'workflowRunId': workflow_run_id,
    }


def verify_release_api(release: dict, *, release_id: int, tag: str, draft: bool,
                       expected_digests: dict[str, str] | None = None,
                       immutable: bool | None = None) -> list[dict]:
    require(release.get('id') == release_id, 'release API ID differs from the numeric release ID')
    require(release.get('tag_name') == tag, 'release API tag differs')
    require(release.get('draft') is draft, 'release API draft state differs')
    if immutable is not None:
        require(release.get('immutable') is immutable, 'release API immutable state differs')
    assets = release.get('assets')
    require(isinstance(assets, list), 'release API asset inventory is malformed')
    names = [asset.get('name') for asset in assets]
    require(all(isinstance(name, str) for name in names) and len(names) == len(set(names)), 'release API asset names are invalid or duplicated')
    if expected_digests is not None:
        require(set(names) == set(expected_digests), 'draft release API asset set differs')
        for asset in assets:
            require(asset.get('digest') == 'sha256:' + expected_digests[asset['name']], f"draft release API digest differs: {asset['name']}")
    return assets


def list_tag_releases(repository: str, tag: str) -> list[dict]:
    result = command(['gh', 'api', '--paginate', '--slurp', f'repos/{repository}/releases?per_page=100'])
    pages = json.loads(result.stdout)
    require(isinstance(pages, list), 'release list API response is malformed')
    rows = [item for page in pages for item in page] if all(isinstance(page, list) for page in pages) else pages
    require(all(isinstance(row, dict) for row in rows), 'release list API contains malformed entries')
    return [row for row in rows if row.get('tag_name') == tag]


def api_tag_commit(repository: str, tag: str) -> str | None:
    ref = command(['gh', 'api', f'repos/{repository}/git/ref/tags/{tag}'], check=False)
    if ref.returncode:
        if 'HTTP 404' in ref.stderr or 'Not Found' in ref.stderr:
            return None
        raise ReleaseError('could not resolve release tag through GitHub API')
    obj = json.loads(ref.stdout).get('object', {})
    if obj.get('type') == 'tag':
        obj = json.loads(command(['gh', 'api', f"repos/{repository}/git/tags/{obj.get('sha')}"]).stdout).get('object', {})
    require(obj.get('type') == 'commit' and re.fullmatch(r'[0-9a-f]{40}', obj.get('sha', '')) is not None, 'release tag does not resolve directly to a commit')
    return obj['sha']


def verify_run(run_id: int, repository: str, allowed_paths: set[str], *, allow_running_current: bool = False) -> dict:
    run = json.loads(command(['gh', 'api', f'repos/{repository}/actions/runs/{run_id}']).stdout)
    require(run.get('repository', {}).get('full_name') == repository, 'packaging producer repository differs')
    require(run.get('path') in allowed_paths, 'packaging producer workflow path differs')
    if not allow_running_current:
        require(run.get('status') == 'completed' and run.get('conclusion') == 'success', 'packaging producer run is not successful')
    require(re.fullmatch(r'[0-9a-f]{40}', run.get('head_sha', '')) is not None, 'packaging producer head SHA is malformed')
    return run


def package_report(path: Path, wheel: Path, version: str, distribution: str, source_commit: str, target: str) -> dict:
    report = json.loads(path.read_text())
    require(report.get('status') == 'passed' and report.get('version') == version, f'wheel packaging did not pass for {target}')
    require(report.get('wheel') == wheel.name and report.get('sourceCommit') == source_commit, f'wheel packaging source/name differs for {target}')
    require(report.get('target') == target, f'wheel target differs for {target}')
    require(report.get('distribution') == distribution, f'wheel distribution differs for {target}')
    raw = wheel.read_bytes()
    require(report.get('wheelSha256') == sha(raw), f'wheel hash differs from packaging report for {target}')
    with zipfile.ZipFile(wheel) as archive:
        metadata_files = [name for name in archive.namelist() if name.endswith('.dist-info/METADATA')]
        modules = [name for name in archive.namelist() if name.endswith(('.so', '.pyd'))]
        require(len(metadata_files) == 1 and len(modules) == 1, f'wheel metadata/module inventory differs for {target}')
        metadata = BytesParser().parsebytes(archive.read(metadata_files[0]))
        require(metadata.get('Name') == distribution and metadata.get('Version') == version, f'wheel metadata identity differs for {target}')
        module_sha = sha(archive.read(modules[0]))
    require(report.get('nativeModule') == modules[0] and report.get('nativeModuleSha256') == module_sha, f'wheel native module hash differs for {target}')
    return report


def main() -> None:
    parser = __import__('argparse').ArgumentParser(description=__doc__)
    parser.add_argument('--dist', type=Path, required=True)
    args = parser.parse_args()
    dist = args.dist
    repository = os.environ['GH_REPOSITORY']
    tag = os.environ['RELEASE_TAG']
    source_sha = os.environ['SOURCE_SHA']
    macos_run_id = int(os.environ['MACOS_RUN_ID'])
    packaging_run_id = os.environ.get('PACKAGING_RUN_ID', '')
    current_run_id = int(os.environ['GITHUB_RUN_ID'])
    project = tomllib.loads(Path('crates/weft-python/pyproject.toml').read_text())['project']
    version = project['version']
    distribution = project['name']
    require(tag == 'v' + version, 'release tag and project version differ')
    require(distribution == distribution_for_version(version) == 'umf-weft', 'hosted producer route is only for the renamed distribution')
    note = Path(f'docs/helix/05-deploy/releases/{tag}.md')
    require(note.is_file(), 'versioned release notes must be present before creating the release tag')
    require(re.fullmatch(r'[0-9a-f]{40}', source_sha) is not None, 'macOS source SHA is malformed')
    head = command(['git', 'rev-parse', 'HEAD']).stdout.strip()
    require(head == source_sha == os.environ.get('GITHUB_SHA'), 'release source must be the exact reviewed default-branch commit')
    require(source_sha == os.environ.get('MACOS_SOURCE_SHA', source_sha), 'macOS source input changed')
    subprocess.run(['git', 'diff', '--exit-code', source_sha, head, '--', *FROZEN_INPUTS], check=True, timeout=60)
    existing_tag = command(['git', 'rev-parse', f'refs/tags/{tag}^{{commit}}'], check=False)
    if existing_tag.returncode == 0:
        require(existing_tag.stdout.strip() == head, 'existing release tag points to another source commit')

    if packaging_run_id:
        require(re.fullmatch(r'[0-9]+', packaging_run_id) is not None, 'packaging run ID is malformed')
        packaging_run = verify_run(int(packaging_run_id), repository, {'.github/workflows/release.yml'})
        packaging_commit = packaging_run['head_sha']
    else:
        packaging_run = verify_run(current_run_id, repository, {'.github/workflows/release.yml'}, allow_running_current=True)
        packaging_commit = head
    require(packaging_commit == source_sha, 'Linux and macOS wheels must use the same exact source commit')
    subprocess.run(['git', 'merge-base', '--is-ancestor', packaging_commit, head], check=True, timeout=30)
    subprocess.run(['git', 'diff', '--exit-code', packaging_commit, head, '--', *FROZEN_INPUTS], check=True, timeout=60)

    mac_verification = json.loads((dist / 'macos-artifact-verification.json').read_text())
    require(mac_verification.get('status') == 'passed' and mac_verification.get('producerRunId') == macos_run_id, 'hosted Mac artifact verification is missing')
    require(mac_verification.get('sourceCommit') == source_sha and mac_verification.get('workflowCommit') == source_sha, 'hosted Mac source identity differs')
    require(mac_verification.get('compilerProfile') == 'nine-frozen-refusal-responses-rust-python-chromium/1', 'hosted Mac compiler profile differs')
    mac_receipt = json.loads((dist / 'macos/producer-receipt.json').read_text())
    preflight = json.loads((dist / 'macos/runner-preflight.json').read_text())
    project_features = tomllib.loads(Path('crates/weft-python/pyproject.toml').read_text())['tool']['maturin']['features']
    require(preflight.get('maturinFeatures') == project_features, 'Mac Maturin features differ from the tagged source')
    require(mac_receipt.get('workflowCommit') == source_sha and mac_receipt.get('sourceCommit') == source_sha, 'Mac producer helper/source inputs differ')

    linux_wheels = sorted(dist.glob('*.whl'))
    linux_reports = sorted(path for path in dist.glob('packaging-*.json') if path.name != 'packaging-macos-arm64.json')
    require(len(linux_wheels) == 2 and len(linux_reports) == 2, 'both Linux packaging artifacts are required')
    expected = {
        'x86_64-unknown-linux-gnu': 'manylinux_2_17_x86_64.manylinux2014_x86_64.whl',
        'aarch64-unknown-linux-gnu': 'manylinux_2_17_aarch64.manylinux2014_aarch64.whl',
    }
    linux_packaging = []
    for target, suffix in expected.items():
        wheel_matches = [path for path in linux_wheels if path.name.startswith(wheel_prefix_for_version(version)) and path.name.endswith(suffix)]
        report_matches = [path for path in linux_reports if target in path.name]
        require(len(wheel_matches) == len(report_matches) == 1, f'Linux target artifact set is incomplete for {target}')
        linux_packaging.append(package_report(report_matches[0], wheel_matches[0], version, distribution, packaging_commit, target))

    mac_wheels = sorted((dist / 'macos').glob('*.whl'))
    require(len(mac_wheels) == 1, 'hosted Mac artifact must contain exactly one wheel')
    mac_report_path = dist / 'macos/packaging-macos-arm64.json'
    mac_report = package_report(mac_report_path, mac_wheels[0], version, distribution, source_sha, 'aarch64-apple-darwin')
    all_wheels = linux_wheels + mac_wheels
    require(len(all_wheels) == 3 and len({path.name for path in all_wheels}) == 3, 'release must contain exactly three unique wheels')

    asset_dir = dist / 'release-assets'
    if asset_dir.exists():
        shutil.rmtree(asset_dir)
    asset_dir.mkdir(parents=True)
    for wheel in all_wheels:
        shutil.copyfile(wheel, asset_dir / wheel.name)
    for report_path in linux_reports:
        shutil.copyfile(report_path, asset_dir / report_path.name)
    for relative, template in MAC_EVIDENCE.items():
        source = dist / 'macos' / relative
        require(source.is_file(), f'required hosted Mac receipt is missing: {relative}')
        shutil.copyfile(source, asset_dir / template.format(version=version))
    shutil.copyfile(dist / 'macos-artifact-verification.json', asset_dir / f'mac-artifact-verification-{version}.json')
    shutil.copyfile(note, asset_dir / note.name)

    sums = ''.join(f'{sha(path.read_bytes())}  {path.name}\n' for path in sorted(all_wheels, key=lambda item: item.name))
    (asset_dir / 'SHA256SUMS').write_text(sums)
    features = project_features
    pins = {
        'schema': 'weft-hosted-release-pins/1', 'tag': tag, 'metadataCommit': head,
        'sourceCommit': source_sha, 'distribution': distribution, 'version': version,
        'releaseWorkflowRunId': current_run_id, 'packagingRunId': int(packaging_run_id) if packaging_run_id else current_run_id,
        'macosProducerRunId': macos_run_id, 'macosProducerArtifactId': mac_verification['artifactId'],
        'macosProducerArtifactZipSha256': mac_verification['artifactZipSha256'],
        'macosCompilerProfile': mac_verification['compilerProfile'],
        'macosProducerReceiptSha256': sha((dist / 'macos/producer-receipt.json').read_bytes()),
        'macosRunnerPreflightSha256': sha((dist / 'macos/runner-preflight.json').read_bytes()),
        'macosToolBinaryHashesSha256': sha((dist / 'macos/tool-binary-sha256.txt').read_bytes()),
        'maturinFeatures': features,
        'maturinFeaturesSha256': sha(json.dumps(features, separators=(',', ':')).encode()),
        'wheelPackaging': [*linux_packaging, mac_report],
        'wheelSha256': {path.name: sha(path.read_bytes()) for path in all_wheels},
        'nativeModuleSha256': {row['wheel']: row['nativeModuleSha256'] for row in [*linux_packaging, mac_report]},
        'qualification': 'Three exact wheel packages; Mac Rust/installed-Python/Chromium parity covers nine frozen refusal responses only. Linux is packaging-only. No positive-query or native PostgreSQL execution is claimed.',
    }
    pins.update(producer_intent(source_sha, head, current_run_id))
    (asset_dir / 'release-pins.json').write_text(json.dumps(pins, indent=2) + '\n')
    manifest = {
        'schema': 'weft-release-assets/1', 'tag': tag, 'version': version,
        'sourceCommit': source_sha, 'metadataCommit': head, 'status': 'verified-before-publication',
        'expectedWheelCount': 3, 'wheels': pins['wheelSha256'],
        'assetSha256': {path.name: sha(path.read_bytes()) for path in sorted(asset_dir.iterdir()) if path.is_file()},
        'limits': pins['qualification'],
    }
    (asset_dir / f'release-assets-{version}.json').write_text(json.dumps(manifest, indent=2) + '\n')

    # Release-by-tag lookup omits drafts on some GitHub API paths. Enumerate
    # releases, then use the numeric release ID for every draft/public readback.
    releases = list_tag_releases(repository, tag)
    require(len(releases) <= 1, 'multiple GitHub releases use the selected tag')
    release = releases[0] if releases else None
    if release is not None:
        require(release.get('draft') is True, 'existing release is already public; refusing asset changes')
    tag_commit = api_tag_commit(repository, tag)
    if tag_commit is None:
        created_ref = command([
            'gh', 'api', '--method', 'POST', f'repos/{repository}/git/refs',
            '--field', f'ref=refs/tags/{tag}', '--field', f'sha={head}',
        ], check=False)
        if created_ref.returncode != 0:
            # A timed-out/unknown create may have succeeded; accept only an
            # exact API readback of the intended ref target.
            require(api_tag_commit(repository, tag) == head, 'could not create the exact release tag ref')
        else:
            ref_object = json.loads(created_ref.stdout).get('object', {})
            require(ref_object.get('sha') == head, 'created release tag points to unexpected source')
        tag_commit = api_tag_commit(repository, tag)
    require(tag_commit == head, 'existing release tag points to another source commit')
    if release is not None:
        require(tag_commit == head, 'existing draft release tag does not point to the verified source')
        release_id = release.get('id')
        require(type(release_id) is int, 'existing draft release has no numeric release ID')
    else:
        request = command([
            'gh', 'api', '--method', 'POST', f'repos/{repository}/releases',
            '--field', f'tag_name={tag}', '--field', f'target_commitish={head}',
            '--field', f'name={tag}', '--field', f'body={note.read_text()}',
            '-F', 'draft=true', '-F', 'prerelease=false',
        ], timeout=120)
        release = json.loads(request.stdout)
        release_id = release.get('id')
        require(type(release_id) is int, 'draft release creation returned no numeric release ID')
        require(release.get('draft') is True and release.get('tag_name') == tag, 'draft release creation returned the wrong release')
        require(api_tag_commit(repository, tag) == head, 'created release tag does not point to the verified source')
    draft_identity = json.loads(command(['gh', 'api', f'repos/{repository}/releases/{release_id}']).stdout)
    verify_release_api(draft_identity, release_id=release_id, tag=tag, draft=True)
    command([sys.executable, 'scripts/release/sync_release_assets.py', '--tag', tag, '--asset-dir', str(asset_dir)])
    expected_digests = {path.name: sha(path.read_bytes()) for path in asset_dir.iterdir() if path.is_file()}
    draft = json.loads(command(['gh', 'api', f'repos/{repository}/releases/{release_id}']).stdout)
    draft_assets = verify_release_api(draft, release_id=release_id, tag=tag, draft=True, expected_digests=expected_digests)
    command(['gh', 'api', '--method', 'PATCH', f'repos/{repository}/releases/{release_id}',
             '-F', 'draft=false', '--raw-field', 'make_latest=true'], timeout=120)
    published = json.loads(command(['gh', 'api', f'repos/{repository}/releases/{release_id}']).stdout)
    api_assets = verify_release_api(published, release_id=release_id, tag=tag, draft=False,
                                    expected_digests=expected_digests, immutable=True)
    api_rows = []
    for item in sorted(api_assets, key=lambda row: row['name']):
        digest = item.get('digest')
        require(digest == 'sha256:' + expected_digests[item['name']], f'public release API digest differs: {item["name"]}')
        api_rows.append({'name': item['name'], 'assetId': item['id'], 'bytes': item['size'], 'digest': digest, 'browserDownloadUrl': item['browser_download_url']})
    receipt = {
        'schema': 'weft-published-release-api/1', 'status': 'passed',
        'releaseId': published['id'], 'tag': tag, 'url': published['html_url'],
        'sourceCommit': source_sha, 'metadataCommit': head, 'immutable': True, 'assets': api_rows,
        'limits': [pins['qualification']],
    }
    (dist / 'release-api-assets.json').write_text(json.dumps(receipt, indent=2) + '\n')
    (dist / 'release-pins.json').write_text(json.dumps(pins, indent=2) + '\n')
    shutil.copyfile(asset_dir / 'SHA256SUMS', dist / 'SHA256SUMS')
    print(json.dumps({'status': 'passed', 'tag': tag, 'releaseUrl': published['html_url'], 'wheelCount': 3, 'macosProducerRunId': macos_run_id}, indent=2))


if __name__ == '__main__':
    try:
        main()
    except (ReleaseError, subprocess.CalledProcessError, KeyError, ValueError) as error:
        raise SystemExit(str(error)) from error
