# SPDX-License-Identifier: MIT OR Apache-2.0
import hashlib
import tempfile
import unittest
from pathlib import Path

from licensed_source import PREFIX, SLASH_PREFIX, source_custody


class HistoricalSourceCustody(unittest.TestCase):
    def test_exact_source_and_one_explicit_annotation(self):
        original = b'print("historical control")\n'
        expected = hashlib.sha256(original).hexdigest()
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory)/'harness.py'
            for content, adjustment in [(original,'none'),(PREFIX+original,'one-leading-SPDX-annotation')]:
                path.write_bytes(content)
                receipt = source_custody(path,expected)
                self.assertEqual(receipt['adjustment'],adjustment)
                self.assertEqual(receipt['currentSha256'],hashlib.sha256(content).hexdigest())
                self.assertEqual(receipt['historicalSha256'],expected)
            for content in [PREFIX+original+b'\n',PREFIX+original.replace(b'control',b'changed'),PREFIX+PREFIX+original,b'# unrelated comment\n'+original]:
                path.write_bytes(content)
                with self.assertRaises(ValueError):
                    source_custody(path,expected)
            rust = Path(directory)/'module.rs';rust.write_bytes(SLASH_PREFIX+original)
            self.assertEqual(source_custody(rust,expected)['adjustment'],'one-leading-SPDX-annotation')
            artifact = Path(directory)/'receipt.json';artifact.write_bytes(PREFIX+original)
            with self.assertRaises(ValueError):
                source_custody(artifact,expected)


if __name__ == '__main__':
    unittest.main()
