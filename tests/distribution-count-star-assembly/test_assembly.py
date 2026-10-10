"""Small profile selection, error lifecycle and retained-byte proof controls."""
import copy,gzip,importlib.util,json,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
ROOT=Path(__file__).resolve().parents[2]
spec=importlib.util.spec_from_file_location('count_assembler',ROOT/'scripts/distribution/assemble-paths-distribution.py')
codec=importlib.util.module_from_spec(spec);sys.modules[spec.name]=codec;spec.loader.exec_module(codec)
spec=importlib.util.spec_from_file_location('count_profile',ROOT/'scripts/distribution/count_star_distribution.py')
m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m)
class Controls(unittest.TestCase):
    def test_explicit_selection_has_no_default_change(self):
        args=(Path('/s'),Path('/b'),Path('/c'),Path('/o'),'candidate')
        self.assertEqual(codec.Config(*args).profile,'paths')
        for name in ('paths','paths-keys','paths-keys-count-star'):
            self.assertEqual(codec.Config(*args,profile=name).profile,name)
        with self.assertRaises(ValueError):codec.Config(*args,profile='count-star')
    def test_cancellation_priority_preserves_identity(self):
        for cancellation in (KeyboardInterrupt(),SystemExit(),GeneratorExit()):
            self.assertIs(m.cleanup_primary(ValueError(),cancellation),cancellation)
            body=KeyboardInterrupt();self.assertIs(m.cleanup_primary(body,cancellation),body)
    def test_read_closing_cancellation_outranks_nonregular_error(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as directory:
            closing=KeyboardInterrupt()
            original_close=m.os.close
            def close(fd):
                original_close(fd);raise closing
            with patch.object(m.os,'close',side_effect=close):
                with self.assertRaises(KeyboardInterrupt) as caught:m.read(Path(directory))
            self.assertIs(caught.exception,closing)
    def test_publish_failed_copy_never_creates_available_output(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as directory:
            s=m.Snapshot(codec);s.generated('manifest.json',b'{}')
            output=Path(directory)/'candidate'
            with patch.object(m,'write_exclusive',side_effect=OSError('disk-full')):
                with self.assertRaises(OSError):m.publish(s,output)
            self.assertFalse(output.exists());self.assertEqual(list(Path(directory).iterdir()),[])
    def test_file_pin_refusal_does_not_capture_artifact(self):
        with tempfile.TemporaryDirectory(dir=Path(tempfile.gettempdir()).resolve()) as directory:
            p=Path(directory)/'file';p.write_bytes(b'changed')
            s=m.Snapshot(codec)
            with self.assertRaises(ValueError):s.take(p,'artifact','0'*64,7)
            self.assertEqual(s.artifacts,{});self.assertEqual(s.originals,{})
if __name__=='__main__':unittest.main()
