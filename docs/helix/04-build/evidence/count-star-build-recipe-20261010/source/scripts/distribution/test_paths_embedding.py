"""Actual inert harness I/O controls; no compiler/embedding qualification."""
import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('paths_embedding_io', Path(__file__).with_name('check-paths-embedding-python.py'))
io = importlib.util.module_from_spec(spec)
spec.loader.exec_module(io)

class PathsEmbeddingIOTests(unittest.TestCase):
    def test_exact_bound_and_one_more_refusal(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp).resolve() / 'input'
            p.write_bytes(b'abc')
            self.assertEqual(io.read(p, 3), b'abc')
            with self.assertRaisesRegex(ValueError, 'input-bound'):
                io.read(p, 2)

    def test_symlink_directory_fifo_refuse(self):
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp).resolve()
            original = root / 'original'
            original.write_bytes(b'valid')
            link = root / 'link'
            link.symlink_to(original)
            fifo = root / 'fifo'
            os.mkfifo(fifo)
            for p in (link, root, fifo):
                with self.assertRaisesRegex(ValueError, 'regular-input'):
                    io.read(p)

    def test_cancellation_primary_survives_close_failure(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp).resolve() / 'input'
            p.write_bytes(b'valid')
            original = io.os.close
            primary = KeyboardInterrupt()
            def close(fd):
                original(fd)
                raise OSError('close')
            with patch.object(io.os, 'read', side_effect=primary), patch.object(io.os, 'close', side_effect=close):
                with self.assertRaises(KeyboardInterrupt) as caught:
                    io.read(p)
            self.assertIs(caught.exception, primary)
            self.assertTrue(primary.cleanup_failed)

    def test_cleanup_cancellation_outranks_ordinary_body_error(self):
        with tempfile.TemporaryDirectory() as tmp:
            p = Path(tmp).resolve() / 'input'
            p.write_bytes(b'valid')
            original = io.os.close
            primary = KeyboardInterrupt()
            def close(fd):
                original(fd)
                raise primary
            with patch.object(io.os, 'read', side_effect=OSError('read')), patch.object(io.os, 'close', side_effect=close):
                with self.assertRaises(KeyboardInterrupt) as caught:
                    io.read(p)
            self.assertIs(caught.exception, primary)

if __name__ == '__main__':
    unittest.main()
