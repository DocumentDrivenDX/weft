import gzip
import hashlib
import importlib.util
import io
import json
from pathlib import Path
import sys
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('distribution_assembly', ROOT / 'scripts/distribution/assemble-distribution.py')
m = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = m
spec.loader.exec_module(m)


class AssemblyTests(unittest.TestCase):
    def test_relative_paths_refuse_all_escape_forms(self):
        for name in ('', '/outside', 'C:outside', '../outside', 'a/../b', 'a/./b', 'a//b', 'a/', 'a\\b', 'a\0b'):
            with self.subTest(name=name), self.assertRaises(ValueError):
                m.relative(name)
        self.assertEqual(m.relative('source/é.json'), 'source/é.json')

    def test_bounded_read_never_requests_whole_oversized_file(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d).resolve() / 'giant'
            p.write_bytes(b'x' * 50)
            requests = []
            original = Path.open
            class Reader:
                def __init__(self, stream): self.stream = stream
                def __enter__(self): return self
                def __exit__(self, *args): self.stream.close()
                def read(self, size=-1):
                    requests.append(size)
                    self.assert_size(size)
                    return self.stream.read(size)
                def assert_size(self, size):
                    if size != 11: raise AssertionError('unbounded read')
            with patch.object(Path, 'open', lambda path, *args, **kwargs: Reader(original(path, *args, **kwargs))):
                with self.assertRaises(ValueError): m.bounded_read(p, 10)
            self.assertEqual(requests, [11])

    def test_single_member_gzip_and_decoded_limit(self):
        raw = gzip.compress(b'123')
        self.assertEqual(m.inflate(raw, 3), b'123')
        for data in (raw + raw, raw + b'extra', raw[:-1], gzip.compress(b'1234')):
            with self.assertRaises((ValueError, __import__('zlib').error)):
                m.inflate(data, 3)

    def test_duplicate_or_nonfinite_json_refuses(self):
        for raw in (b'{"a":1,"a":2}', b'{"a":NaN}'):
            with self.assertRaises(ValueError): m.json_value(raw)

    def test_source_digest_git_blob_and_shape_correspondence(self):
        raw = b'original'
        entry = {'path': 'source/original', 'mode': '100644', 'gitBlob': hashlib.sha1(b'blob 8\0' + raw).hexdigest(), 'sha256': m.sha(raw), 'bytes': len(raw)}
        m.source_entry(raw, entry)
        for key, wrong in (('gitBlob', '0' * 40), ('sha256', '0' * 64), ('bytes', True), ('mode', '120000'), ('path', '../outside')):
            with self.subTest(key=key), self.assertRaises(ValueError):
                m.source_entry(raw, {**entry, key: wrong})
        with self.assertRaises(ValueError): m.source_entry(raw, {**entry, 'extra': 1})

    def test_snapshot_closing_drift_and_conflicting_destination(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            (root / 'a').write_bytes(b'original')
            (root / 'b').write_bytes(b'other')
            snapshot = m.Snapshot()
            snapshot.take(root, 'a', 'a')
            with self.assertRaises(ValueError): snapshot.take(root, 'b', 'a')
            (root / 'a').write_bytes(b'changed')
            with self.assertRaises(ValueError): snapshot.close()

    def test_symlink_component_refuses(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            (root / 'actual').mkdir()
            (root / 'actual/a').write_bytes(b'a')
            (root / 'link').symlink_to(root / 'actual', target_is_directory=True)
            with self.assertRaises(ValueError): m.Snapshot().take(root, 'link/a', 'a')

    def test_output_exists_refuses_before_input_reads(self):
        with tempfile.TemporaryDirectory() as d:
            p = Path(d).resolve()
            output = p / 'existing'; output.mkdir()
            config = m.Config(p, p, p, p / 'binary', output, 'candidate')
            with patch.object(m.Snapshot, 'take', side_effect=AssertionError('inputs reached')):
                with self.assertRaises(ValueError): m.assemble(config)

    def test_public_response_identity_and_framing(self):
        stdout = '{"status":"compiled"}\n'
        record = {'id': 'original', 'requestSha256': '1' * 64, 'exit': 0,
                  'stdoutSha256': m.sha(stdout.encode()), 'stderrSha256': m.sha(b''), 'stdout': stdout, 'stderr': ''}
        self.assertEqual(m.public_case(record)['status'], 'compiled')
        for change in ({'stdoutSha256': '0' * 64}, {'stderr': 'unexpected'}, {'exit': True}, {'extra': 1}, {'stdout': stdout + '\n'}):
            with self.subTest(change=change), self.assertRaises(ValueError): m.public_case({**record, **change})
        wrong = '{"status":"made-up"}\n'
        with self.assertRaises(ValueError): m.public_case({**record, 'stdout': wrong, 'stdoutSha256': m.sha(wrong.encode())})

    def test_missing_control_and_changed_request_refuse(self):
        rows = []
        for i, identity in enumerate(m.CONTROL_IDS):
            request = '{"original":"request"}'
            response = '{"status":"blocked","diagnostics":[{"code":"WFT-CAPABILITY"}]}\n' if i < 16 else '{"status":"compiled"}\n'
            rows.append({'id': identity, 'requestSha256': m.sha(request.encode()), 'requestJson': request,
                         'stdout': response, 'stderr': '', 'exit': 0, 'expectedCode': 'WFT-CAPABILITY' if i < 16 else None})
        def check(items):
            raw = b''.join(m.encoded(row) + b'\n' for row in items)
            snapshot = m.Snapshot()
            snapshot.artifacts['evidence/cli-candidate-controls-20261009/cases.jsonl.gz'] = gzip.compress(raw)
            m.verify_controls(snapshot, {'decodedCaseSha256': m.sha(raw), 'decodedCaseBytes': len(raw)},
                              {'cases': 19, 'blockedCases': 16, 'compiledCases': 3, 'custodyClosed': True})
        check(rows)
        with self.assertRaises(ValueError): check(rows[:-1])
        with self.assertRaises(ValueError): check([rows[0], *rows[:-1]])
        rows[0]['requestJson'] = '{"changed":true}'
        with self.assertRaises(ValueError): check(rows)

class PublicationTests(unittest.TestCase):
    def fixture(self, root):
        (root / 'input').write_bytes(b'exact')
        snap = m.Snapshot()
        snap.take(root, 'input', 'proof/input')
        snap.generated('manifest.json', b'{"inert":true}\n')
        return snap

    def test_atomic_copy_keeps_complete_bytes_and_modes(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            snap = self.fixture(root)
            m.publish(snap, root / 'output')
            self.assertEqual((root / 'output/proof/input').read_bytes(), b'exact')
            self.assertEqual((root / 'output/manifest.json').stat().st_mode & 0o777, 0o444)
            self.assertFalse(any(root.glob('.weft-distribution-*')))

    def test_rename_failure_leaves_no_candidate(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            snap = self.fixture(root)
            with patch.object(m.os, 'rename', side_effect=OSError('injected rename failure')):
                with self.assertRaises(OSError): m.publish(snap, root / 'output')
            self.assertFalse((root / 'output').exists())
            self.assertFalse(any(root.glob('.weft-distribution-*')))

    def test_closing_drift_after_copy_withholds_candidate(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            snap = self.fixture(root)
            original = snap.close
            calls = []
            def close():
                calls.append(1)
                if len(calls) == 2: (root / 'input').write_bytes(b'drift')
                original()
            with patch.object(snap, 'close', side_effect=close):
                with self.assertRaises(ValueError): m.publish(snap, root / 'output')
            self.assertFalse((root / 'output').exists())
            self.assertFalse(any(root.glob('.weft-distribution-*')))

    def test_symlink_replacement_at_closing_refuses(self):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d).resolve()
            snap = self.fixture(root)
            (root / 'input').unlink()
            (root / 'other').write_bytes(b'exact')
            (root / 'input').symlink_to(root / 'other')
            with self.assertRaises(ValueError): m.publish(snap, root / 'output')
            self.assertFalse((root / 'output').exists())


if __name__ == '__main__': unittest.main()
