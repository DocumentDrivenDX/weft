# SPDX-License-Identifier: MIT OR Apache-2.0
import copy
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT / 'scripts/distribution'))
import implicit_arithmetic_distribution as owner

spec = importlib.util.spec_from_file_location('independent_expectations', Path(__file__).with_name('expectations.py'))
expectations = importlib.util.module_from_spec(spec)
spec.loader.exec_module(expectations)
FIXTURES = Path(__file__).with_name('fixtures')

class Codec:
    TRANSPORT_IDS = ("oversize", "invalid-utf8", "split-utf8-at-limit", "exact-limit", "exact-limit-multibyte", "malformed-json", "directory-input", "closed-output")
    @staticmethod
    def sha(raw):
        return hashlib.sha256(raw).hexdigest()
    @staticmethod
    def document(raw):
        return json.loads(raw)
    @staticmethod
    def relative(name):
        if not name or name.startswith('/') or '..' in Path(name).parts:
            raise ValueError('relative')
        return name
    @classmethod
    def descriptor(cls, name, raw):
        return {'path': name, 'bytes': len(raw), 'sha256': cls.sha(raw)}

class AssemblyControls(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.authored = json.loads(gzip.decompress((FIXTURES / 'authored-corpus.json.gz').read_bytes()))['cases']
        rows = []
        for kind in ('corpus', 'controls'):
            rows.extend(json.loads(gzip.decompress((FIXTURES / (kind + '-responses.json.gz')).read_bytes()))['cases'])
        by_id = {r['id']: r for r in rows}
        cls.rows = [by_id[c['id']] for c in cls.authored]
        cls.backend = json.loads((FIXTURES / 'backend-manifest.json').read_bytes())

    def verify(self, rows=None, backend=None):
        return owner.verify_records(self.rows if rows is None else rows, self.authored,
                                    self.backend if backend is None else backend, Codec, expectations.check)

    def test_every_delivered_component_matches_receipt_and_exact_set(self):
        artifacts = {destination: ('original-' + original).encode() for original, destination in owner.PRODUCT_NAMES.items()}
        products = [{'path': '/observed/' + original, 'bytes': len(artifacts[destination]), 'sha256': Codec.sha(artifacts[destination])}
                    for original, destination in owner.PRODUCT_NAMES.items()]
        receipt = {'records': [{'products': products}]}
        owner.verify_products(artifacts, receipt, Codec)
        changed = dict(artifacts)
        changed['components/browser/weft.js'] = b'altered worker'
        with self.assertRaisesRegex(ValueError, 'fresh-component'):
            owner.verify_products(changed, receipt, Codec)
        changed = dict(artifacts)
        del changed['components/python/weft_sql.whl']
        with self.assertRaisesRegex(ValueError, 'complete-delivered'):
            owner.verify_products(changed, receipt, Codec)
        changed = dict(artifacts)
        changed['components/browser/unobserved.js'] = b'extra'
        with self.assertRaisesRegex(ValueError, 'complete-delivered'):
            owner.verify_products(changed, receipt, Codec)

    def test_actual_build_phase_names_and_failure_refusal(self):
        receipt = {'records': [{'phase': p, 'capture': {'exitCode': 0}} for p in ('cli', 'python', 'wasm', 'wasm-bindings')]}
        owner.build_phases(receipt)
        receipt['records'][-1]['phase'] = 'wasm-web'
        with self.assertRaisesRegex(ValueError, 'build-phases'):
            owner.build_phases(receipt)
        receipt['records'][-1]['phase'] = 'wasm-bindings'
        receipt['records'][-1]['capture']['exitCode'] = 1
        with self.assertRaisesRegex(ValueError, 'build-phases'):
            owner.build_phases(receipt)

    def test_manifest_environment_subset_preserves_original_record(self):
        original = {'CARGO_HOME': 'cargo', 'RUSTUP_HOME': 'rustup', 'CARGO_TARGET_DIR': 'target', 'PATH': 'path', 'RUSTC': 'compiler'}
        before = dict(original)
        self.assertEqual(owner.build_environment(original), {'CARGO_HOME': 'cargo', 'RUSTUP_HOME': 'rustup', 'CARGO_TARGET_DIR': 'target', 'PATHPrefix': 'path'})
        self.assertEqual(original, before)

    def test_retained_original94_and_actual50(self):
        self.assertEqual(Codec.sha((FIXTURES / 'backend-manifest.json').read_bytes()), owner.BACKEND)
        groups = self.verify()
        self.assertEqual((len(groups['corpus']), len(groups['controls'])), (65, 29))
        partial = next(r for r in self.rows if r['id'].startswith('original-commerce:'))
        self.assertEqual(partial['status'], 'compiled')

    def test_changed_original_request_refuses(self):
        rows = copy.deepcopy(self.rows)
        rows[0]['requestHex'] = b'changed original SQL'.hex()
        with self.assertRaisesRegex(ValueError, 'original-request'):
            self.verify(rows)

    def test_rebound_changed_response_refuses(self):
        rows = copy.deepcopy(self.rows)
        response = json.loads(bytes.fromhex(rows[0]['responseHex']))
        response['diagnostics'][0]['code'] = 'FORGED'
        raw = json.dumps(response).encode() + b'\n'
        rows[0]['responseHex'] = raw.hex()
        rows[0]['responseSha256'] = Codec.sha(raw)
        with self.assertRaises(ValueError):
            self.verify(rows)

    def test_changed_witness_and_same_count_capabilities_refuse(self):
        rows = copy.deepcopy(self.rows)
        row = next(r for r in rows if r['status'] == 'compiled')
        row['requiredCapabilities'] = []
        with self.assertRaisesRegex(ValueError, 'response-witness'):
            self.verify(rows)
        backend = copy.deepcopy(self.backend)
        backend['capabilities'][0]['id'] = 'forged.capability'
        with self.assertRaisesRegex(ValueError, 'actual-capability'):
            self.verify(backend=backend)

    def test_mutated_transport_exit_and_input_digest_refuse(self):
        raw = (FIXTURES / 'transport.jsonl').read_bytes()
        self.assertEqual(len(owner.verify_transport(raw, Codec)), 8)
        for key, replacement in [('exit', 0), ('inputSha256', '0' * 64)]:
            records = [json.loads(line) for line in raw.splitlines()]
            records[0][key] = replacement
            changed = b''.join(json.dumps(r, separators=(',', ':')).encode() + b'\n' for r in records)
            with self.assertRaisesRegex(ValueError, 'actual-transport'):
                owner.verify_transport(changed, Codec)

    def test_postcomparison_cleanup_failure_withholds_embedding_claim(self):
        parity = {'success': True, 'failure': None, 'cases': [{}] * 94}
        outcome = {'openingClosingCustody': True, 'failure': None, 'capture': {'exitCode': 0}}
        owner.verify_parity_outcome(parity, outcome)
        outcome['openingClosingCustody'] = False
        outcome['failure'] = 'closing failure'
        with self.assertRaisesRegex(ValueError, 'embedding-closing'):
            owner.verify_parity_outcome(parity, outcome)
        outcome = {'openingClosingCustody': True, 'failure': None, 'capture': {'exitCode': 1}}
        with self.assertRaisesRegex(ValueError, 'embedding-exit'):
            owner.verify_parity_outcome(parity, outcome)

    def test_fresh_publication_preserves_readonly_and_no_clobber(self):
        with tempfile.TemporaryDirectory() as td:
            output = Path(td).resolve() / 'candidate'
            snapshot = owner.Snapshot(Codec)
            snapshot.generated(owner.EXECUTABLE, b'original executable')
            snapshot.generated('manifest.json', b'original manifest')
            owner.publish(snapshot, output)
            self.assertEqual((output / owner.EXECUTABLE).stat().st_mode & 0o777, 0o555)
            self.assertEqual((output / 'manifest.json').stat().st_mode & 0o777, 0o444)
            with self.assertRaisesRegex(ValueError, 'fresh-contained'):
                owner.publish(snapshot, output)
            self.assertEqual((output / 'manifest.json').read_bytes(), b'original manifest')

    def test_symlink_output_refuses_before_copy(self):
        with tempfile.TemporaryDirectory() as td:
            output = Path(td).resolve() / 'candidate'
            output.symlink_to(Path(td).resolve() / 'absent')
            with self.assertRaisesRegex(ValueError, 'fresh-contained'):
                owner.publish(owner.Snapshot(Codec), output)

    def test_body_cancellation_identity_survives_cleanup_error(self):
        cancel = KeyboardInterrupt('original')
        with tempfile.TemporaryDirectory() as td:
            snapshot = owner.Snapshot(Codec)
            snapshot.generated('manifest.json', b'original')
            with patch.object(owner, 'write_exclusive', side_effect=cancel), patch.object(owner.shutil, 'rmtree', side_effect=OSError('cleanup')):
                with self.assertRaises(KeyboardInterrupt) as found:
                    owner.publish(snapshot, Path(td).resolve() / 'candidate')
            self.assertIs(found.exception, cancel)

    def test_closing_cancellation_outranks_ordinary_body_failure(self):
        cancel = SystemExit('closing')
        with tempfile.TemporaryDirectory() as td:
            snapshot = owner.Snapshot(Codec)
            snapshot.generated('manifest.json', b'original')
            with patch.object(owner, 'write_exclusive', side_effect=ValueError('body')), patch.object(owner.shutil, 'rmtree', side_effect=cancel):
                with self.assertRaises(SystemExit) as found:
                    owner.publish(snapshot, Path(td).resolve() / 'candidate')
            self.assertIs(found.exception, cancel)

if __name__ == '__main__':
    unittest.main()
