"""@covers US-008-AC1 @covers US-008-AC7 Full input/history stale controls."""
import hashlib,json,pathlib,shutil,tempfile,unittest
from unittest.mock import patch
from reliability import custody
ROOT=pathlib.Path(__file__).resolve().parents[2]
class CustodyTests(unittest.TestCase):
 def test_actual_current_closure_and_historical_checkpoint(self):
  manifest=custody.snapshot(ROOT);self.assertGreater(custody.verify(ROOT,manifest),1000);self.assertEqual(custody.historical(ROOT)['sourceHashesVerified'],111)
 def test_changed_deleted_new_inputs_refuse(self):
  for mode in ('changed','deleted','added'):
   with tempfile.TemporaryDirectory() as work:
    root=pathlib.Path(work);(root/'src').mkdir();(root/'Cargo.lock').write_text('pin');(root/'src/a.rs').write_text('source')
    with patch.object(custody,'ROOTS',('src',)),patch.object(custody,'FILES',('Cargo.lock',)):
     manifest=custody.snapshot(root)
     if mode=='changed':(root/'src/a.rs').write_text('changed')
     elif mode=='deleted':(root/'src/a.rs').unlink()
     else:(root/'src/new.rs').write_text('new')
     with self.assertRaises(custody.CustodyError):custody.verify(root,manifest)
 def test_unowned_symlink_and_excluded_embedded_input_refuse(self):
  with tempfile.TemporaryDirectory() as work:
   root=pathlib.Path(work);(root/'src').mkdir();(root/'Cargo.lock').write_text('pin');(root/'src/a.rs').write_text('source');(root/'excluded.json').write_text('hidden')
   with patch.object(custody,'ROOTS',('src',)),patch.object(custody,'FILES',('Cargo.lock',)):
    (root/'src/link.rs').symlink_to(root/'src/a.rs')
    with self.assertRaises(custody.CustodyError):custody.snapshot(root)
    (root/'src/link.rs').unlink();(root/'src/a.rs').write_text('include_str!("../excluded.json")')
    with self.assertRaises(custody.CustodyError):custody.snapshot(root)
 def test_missing_git_checkpoint_and_wrong_history_refuse(self):
  failed=type('Failure',(),{'returncode':1,'stdout':b''})()
  with patch.object(custody.subprocess,'run',return_value=failed):
   with self.assertRaises(custody.CustodyError):custody.historical(ROOT)
  wrong=type('Wrong',(),{'returncode':0,'stdout':b'wrong historical bytes'})()
  with patch.object(custody.subprocess,'run',return_value=wrong):
   with self.assertRaises(custody.CustodyError):custody.historical(ROOT)
 def test_nested_build_named_sources_are_inputs(self):
  for directory in ('dist','target','node_modules','__pycache__'):
   with tempfile.TemporaryDirectory() as work:
    root=pathlib.Path(work);(root/'src').mkdir();(root/'Cargo.lock').write_text('pin')
    with patch.object(custody,'ROOTS',('src',)),patch.object(custody,'FILES',('Cargo.lock',)):
     manifest=custody.snapshot(root);path=root/'src'/directory/'meaning.rs';path.parent.mkdir();path.write_text('pub struct Meaning;')
     with self.assertRaises(custody.CustodyError):custody.verify(root,manifest)
 def test_raw_computed_include_and_schema_inputs_cannot_escape(self):
  sources=['include_str!(r#"../outside.json"#)','include_str!(concat!("../","outside.json"))','include!("../outside.rs")','#[jsonschema::validator(path="outside.json")] struct S;']
  for source in sources:
   with tempfile.TemporaryDirectory() as work:
    root=pathlib.Path(work);(root/'src').mkdir();(root/'Cargo.toml').write_text('[package]\nname="control"\nversion="0.1.0"\n');(root/'src/a.rs').write_text(source);(root/'outside.json').write_text('unowned');(root/'outside.rs').write_text('unowned')
    with patch.object(custody,'ROOTS',('src',)),patch.object(custody,'FILES',('Cargo.toml',)):
     with self.assertRaises(custody.CustodyError):custody.snapshot(root)
 def test_supported_raw_include_is_owned_and_comments_are_not_inputs(self):
  with tempfile.TemporaryDirectory() as work:
   root=pathlib.Path(work);(root/'src').mkdir();(root/'Cargo.lock').write_text('pin');(root/'src/data.json').write_text('owned');(root/'src/a.rs').write_text('/* nested /* include!("unowned") */ comment */ include_str!(r#"data.json"#); // include!("unowned")')
   with patch.object(custody,'ROOTS',('src',)),patch.object(custody,'FILES',('Cargo.lock',)):self.assertEqual(custody.verify(root,custody.snapshot(root)),3)
 def test_deleted_substituted_historical_manifest_entries_refuse(self):
  for mode in ('deleted','substituted'):
   with tempfile.TemporaryDirectory() as work:
    root=pathlib.Path(work);path=root/'docs/helix/04-build/evidence/B-007-workspace-qualified-final/sources.json';path.parent.mkdir(parents=True);rows=json.loads((ROOT/path.relative_to(root)).read_bytes());name=next(iter(rows))
    if mode=='deleted':del rows[name]
    else:rows[name]='0'*64
    path.write_text(json.dumps(rows))
    with self.assertRaises(custody.CustodyError):custody.historical(root)
 def test_new_root_inputs_and_excluded_cargo_targets_refuse(self):
  with tempfile.TemporaryDirectory() as work:
   root=pathlib.Path(work);(root/'Cargo.toml').write_text('[package]\nname="control"\nversion="0.1.0"\n');manifest=custody.snapshot(root);(root/'new-build-input.js').write_text('new')
   with self.assertRaises(custody.CustodyError):custody.verify(root,manifest)
   (root/'new-build-input.js').unlink();(root/'dist').mkdir();(root/'dist/lib.rs').write_text('pub struct Hidden;');(root/'Cargo.toml').write_text('[lib]\npath="dist/lib.rs"\n')
   with self.assertRaises(custody.CustodyError):custody.snapshot(root)
 def test_nested_schema_cargo_base_decoy_cannot_bypass(self):
  with tempfile.TemporaryDirectory() as work:
   base=pathlib.Path(work);root=base/'repo';source=root/'crates/c/src';source.mkdir(parents=True);(source.parent/'Cargo.toml').write_text('[package]\nname="c"\nversion="0.1.0"\n');(root/'outside.json').write_text('owned decoy');(base/'outside.json').write_text('unowned actual');(source/'lib.rs').write_text('#[cfg_attr(all(), jsonschema::validator(path="../../../outside.json"))] struct S;')
   with self.assertRaises(custody.CustodyError):custody.snapshot(root)
 def test_unknown_alias_path_attribute_refuses(self):
  with tempfile.TemporaryDirectory() as work:
   base=pathlib.Path(work);root=base/'repo';source=root/'crates/c/src';source.mkdir(parents=True);(source.parent/'Cargo.toml').write_text('[package]\nname="c"\nversion="0.1.0"\n');(root/'outside.json').write_text('owned decoy');(base/'outside.json').write_text('unowned actual');(source/'lib.rs').write_text('use jsonschema::validator as schema; #[schema(path="../../../outside.json")] struct S;')
   with self.assertRaises(custody.CustodyError):custody.snapshot(root)
if __name__=='__main__':unittest.main()
