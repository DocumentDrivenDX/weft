"""Adversarial controls for the actual public-source metadata boundary."""
import gzip
import hashlib
import importlib.util
import json
from pathlib import Path
import tempfile
import subprocess
import sys
import unittest

ROOT = Path(__file__).resolve().parents[2]
spec = importlib.util.spec_from_file_location('distribution_metadata', ROOT / 'scripts/distribution/describe-backend.py')
metadata = importlib.util.module_from_spec(spec)
spec.loader.exec_module(metadata)


class SourceCustody(unittest.TestCase):
    def setUp(self):
        self.directory = tempfile.TemporaryDirectory()
        self.addCleanup(self.directory.cleanup)
        self.base = Path(self.directory.name).resolve()
        self.root = self.base / 'source'
        self.root.mkdir()
        self.inventory = self.base / 'inventory.json.gz'
        self.source = self.root / 'module.rs'
        self.source.write_bytes(b'pub struct Original;\n')
        self.source.chmod(0o644)
        data = self.source.read_bytes()
        self.entries = [{'path': 'module.rs', 'mode': '100644',
                         'gitBlob': hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest(),
                         'sha256': hashlib.sha256(data).hexdigest(), 'bytes': len(data)}]
        self.save()

    def save(self):
        with gzip.open(self.inventory, 'wb') as output:
            output.write(json.dumps(self.entries).encode())

    def refused(self):
        with self.assertRaises((ValueError, FileNotFoundError)):
            metadata.verify_source(self.root, self.inventory)

    def test_complete_original_passes(self):
        self.assertEqual(metadata.verify_source(self.root, self.inventory)[2], 1)

    def test_mutated_source_refuses(self):
        self.source.write_bytes(b'pub struct Replacement;\n')
        self.refused()

    def test_unlisted_source_refuses(self):
        (self.root / 'unexpected.rs').write_text('unlisted')
        self.refused()

    def test_missing_source_refuses(self):
        self.source.unlink()
        self.refused()

    def test_file_symlink_refuses_even_matching_bytes(self):
        retained = self.base / 'retained.rs'
        retained.write_bytes(self.source.read_bytes())
        self.source.unlink()
        self.source.symlink_to(retained)
        self.refused()

    def test_parent_symlink_refuses_even_matching_bytes(self):
        retained = self.base / 'retained'
        retained.mkdir()
        (retained / 'module.rs').write_bytes(self.source.read_bytes())
        (self.root / 'alias').symlink_to(retained, target_is_directory=True)
        self.entries[0]['path'] = 'alias/module.rs'
        self.save()
        self.refused()

    def test_traversal_refuses(self):
        self.entries[0]['path'] = '../source/module.rs'
        self.save()
        self.refused()

    def test_duplicate_path_refuses(self):
        self.entries.append(dict(self.entries[0]))
        self.save()
        self.refused()

    def test_duplicate_json_member_refuses(self):
        raw = json.dumps(self.entries).replace('"mode": "100644"', '"mode": "100644", "mode": "100644"')
        with gzip.open(self.inventory, 'wb') as output:
            output.write(raw.encode())
        self.refused()

    def test_git_blob_drift_refuses(self):
        self.entries[0]['gitBlob'] = '0' * 40
        self.save()
        self.refused()

    def test_executable_mode_drift_refuses(self):
        self.source.chmod(0o755)
        self.refused()

    def test_boolean_byte_count_refuses(self):
        self.entries[0]['bytes'] = True
        self.save()
        self.refused()

    def test_compressed_inventory_limit_refuses(self):
        self.inventory.write_bytes(b'x' * (metadata.LIMIT + 1))
        self.refused()

    def test_oversized_source_refuses_before_read(self):
        with self.source.open('wb') as output:
            output.truncate(metadata.SOURCE_FILE_LIMIT + 1)
        self.entries[0]['bytes'] = metadata.SOURCE_FILE_LIMIT + 1
        self.save()
        self.refused()

    def test_cli_configuration_error_excludes_raw_argument(self):
        sentinel = 'synthetic-sensitive-argument-do-not-emit'
        process = subprocess.run([sys.executable, str(ROOT / 'scripts/distribution/describe-backend.py'), '--unrecognized', sentinel], capture_output=True)
        self.assertEqual(process.returncode, 2)
        self.assertEqual(process.stdout, b'')
        self.assertEqual(process.stderr, b'WEFT-DISTRIBUTION-METADATA-CONFIG\n')
        self.assertNotIn(sentinel.encode(), process.stderr)

    def test_decoded_inventory_limit_refuses(self):
        with gzip.open(self.inventory, 'wb') as output:
            output.write(b' ' * (metadata.LIMIT + 1))
        self.refused()


if __name__ == '__main__':
    unittest.main()
