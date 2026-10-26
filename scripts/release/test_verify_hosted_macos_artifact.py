#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
"""Focused hostile-metadata and byte-binding controls for hosted Mac assets."""
import hashlib
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, str(Path(__file__).parent))
from verify_hosted_macos_artifact import VerificationError, verify_archive
from finalize_hosted_macos import validate_refusal_rows

SOURCE = '1' * 40
VERSION = '0.4.55'
DISTRIBUTION = 'umf-weft'
RUN_ID = 123456
REPOSITORY = 'DocumentDrivenDX/weft'
CASE_IDS = [
    'original-sales-join-truss.postgresql', 'original-empty-global-truss.postgresql',
    'original-empty-grouped-truss.postgresql', 'original-sales-join-ashlar.databricks',
    'original-empty-global-ashlar.databricks', 'original-empty-grouped-ashlar.databricks',
    'original-string-exact-0', 'original-string-exact-1', 'original-string-exact-2',
]


def digest(payload: bytes) -> str:
    return hashlib.sha256(payload).hexdigest()


def fixture(root: Path, *, profile: str = 'nine-frozen-refusal-responses-rust-python-chromium/1'):
    wheel_name = 'umf_weft-0.4.55-cp39-abi3-macosx_11_0_arm64.whl'
    module_name = 'weft/weft.abi3.so'
    module = b'fixture-native-module'
    module_sha = digest(module)
    wheel_bytes = io.BytesIO()
    with zipfile.ZipFile(wheel_bytes, 'w') as wheel:
        wheel.writestr('umf_weft-0.4.55.dist-info/METADATA', 'Name: umf-weft\nVersion: 0.4.55\n\n')
        wheel.writestr(module_name, module)
    wheel = wheel_bytes.getvalue()
    cases = [{'id': case_id, 'expected': {'status': 'blocked', 'code': 'WFT-BACKEND-MISSING'}} for case_id in CASE_IDS]
    rust_rows = [{'id': case_id, 'raw': json.dumps({'status': 'blocked', 'diagnostics': [{'code': 'WFT-BACKEND-VERSION' if case_id.endswith('ashlar.databricks') else 'WFT-BACKEND-MISSING'}]})} for case_id in CASE_IDS]
    features = ['weft-runtime/serde']
    output = {
        wheel_name: wheel,
        'compiler-roster.json': json.dumps({'profile': 'nine-frozen-refusal-responses/1', 'cases': CASE_IDS, 'selectedCasesSha256': digest(json.dumps(cases).encode())}).encode(),
        'compiler-cases.json': json.dumps(cases).encode(),
        'compiler-reports.json': json.dumps(rust_rows).encode(),
        'python-compiler-parity.json': json.dumps({'status': 'passed', 'distribution': DISTRIBUTION, 'distributionVersion': VERSION, 'nativeModuleSha256': module_sha, 'cases': [{'id': item} for item in CASE_IDS]}).encode(),
        'browser-compiler-parity.json': json.dumps({'byteParity': True, 'cases': 9}).encode(),
        'browser-compiler-receipts.json': b'{}',
        'packaging-macos-arm64.json': json.dumps({'status': 'passed', 'version': VERSION, 'distribution': DISTRIBUTION, 'target': 'aarch64-apple-darwin', 'wheel': wheel_name, 'wheelSha256': digest(wheel), 'nativeModule': module_name, 'nativeModuleSha256': module_sha}).encode(),
        'web/weft_wasm.js': b'export{};',
        'web/weft_wasm_bg.wasm': b'wasm',
    }
    artifact_rows = [{'name': name, 'bytes': len(data), 'sha256': digest(data)} for name, data in output.items()]
    preflight = json.dumps({
        'sourceCommit': SOURCE, 'workflowCommit': SOURCE, 'version': VERSION, 'distribution': DISTRIBUTION,
        'target': 'aarch64-apple-darwin', 'runner': {'architecture': 'arm64', 'os': 'Darwin'},
        'bun': '1.4.2', 'maturin': 'maturin 1.9.6', 'wasmBindgen': 'wasm-bindgen 0.2.105',
        'python': 'Python 3.12.12', 'node': 'v24.10.0', 'rustc': 'rustc 1.90.0 (fixture)', 'cargo': 'cargo 1.90.0 (fixture)',
        'maturinFeatures': features,
        'maturinFeaturesSha256': digest(json.dumps(features, separators=(',', ':')).encode()),
    }).encode()
    tool_hashes = b'compiler  deadbeef\n'
    receipt = json.dumps({
        'schema': 'weft-hosted-macos-arm64-producer/1', 'status': 'passed',
        'sourceCommit': SOURCE, 'workflowCommit': SOURCE, 'workflowRunId': str(RUN_ID),
        'version': VERSION, 'distribution': DISTRIBUTION, 'runner': 'macos-14-arm64',
        'target': 'aarch64-apple-darwin', 'compilerProfile': profile,
        'functionalQualification': 'hosted-installed-wheel-refusal-parity-only',
        'checks': {'PREFLIGHT': 'success', 'WHEEL': 'success', 'VECTORS': 'success', 'PARITY': 'success'},
        'parity': {'rust': 'passed', 'installedPython': 'passed', 'chromiumWasm': 'passed'},
        'runnerPreflightSha256': digest(preflight), 'toolBinaryHashesSha256': digest(tool_hashes),
        'wheel': {'name': wheel_name, 'sha256': digest(wheel), 'nativeModule': module_name, 'nativeModuleSha256': module_sha},
        'artifacts': artifact_rows,
    }).encode()
    members = {**output, 'producer-receipt.json': receipt, 'runner-preflight.json': preflight, 'tool-binary-sha256.txt': tool_hashes}
    archive_path = root / 'producer.zip'
    with zipfile.ZipFile(archive_path, 'w', zipfile.ZIP_DEFLATED) as archive:
        for name, data in members.items():
            archive.writestr(name, data)
    raw = archive_path.read_bytes()
    run = {
        'id': RUN_ID, 'repository': {'full_name': REPOSITORY}, 'event': 'workflow_dispatch',
        'head_branch': 'main', 'status': 'completed', 'conclusion': 'success',
        'path': '.github/workflows/release-hosted-macos.yml', 'head_sha': SOURCE, 'run_attempt': 1,
    }
    artifact = {
        'id': 777, 'name': f'weft-hosted-{VERSION}-macos-arm64-{SOURCE}', 'expired': False,
        'size_in_bytes': len(raw), 'digest': 'sha256:' + digest(raw),
        'workflow_run': {'id': RUN_ID, 'head_sha': SOURCE},
    }
    return archive_path, run, {'artifacts': [artifact]}


