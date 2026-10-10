"""Finite inert producer boundary controls; no compiler or engine execution."""
import gzip
import os
import importlib.util
from pathlib import Path
import struct
import sys
import tempfile
import unittest
from unittest.mock import patch

spec=importlib.util.spec_from_file_location('paths_assembly',Path(__file__).with_name('assemble-paths-distribution.py'))
a=importlib.util.module_from_spec(spec);sys.modules[spec.name]=a;spec.loader.exec_module(a)

class PathsAssemblyTests(unittest.TestCase):
    def test_compressed_expansion_and_exact_single_member(self):
        raw=b'x'*17;compressed=gzip.compress(raw,mtime=0)
        self.assertEqual(a.inflate(compressed,17),raw)
        for bad,limit in [(compressed,16),(compressed+compressed,100),(compressed+b'x',100),(compressed[:-1],100)]:
            with self.assertRaises(ValueError):a.inflate(bad,limit)

    def test_regular_file_cap_and_symlink(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory).resolve();p=root/'input';p.write_bytes(b'abcd')
            self.assertEqual(a.read(p,4),b'abcd')
            with self.assertRaises(ValueError):a.read(p,3)
            link=root/'link';link.symlink_to(p)
            with self.assertRaises(ValueError):a.read(link,4)

    def test_aggregate_precharge_precedes_next_read(self):
        with tempfile.TemporaryDirectory()as directory:
            p=Path(directory).resolve()/'input';p.write_bytes(b'valid')
            s=a.Snapshot();s.total=a.TOTAL_LIMIT
            with patch.object(a,'read',side_effect=AssertionError('must not allocate')):
                with self.assertRaisesRegex(ValueError,'package-bound'):s.take(p,'next')
            self.assertEqual(s.artifacts,{})

    def test_closed_json_and_giant_numeric_atom(self):
        for raw in [b'{"x":1,"x":2}',b'1.0',b'NaN',b'1'*1000,b'"\xff"']:
            with self.assertRaises((ValueError,UnicodeError)):a.document(raw)
        self.assertEqual(a.document(b'{"n":1842,"lexeme":"0002"}'),{'n':1842,'lexeme':'0002'})

    def test_original_profiles_and_namespace_fence_are_separate(self):
        self.assertEqual(a.SOURCE,'530ae3511a4a50364d3d7e26195d3883952601df')
        fence=a.encoded(a.FENCE)+b'\n'
        self.assertEqual(a.document(fence)['interfaceVersion'],'weft-compile/0.4.0')
        for raw in [b'{}',b'{"sourceCommit":"old"}']:
            with self.assertRaises((KeyError,ValueError)):a.verify_records(a.document(raw),{}, {})
        self.assertEqual(a.FILE_LIMIT,32*1024*1024)
        self.assertEqual(a.TOTAL_LIMIT,96*1024*1024)
        self.assertEqual(a.DECODED_LIMIT,64*1024*1024)

    def test_macho_platform_requires_real_selected_structure(self):
        command=struct.pack('<6I',0x32,24,1,11<<16,27<<16,0)
        binary=struct.pack('<8I',0xfeedfacf,0x0100000c,0,2,1,len(command),0,0)+command
        self.assertEqual(a.platform(binary,'27.0.1')['machine'],'arm64')
        for bad,os_version in [(binary,'27.0.2'),(binary[:16],'27.0.1'),(binary[:4]+struct.pack('<I',7)+binary[8:],'27.0.1')]:
            with self.assertRaises(ValueError):a.platform(bad,os_version)

    def test_fresh_output_and_public_index_remain_unchanged(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory).resolve();index=root/'index.json';index.write_bytes(b'trusted index bytes')
            existing=root/'candidate';existing.mkdir();sentinel=existing/'sentinel';sentinel.write_bytes(b'original')
            s=a.Snapshot();s.generated('manifest.json',b'candidate')
            with self.assertRaisesRegex(ValueError,'output-appeared'):a.publish(s,existing)
            self.assertEqual(sentinel.read_bytes(),b'original');self.assertEqual(index.read_bytes(),b'trusted index bytes')
            output=root/'new';a.publish(s,output)
            self.assertEqual((output/'manifest.json').read_bytes(),b'candidate')
            self.assertEqual(index.read_bytes(),b'trusted index bytes')
            self.assertFalse(list(root.glob('.paths-distribution-*')))

    def test_file_replaced_with_fifo_before_open_refuses_without_blocking(self):
        with tempfile.TemporaryDirectory()as directory:
            p=Path(directory).resolve()/'input';p.write_bytes(b'old');original=os.open
            def changed(path,flags):
                p.unlink();os.mkfifo(p)
                return original(path,flags)
            with patch.object(a.os,'open',side_effect=changed):
                with self.assertRaisesRegex(ValueError,'regular-contained-input'):a.read(p)

    def test_verified_extractor_snapshot_is_executed_without_reopen(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory).resolve();p=root/'extractor.py'
            raw=b"def encoded(v): return b'{}'\ndef extract_legacy(source,maximum): return ([{'id':'one','request':{},'response':{}}]*463,[])\n"
            p.write_bytes(raw);original=a.read
            def changed(path,*args):
                result=original(path,*args);p.write_bytes(b"raise AssertionError('replacement executed')\n");return result
            row={'id':'legacy:one','requestHex':b'{}'.hex(),'originalExpectedHex':b'{}\n'.hex()}
            with patch.object(a,'read',side_effect=changed):a.verify_legacy(root,p,[row]*463,a.sha(raw))

    def test_copy_cancellation_survives_cleanup_failure(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory).resolve();s=a.Snapshot();s.generated('manifest.json',b'candidate');primary=KeyboardInterrupt()
            with patch.object(a.os,'fsync',side_effect=primary),patch.object(a.shutil,'rmtree',side_effect=OSError('cleanup')):
                with self.assertRaises(KeyboardInterrupt)as caught:a.publish(s,root/'output')
            self.assertIs(caught.exception,primary);self.assertTrue(primary.cleanup_failed)
            self.assertFalse((root/'output').exists())

    def test_read_cancellation_survives_descriptor_close_failure(self):
        with tempfile.TemporaryDirectory()as directory:
            p=Path(directory).resolve()/'input';p.write_bytes(b'original');primary=KeyboardInterrupt();original=os.close
            def close(fd):original(fd);raise OSError('cleanup')
            with patch.object(a.os,'read',side_effect=primary),patch.object(a.os,'close',side_effect=close):
                with self.assertRaises(KeyboardInterrupt)as caught:a.read(p)
            self.assertIs(caught.exception,primary);self.assertTrue(primary.cleanup_failed)

    def test_closing_input_drift_withholds_package(self):
        with tempfile.TemporaryDirectory()as directory:
            root=Path(directory).resolve();p=root/'input';p.write_bytes(b'old');s=a.Snapshot();s.take(p,'input');p.write_bytes(b'new')
            with self.assertRaisesRegex(ValueError,'closing-input-drift'):a.publish(s,root/'output')
            self.assertFalse((root/'output').exists())

if __name__=='__main__':unittest.main()
