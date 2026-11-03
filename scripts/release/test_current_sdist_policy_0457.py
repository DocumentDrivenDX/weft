"""Closed source packaging policy controls; no Cargo, Maturin, API or builds."""
from pathlib import Path
import ast,copy,json,os,tarfile,tempfile,tomllib,unittest
import current_sdist_policy_0457 as policy
import prepare_current_sources_0457 as assembler
ROOT=Path(os.environ.get('WEFT_SOURCE_TEST_ROOT',Path(__file__).resolve().parents[2]))
ROOT_CONFIG=Path(os.environ.get('WEFT_ROOT_CONFIG_TEST_PATH',ROOT/'pyproject.toml'))
class RootConfig(unittest.TestCase):
 def setUp(self):
  self.nested=tomllib.loads((ROOT/'crates/weft-python/pyproject.toml').read_text());self.root=tomllib.loads(ROOT_CONFIG.read_text())
 def check(self):return policy.validate_sdist_configs(self.root,self.nested)
 def test_actual_root_config_preserves_nested_metadata_features(self):self.assertTrue(self.check()['metadataFeaturesMatchNested'])
 def test_root_discovery_uses_no_manifest_cli_override(self):self.assertEqual(policy.sdist_command(Path('/fresh/out')),['maturin','sdist','--out','/fresh/out'])
 def test_assembler_calls_root_discovery_and_same_selected_metadata_context(self):
  tree=ast.parse(Path(assembler.__file__).read_text());main=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='main');calls=[n for n in ast.walk(main) if isinstance(n,ast.Call) and isinstance(n.func,ast.Name) and n.func.id=='run']
  stage={n.args[-1].value:n for n in calls if isinstance(n.args[-1],ast.Constant)}
  self.assertEqual(ast.unparse(stage['maturin-sdist'].args[1]),'sdist_command(distribution)')
  for name in ['locked-metadata','archive-locked-metadata']:self.assertEqual(ast.unparse(stage[name].args[1]),'selected_metadata_command(features)')
  args=assembler.selected_metadata_command(','.join(self.nested['tool']['maturin']['features']));self.assertNotIn('--workspace',args);self.assertIn('--locked',args);self.assertEqual(args[args.index('--manifest-path')+1],'crates/weft-python/Cargo.toml');self.assertEqual(args[args.index('--filter-platform')+1],'x86_64-unknown-linux-gnu')
 def test_root_feature_drift_refuses(self):
  self.root['tool']['maturin']['features']=[]
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
 def test_root_version_drift_refuses(self):
  self.root['project']['version']='0.4.58'
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
 def test_root_license_path_drift_refuses(self):
  self.root['project']['license-files'][0]='LICENSE.md'
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
 def test_root_includes_are_sdist_only(self):
  self.root['tool']['maturin']['include'][0]['format']='wheel'
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
 def test_terminal_directory_only_globs_refuse(self):
  for item in self.root['tool']['maturin']['include']:item['path']=item['path'].removesuffix('/*')
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
 def test_missing_vendor_include_refuses(self):
  self.root['tool']['maturin']['include'].pop()
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
 def test_nested_parent_relative_discovery_is_not_root_profile(self):
  self.root=copy.deepcopy(self.nested)
  with self.assertRaisesRegex(ValueError,'parity'):self.check()