class HostedMacArtifactTests(unittest.TestCase):
    def setUp(self):
        self.temporary = tempfile.TemporaryDirectory()
        self.root = Path(self.temporary.name)
        self.archive, self.run, self.artifacts = fixture(self.root)

    def tearDown(self):
        self.temporary.cleanup()

    def verify(self, *, run=None, artifacts=None, run_id=RUN_ID, source=SOURCE, archive=None):
        return verify_archive(
            archive_path=archive or self.archive, run=run or self.run,
            artifact_response=artifacts or self.artifacts, run_id=run_id,
            source_sha=source, version=VERSION, distribution=DISTRIBUTION,
            repository=REPOSITORY, output_dir=self.root / 'extracted',
        )

    def test_exact_run_archive_and_profile_pass(self):
        result = self.verify()
        self.assertEqual(result['status'], 'passed')
        self.assertEqual(result['compilerProfile'], 'nine-frozen-refusal-responses-rust-python-chromium/1')
        self.assertTrue((self.root / 'extracted/umf_weft-0.4.55-cp39-abi3-macosx_11_0_arm64.whl').is_file())

    def test_profile_reads_real_response_contract_and_retained_predecessor_expectations(self):
        rows = json.loads(Path('docs/helix/04-build/evidence/release-0.4.54-s2/compiler-reports.json').read_bytes())
        corpus = {row['id']: row for row in json.loads(Path('tests/compile/fixtures/cases.json').read_bytes())}
        cases = [corpus[case_id] for case_id in CASE_IDS]
        validate_refusal_rows(cases, rows)
        changed = json.loads(json.dumps(rows))
        changed[3]['raw'] = json.dumps({'status': 'blocked', 'diagnostics': [{'code': 'WFT-BACKEND-MISSING'}]})
        with self.assertRaisesRegex(ValueError, 'selected refusal'):
            validate_refusal_rows(cases, changed)
        changed[3]['raw'] = json.dumps({'status': 'compiled', 'sql': 'SELECT 1', 'diagnostics': []})
        with self.assertRaisesRegex(ValueError, 'selected refusal'):
            validate_refusal_rows(cases, changed)

    def test_wrong_source_or_run_identity_refuses(self):
        with self.assertRaisesRegex(VerificationError, 'head SHA|producer source SHA'):
            self.verify(source='2' * 40)
        with self.assertRaisesRegex(VerificationError, 'run ID'):
            self.verify(run_id=RUN_ID + 1)

    def test_wrong_artifact_zip_digest_refuses(self):
        tampered = json.loads(json.dumps(self.artifacts))
        tampered['artifacts'][0]['digest'] = 'sha256:' + '0' * 64
        with self.assertRaisesRegex(VerificationError, 'ZIP digest'):
            self.verify(artifacts=tampered)

    def test_unexpected_profile_refuses_even_with_matching_archive_digest(self):
        archive, run, artifacts = fixture(self.root, profile='nine-positive-cases/1')
        with self.assertRaisesRegex(VerificationError, 'profile differs'):
            self.verify(archive=archive, run=run, artifacts=artifacts)

    def test_non_main_or_wrong_workflow_run_refuses(self):
        run = dict(self.run, head_branch='feature')
        with self.assertRaisesRegex(VerificationError, 'main-branch'):
            self.verify(run=run)
        run = dict(self.run, path='.github/workflows/release.yml')
        with self.assertRaisesRegex(VerificationError, 'workflow path'):
            self.verify(run=run)

    def test_unsafe_archive_member_refuses(self):
        bad = self.root / 'unsafe.zip'
        with zipfile.ZipFile(self.archive) as source, zipfile.ZipFile(bad, 'w') as output:
            for info in source.infolist():
                output.writestr(info, source.read(info))
            output.writestr('../escape', b'no')
        artifacts = json.loads(json.dumps(self.artifacts))
        artifacts['artifacts'][0]['digest'] = 'sha256:' + digest(bad.read_bytes())
        artifacts['artifacts'][0]['size_in_bytes'] = bad.stat().st_size
        with self.assertRaisesRegex(VerificationError, 'unsafe path'):
            self.verify(archive=bad, artifacts=artifacts)

    def test_self_consistent_zip_cannot_omit_receipt_inventory(self):
        bad = self.root / 'omitted.zip'
        with zipfile.ZipFile(self.archive) as source, zipfile.ZipFile(bad, 'w') as output:
            for info in source.infolist():
                data = source.read(info)
                if info.filename == 'producer-receipt.json':
                    receipt = json.loads(data)
                    receipt['artifacts'] = []
                    data = json.dumps(receipt).encode()
                output.writestr(info, data)
        artifacts = json.loads(json.dumps(self.artifacts))
        artifacts['artifacts'][0].update(digest='sha256:' + digest(bad.read_bytes()), size_in_bytes=bad.stat().st_size)
        with self.assertRaisesRegex(VerificationError, 'inventory omits'):
            self.verify(archive=bad, artifacts=artifacts)


if __name__ == '__main__':
    unittest.main()
