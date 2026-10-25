# SPDX-License-Identifier: MIT OR Apache-2.0
"""Synthetic fail-closed controls for the 0.4.54 selected asset verifier."""
import hashlib
import importlib.util
import json
import unittest
from pathlib import Path

path = Path(__file__).with_name('verify_selected_assets_0454.py')
spec = importlib.util.spec_from_file_location('verify_selected_assets_0454', path)
verifier = importlib.util.module_from_spec(spec)
spec.loader.exec_module(verifier)


def valid_run(workflow, source='a' * 40, run_id=123):
    return {
        'repository': {'full_name': 'DocumentDrivenDX/weft'}, 'path': workflow,
        'conclusion': 'success', 'status': 'completed', 'head_sha': source,
        'event': 'workflow_dispatch', 'head_branch': 'main', 'id': run_id,
    }


def mac_receipt_and_preflight(source='a' * 40, run_id=456):
    features = ['weft-runtime/truss-postgresql-qualified']
    preflight = {
        'status': 'passed', 'sourceCommit': source, 'workflowCommit': source,
        'runner': {
            'label': 'macos-14',
            'architectureSource': 'https://docs.github.com/en/actions/reference/runners/github-hosted-runners',
            'os': 'Darwin', 'architecture': 'arm64', 'target': 'aarch64-apple-darwin',
        },
        'python': 'Python 3.12.1', 'bun': '1.4.2', 'node': 'v24.1.0', 'maturin': 'maturin 1.9.6',
        'rustc': 'rustc 1.90.0 (abc 2025-01-01)', 'cargo': 'cargo 1.90.0 (abc 2025-01-01)',
        'wasmBindgen': 'wasm-bindgen 0.2.105',
        'maturinFeatures': features,
        'maturinFeaturesSha256': hashlib.sha256(json.dumps(features, separators=(',', ':')).encode()).hexdigest(),
    }
    receipt = {
        'schema': 'weft-selected-macos-host/1', 'status': 'passed', 'version': '0.4.54',
        'sourceCommit': source, 'workflowCommit': source, 'workflowRunId': run_id,
        'functionalQualification': 'hosted-installed-wheel-compiler-parity-only',
        'checks': {'PREFLIGHT': 'success', 'WHEEL': 'success', 'VECTORS': 'success', 'PARITY': 'success'},
        'wheel': {'name': 'weft_sql-0.4.54-cp39-abi3-macosx_11_0_arm64.whl', 'sha256': 'b' * 64, 'nativeModuleSha256': 'c' * 64},
        'compilerRoster': verifier.MAC_CASES,
        'parity': {'rust': 'passed', 'python': 'passed', 'chromiumWasm': 'passed'},
        'runnerPreflightSha256': hashlib.sha256((json.dumps(preflight, indent=2) + '\n').encode()).hexdigest(),
    }
    return receipt, preflight


class ReleaseGateTests(unittest.TestCase):
    def test_frozen_s_m_release_workflow_positive_and_changed_workflow_negative(self):
        frozen = f"if: a && {verifier.RELEASE_EXCLUSION}\nif: b && {verifier.RELEASE_EXCLUSION}\n"
        verifier.validate_release_workflow(frozen, frozen)
        with self.assertRaises(ValueError):
            verifier.validate_release_workflow(frozen, frozen + '# metadata edit\n')
        with self.assertRaises(ValueError):
            verifier.validate_release_workflow(frozen.replace('v0.4.54', 'v0.4.53', 1), frozen)

    def test_linux_artifact_only_job_shape_accepts_skipped_publisher(self):
        jobs = {'jobs': [
            {'name': 'wheels (ubuntu-24.04, x86_64-unknown-linux-gnu, 2014)', 'status': 'completed', 'conclusion': 'success'},
            {'name': 'wheels (ubuntu-24.04-arm, aarch64-unknown-linux-gnu, 2014)', 'status': 'completed', 'conclusion': 'success'},
            {'name': 'publish', 'status': 'completed', 'conclusion': 'skipped'},
        ]}
        verifier.validate_linux_jobs(jobs)
        jobs['jobs'][2]['conclusion'] = 'success'
        with self.assertRaises(ValueError):
            verifier.validate_linux_jobs(jobs)
        with self.assertRaises(ValueError):
            verifier.validate_linux_jobs({'jobs': jobs['jobs'][:2]})
        jobs['jobs'][2]['conclusion'] = 'skipped'
        jobs['jobs'].append({'name': 'unexpected', 'status': 'completed', 'conclusion': 'skipped'})
        with self.assertRaises(ValueError):
            verifier.validate_linux_jobs(jobs)

    def test_wrong_workflow_and_wrong_source_runs_refuse(self):
        with self.assertRaises(ValueError):
            verifier.validate_run(valid_run('wrong.yml'), repository='DocumentDrivenDX/weft', workflow=verifier.RELEASE_WORKFLOW, source='a' * 40)
        with self.assertRaises(ValueError):
            verifier.validate_run(valid_run(verifier.RELEASE_WORKFLOW, source='b' * 40), repository='DocumentDrivenDX/weft', workflow=verifier.RELEASE_WORKFLOW, source='a' * 40)
        with self.assertRaises(ValueError):
            verifier.validate_run(valid_run(verifier.RELEASE_WORKFLOW, run_id=321), repository='DocumentDrivenDX/weft', workflow=verifier.RELEASE_WORKFLOW, source='a' * 40, run_id=123)

    def test_exact_mac_source_and_hosted_receipt_positive(self):
        receipt, preflight = mac_receipt_and_preflight()
        run = valid_run(verifier.MAC_WORKFLOW, source='a' * 40, run_id=456)
        verifier.validate_mac(receipt, preflight, run, source='a' * 40, run_id=456)
        run['head_sha'] = 'd' * 40
        with self.assertRaises(ValueError):
            verifier.validate_mac(receipt, preflight, run, source='a' * 40, run_id=456)

    def test_incomplete_hosted_receipt_and_wrong_module_digest_refuse(self):
        receipt, preflight = mac_receipt_and_preflight()
        run = valid_run(verifier.MAC_WORKFLOW, source='a' * 40, run_id=456)
        del receipt['runnerPreflightSha256']
        with self.assertRaises(ValueError):
            verifier.validate_mac(receipt, preflight, run, source='a' * 40, run_id=456)
        receipt, _ = mac_receipt_and_preflight()
        with self.assertRaises(ValueError):
            verifier.validate_module_binding(receipt, {'nativeModuleSha256': 'e' * 64}, 'c' * 64)
        with self.assertRaises(ValueError):
            verifier.validate_module_binding(receipt, {'nativeModuleSha256': 'c' * 64}, 'e' * 64)

    def test_unexpected_target_refuses(self):
        with self.assertRaises(ValueError):
            verifier.classify_wheels([Path('weft_sql-0.4.54-cp39-abi3-win_amd64.whl')] * 3)


if __name__ == '__main__':
    unittest.main()