class TypedModes(unittest.TestCase):
 def test_only_four_regular_representations(self):
  for raw,git,expected in [(0o644,'100644',0o644),(0o100644,'100644',0o644),(0o755,'100755',0o755),(0o100755,'100755',0o755)]:self.assertEqual(policy.regular_mode_permissions(raw,b'0',git),expected)
 def test_mode_drift_special_bits_and_wrong_file_type_bits_refuse(self):
  for raw,git in [(0o664,'100644'),(0o100664,'100644'),(0o755,'100644'),(0o644,'100755'),(0o4755,'100755'),(0o104755,'100755'),(0o20644,'100644'),(0o40644,'100644'),(0o120644,'100644')]:
   with self.subTest(raw=oct(raw),git=git),self.assertRaises(ValueError):policy.regular_mode_permissions(raw,b'0',git)
 def test_tar_type_is_checked_independently(self):
  for kind in [b'1',b'2',b'3',b'4',b'5',b'6',b'7',b'S']:
   with self.subTest(kind=kind),self.assertRaisesRegex(ValueError,'Nonregular'):policy.regular_mode_permissions(0o644,kind,'100644')
 def test_generated_metadata_cannot_become_executable(self):
  with self.assertRaises(ValueError):policy.regular_mode_permissions(0o755,b'0',None)
 def test_contiguous_tar_type_cannot_hide_as_regular(self):
  with tempfile.TemporaryDirectory() as d:
   p=Path(d)/'special.tar.gz'
   with tarfile.open(p,'w:gz') as t:m=tarfile.TarInfo('root/file');m.type=b'7';m.mode=0o644;t.addfile(m)
   with self.assertRaisesRegex(ValueError,'Unsupported archive source type'):assembler.members(p)
class WorkspacePolicy(unittest.TestCase):
 def setUp(self):
  self.original={'workspace':{'members':['crates/weft-core','crates/weft-python'],'resolver':'2','dependencies':{'serde':'=1.0.228'}},'patch':{'crates-io':{'ahash':{'path':'vendor/ahash'}}}}
  self.metadata=dict(workspace_root='/source',packages=[dict(id='py',name='weft-python',manifest_path='/source/crates/weft-python/Cargo.toml',dependencies=[dict(name='weft-core',path='/source/crates/weft-core',rename=None),dict(name='ahash',path=None,rename=None)]),dict(id='core',name='weft-core',manifest_path='/source/crates/weft-core/Cargo.toml',dependencies=[]),dict(id='ahash',name='ahash',manifest_path='/source/vendor/ahash/Cargo.toml',dependencies=[])],resolve=dict(root='py',nodes=[dict(id='py',dependencies=['core','ahash']),dict(id='core',dependencies=[]),dict(id='ahash',dependencies=[])]))
 def test_independent_pinned_algorithm_keeps_python_only_and_all_other_fields(self):
  result=policy.expected_maturin196_workspace(self.original,self.metadata);expected=copy.deepcopy(self.original);expected['workspace']['members']=['crates/weft-python'];self.assertEqual(result,expected);self.assertEqual(self.original['workspace']['members'],['crates/weft-core','crates/weft-python'])
 def test_workspace_glob_refuses(self):
  self.original['workspace']['members'].append('crates/*')
  with self.assertRaisesRegex(ValueError,'glob'):policy.expected_maturin196_workspace(self.original,self.metadata)
 def test_external_dependency_path_refuses(self):
  self.metadata['packages'][0]['dependencies'][0]['path']='/other/core'
  with self.assertRaisesRegex(ValueError,'External'):policy.expected_maturin196_workspace(self.original,self.metadata)
 def test_wrong_python_root_refuses(self):
  self.metadata['packages'][0]['name']='other'
  with self.assertRaisesRegex(ValueError,'root differs'):policy.expected_maturin196_workspace(self.original,self.metadata)
class ArchiveMutation(unittest.TestCase):
 def test_checked_source_bytes_mode_and_extra_file_drift_are_visible(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);p=root/'Cargo.lock';p.write_bytes(b'locked');p.chmod(0o644);before=assembler.checked_source_snapshot(root)
   p.write_bytes(b'mutated');self.assertNotEqual(before,assembler.checked_source_snapshot(root));p.write_bytes(b'locked');p.chmod(0o755);self.assertNotEqual(before,assembler.checked_source_snapshot(root));p.chmod(0o644);(root/'extra').write_bytes(b'extra');self.assertNotEqual(before,assembler.checked_source_snapshot(root))
 def test_checked_source_symlink_refuses(self):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);(root/'link').symlink_to('/not-a-source')
   with self.assertRaisesRegex(ValueError,'unsafe'):assembler.checked_source_snapshot(root)
if __name__=='__main__':unittest.main()
