"""Source-only wiring/admission tests; no remote calls, builds or native imports."""
import copy,hashlib,importlib.util,json,os,subprocess,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import verify_current_tools_0457 as tools
import prepare_current_sources_0457 as archives
R='a'*40
class ToolCoupling(unittest.TestCase):
 def setUp(self):
  self.t=tempfile.TemporaryDirectory();self.addCleanup(self.t.cleanup);self.root=Path(self.t.name)
  (self.root/'Cargo.lock').write_text('[[package]]\nname="wasm-bindgen"\nversion="0.2.105"\n')
  (self.root/'package.json').write_text('{"devDependencies":{"playwright":"1.62.1"}}')
  self.versions={'rustc':'rustc 1.90.0 (test)','cargo':'cargo 1.90.0 (test)','maturin':'maturin 1.9.6','wasm-bindgen':'wasm-bindgen 0.2.105','bun':'1.4.2','node':'v24.1.0'}
 def run_tools(self):
  def fake(args,**kwargs):return subprocess.CompletedProcess(args,0,self.versions[args[0]]+'\n','')
  with patch.object(tools.subprocess,'run',side_effect=fake):return tools.inspect(self.root,self.root/'proof',wasm=True)
 def test_exact_cli_matches_locked_crate(self):self.assertIn('bindgen',self.run_tools())
 def test_cli_mismatch_fails_before_build(self):
  self.versions['wasm-bindgen']='wasm-bindgen 0.2.104'
  with self.assertRaisesRegex(ValueError,'CLI/locked'):self.run_tools()
 def test_lock_mismatch_fails_before_build(self):
  (self.root/'Cargo.lock').write_text('[[package]]\nname="wasm-bindgen"\nversion="0.2.104"\n')
  with self.assertRaisesRegex(ValueError,'Locked'):self.run_tools()
 def test_other_pinned_tool_mismatch(self):
  self.versions['maturin']='maturin 1.9.5'
  with self.assertRaisesRegex(ValueError,'Maturin'):self.run_tools()
class SourceBytePolicy(unittest.TestCase):
 def test_archive_path_traversal_refuses(self):
  import tarfile,io
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'bad.tar.gz'
   with tarfile.open(p,'w:gz') as z:
    m=tarfile.TarInfo('root/../outside.rs');m.size=1;z.addfile(m,io.BytesIO(b'x'))
   with self.assertRaisesRegex(ValueError,'Unsafe'):archives.members(p)
 def test_archive_duplicate_refuses(self):
  import tarfile,io
  with tempfile.TemporaryDirectory() as t:
   p=Path(t)/'bad.tar.gz'
   with tarfile.open(p,'w:gz') as z:
    for i in range(2):m=tarfile.TarInfo('root/Cargo.lock');m.size=1;z.addfile(m,io.BytesIO(b'x'))
   with self.assertRaisesRegex(ValueError,'Duplicate'):archives.members(p)

class ActualArchiveCorrespondence(unittest.TestCase):
 def setUp(self):
  import tarfile,io
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name).resolve()
  self.files={
   'Cargo.toml':b'[workspace]\nmembers=["crates/weft-core","crates/weft-databricks","crates/weft-postgresql","crates/weft-python","crates/weft-runtime","crates/weft-wasm"]\n[workspace.package]\nversion="0.1.0"\n[patch.crates-io]\nahash={path="vendor/ahash"}\n',
   'Cargo.lock':b'FROZEN-LOCK-BYTE-FIXTURE',
   'crates/weft-python/pyproject.toml':b'[project]\nname="umf-weft"\nversion="0.4.57"\nreadme="README.md"\nlicense-files=["LICENSE.md","LICENSE-MIT","LICENSE-APACHE","THIRD-PARTY-LICENSES.txt"]\n[tool.maturin]\nbindings="pyo3"\ninclude=[{path="../../tests/**",format="sdist"},{path="../../spec/**",format="sdist"},{path="../../.cargo/**",format="sdist"}]\n',
   'crates/weft-python/Cargo.toml':b'[package]\nname="weft-python"\nversion="0.4.57"\n',
   'crates/weft-core/Cargo.toml':b'[package]\nname="weft-core"\nversion="0.1.0"\n',
   'crates/weft-core/src/lib.rs':b'// synthetic byte fixture, never compiled\n',
   'crates/weft-core/resources/core.json':b'{"opaque":"retained"}',
   'unrelated.rs':b'// tracked but outside declared native sdist policy\n',
  }
  for name in ['README.md','LICENSE.md','LICENSE-MIT','LICENSE-APACHE','THIRD-PARTY-LICENSES.txt']:
   self.files['crates/weft-python/'+name]=('synthetic '+name+'\n').encode()
  self.files.update({'tests/run-roster.py':b'# synthetic executable, NEVER executed\n',
    'spec/required.json':b'{"required":"spec"}', '.cargo/config.toml':b'[net]\noffline=true\n',
    'crates/weft-core/docs/required.md':b'# Cargo-selected documentation\n',
    'crates/weft-core/developer-only.txt':b'not selected by synthetic Cargo policy\n'})
  for native,name in [('crates/weft-databricks','weft-databricks'),('crates/weft-postgresql','weft-postgresql'),('crates/weft-runtime','weft-runtime')]:
   self.files[native+'/Cargo.toml']=('[package]\nname="'+name+'"\nversion.workspace=true\n').encode()
   self.files[native+'/src/lib.rs']=b'// synthetic source, never compiled\n'
  self.files.update({'vendor/ahash/Cargo.toml':b'[package]\nname="ahash"\nversion="0.8.12"\nbuild="build.rs"\n',
    'vendor/ahash/build.rs':b'// synthetic build script, NEVER executed\n',
    'vendor/ahash/src/lib.rs':b'// synthetic vendored library, never compiled\n',
    'vendor/ahash/README.md':b'ahash retained readme\n','vendor/ahash/LICENSE-MIT':b'synthetic MIT notice\n',
    'vendor/ahash/LICENSE-APACHE':b'synthetic Apache notice\n','vendor/ahash/Cargo.toml.orig':b'original upstream manifest, filtered by Maturin\n',
    'tests/dev-probe/Cargo.toml':b'[package]\nname="dev-probe"\nversion="0.0.0"\n'})
  self.source_modes={'tests/run-roster.py':0o755};self.tar_modes={}
  for n,b in self.files.items():
   p=self.root/n;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(b);p.chmod(self.source_modes.get(n,0o644))
  self.package_lists={native:'\n'.join(name[len(native)+1:] for name in self.files if name.startswith(native+'/') and not name.endswith(('developer-only.txt','Cargo.toml.orig')))+'\nCargo.toml.orig\n.cargo_vcs_info.json\n' for native in archives.REVIEWED_LOCAL_ROOTS}

  subprocess.run(['git','init','-q'],cwd=self.root,check=True)
  subprocess.run(['git','add','.'],cwd=self.root,check=True)
  subprocess.run(['git','-c','user.name=Archive Unit','-c','user.email=unit@example.invalid','-c','commit.gpgsign=false','commit','-qm','synthetic source-byte fixture'],cwd=self.root,check=True)
  self.R=subprocess.check_output(['git','rev-parse','HEAD'],cwd=self.root,text=True).strip()
  self.archive=dict(self.files);self.archive.pop('unrelated.rs');self.archive.pop('crates/weft-python/pyproject.toml');self.archive.pop('crates/weft-core/developer-only.txt');self.archive.pop('vendor/ahash/Cargo.toml.orig')
  for name in ['README.md','LICENSE.md','LICENSE-MIT','LICENSE-APACHE','THIRD-PARTY-LICENSES.txt']:self.archive[name]=self.files['crates/weft-python/'+name]
  self.archive['Cargo.toml']=self.files['Cargo.toml'].replace(b',"crates/weft-wasm"',b'')
  self.archive['crates/weft-python/Cargo.toml']+=b'readme="README.md"\n'
  self.archive['pyproject.toml']=self.files['crates/weft-python/pyproject.toml']+b'manifest-path="crates/weft-python/Cargo.toml"\n'
  self.archive['PKG-INFO']=b'Metadata-Version: 2.4\nName: umf-weft\nVersion: 0.4.57\nRequires-Python: >=3.9\nLicense-Expression: MIT OR Apache-2.0\n\n'
  self.metadata=self.graph_fixture()
 def graph_fixture(self):
  packages=[];nodes=[];ids={}
  for native,(name,version) in archives.REVIEWED_LOCAL_ROOTS.items():
   key='path+'+(self.root/native).as_uri()+'#'+version;ids[native]=key
   packages.append(dict(id=key,name=name,version=version,source=None,manifest_path=str(self.root/native/'Cargo.toml')))
  registry='registry+synthetic-source#dependency@1.0.0'
  packages.append(dict(id=registry,name='synthetic-transitive-dependency',version='1.0.0',source='registry+synthetic-source',manifest_path='/not-a-local-source/Cargo.toml'))
  probe='path+'+(self.root/'tests/dev-probe').as_uri()+'#dev-probe@0.0.0'
  packages.append(dict(id=probe,name='dev-probe',version='0.0.0',source=None,manifest_path=str(self.root/'tests/dev-probe/Cargo.toml')))
  edges={'crates/weft-python':['crates/weft-runtime'],'crates/weft-runtime':['crates/weft-core','crates/weft-databricks','crates/weft-postgresql'],
    'crates/weft-databricks':['crates/weft-core'],'crates/weft-postgresql':['crates/weft-core'],'crates/weft-core':[],'vendor/ahash':[]}
  for native,key in ids.items():
   deps=[dict(pkg=ids[target],dep_kinds=[dict(kind=None,target=None)]) for target in edges[native]]
   if native=='crates/weft-core':deps.append(dict(pkg=registry,dep_kinds=[dict(kind=None,target=None)]))
   if native=='crates/weft-python':deps.append(dict(pkg=probe,dep_kinds=[dict(kind='dev',target=None)]))
   nodes.append(dict(id=key,deps=deps))
  nodes.append(dict(id=registry,deps=[dict(pkg=ids['vendor/ahash'],dep_kinds=[dict(kind='build',target='cfg(target_os = "linux")')])]))
  nodes.append(dict(id=probe,deps=[]))
  return dict(workspace_root=str(self.root),packages=packages,resolve=dict(root=ids['crates/weft-python'],nodes=nodes))
 def verify_archive(self):
  import tarfile,io
  path=self.root/'candidate.tar.gz'
  with tarfile.open(path,'w:gz') as t:
   for name,raw in self.archive.items():m=tarfile.TarInfo('umf_weft-0.4.57/'+name);m.size=len(raw);m.mode=self.tar_modes.get(name,self.source_modes.get(name,0o644));t.addfile(m,io.BytesIO(raw))
  check_output=subprocess.check_output
  def fake(args,**kwargs):
   if args[0]=='cargo':
    if getattr(self,'package_mutation',None):
     change=self.package_mutation;self.package_mutation=None;change()
    self.assertEqual(args[:5],['cargo','package','--list','--allow-dirty','--manifest-path'])
    return self.package_lists[args[-1].removesuffix('/Cargo.toml')].encode()
   return check_output(args,**kwargs)
  with patch.object(archives.subprocess,'check_output',side_effect=fake):
   return archives.verify_sdist(self.root,self.R,path,self.metadata)
 def test_positive_actual_archive_bytes_and_declared_metadata(self):self.assertTrue(self.verify_archive()['allOtherBytesMatchR'])
 def test_complete_declared_roster_is_recorded(self):
  proof=self.verify_archive();self.assertTrue(proof['completeDeclaredInventory']);self.assertTrue(proof['mappedGitModesMatchR'])
  self.assertEqual(set(proof['declaredInventory']),set(self.archive))
  self.assertEqual(proof['declaredInventory']['README.md']['source'],'crates/weft-python/README.md')
 def test_missing_required_generated_metadata(self):
  self.archive.pop('PKG-INFO')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_required_root_pyproject(self):
  self.archive.pop('pyproject.toml')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_required_readme(self):
  self.archive.pop('README.md')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_required_notices(self):
  self.archive.pop('THIRD-PARTY-LICENSES.txt')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_declared_tests(self):
  self.archive.pop('tests/run-roster.py')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_declared_spec(self):
  self.archive.pop('spec/required.json')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_declared_cargo_configuration(self):
  self.archive.pop('.cargo/config.toml')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_missing_cargo_selected_non_native_entry(self):
  self.archive.pop('crates/weft-core/docs/required.md')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_executable_mode_drift_refuses(self):
  self.tar_modes['tests/run-roster.py']=0o644
  with self.assertRaisesRegex(ValueError,'mode differs'):self.verify_archive()
 def test_unexpected_executable_mode_refuses(self):
  self.tar_modes['spec/required.json']=0o755
  with self.assertRaisesRegex(ValueError,'mode differs'):self.verify_archive()
 def test_generated_metadata_mode_drift_refuses(self):
  self.tar_modes['PKG-INFO']=0o755
  with self.assertRaisesRegex(ValueError,'mode differs'):self.verify_archive()
 def test_cargo_excluded_tracked_extra_refuses(self):
  self.archive['crates/weft-core/developer-only.txt']=self.files['crates/weft-core/developer-only.txt']
  with self.assertRaisesRegex(ValueError,'extra source'):self.verify_archive()
 def test_package_inventory_cannot_mutate_frozen_source_or_lock(self):
  changed=b'UNAPPROVED LOCK MUTATION'
  self.package_mutation=lambda:(self.root/'Cargo.lock').write_bytes(changed)
  with self.assertRaisesRegex(ValueError,'source bytes differ|mutation'):self.verify_archive()
  self.assertEqual((self.root/'Cargo.lock').read_bytes(),changed) # no silent restore
 def test_package_inventory_cannot_create_untracked_source(self):
  self.package_mutation=lambda:(self.root/'unexpected-source').write_text('unexpected')
  with self.assertRaisesRegex(ValueError,'mutation'):self.verify_archive()
  self.assertTrue((self.root/'unexpected-source').exists())
 def test_root_pyproject_transform_must_remain_explicit(self):
  self.archive['pyproject.toml']+=b'new-unreviewed-option=true\n'
  with self.assertRaisesRegex(ValueError,'manifest location'):self.verify_archive()
 def test_workspace_transform_cannot_change_dependency_pins(self):
  self.archive['Cargo.toml']+=b'[workspace.dependencies]\nwrong="1"\n'
  with self.assertRaisesRegex(ValueError,'workspace/pins'):self.verify_archive()
 def test_python_readme_transform_cannot_change_other_fields(self):
  self.archive['crates/weft-python/Cargo.toml']+=b'new-unreviewed-option=true\n'
  with self.assertRaisesRegex(ValueError,'readme metadata'):self.verify_archive()
 def metadata_package(self,name):return next(p for p in self.metadata['packages'] if p['name']==name)
 def test_genuine_selected_six_root_fixture_includes_exact_ahash(self):
  proof=self.verify_archive()
  self.assertEqual(proof['selectedLocalRoots'],sorted(archives.REVIEWED_LOCAL_ROOTS))
  self.assertIn('vendor/ahash/build.rs',proof['declaredInventory'])
  self.assertIn('vendor/ahash/LICENSE-APACHE',proof['declaredInventory'])
  self.assertIn('vendor/ahash',proof['cargoPackageLists'])
 def test_other_vendor_root_is_not_generically_admitted(self):
  self.metadata_package('ahash')['manifest_path']=str(self.root/'vendor/other/Cargo.toml')
  with self.assertRaisesRegex(ValueError,'Unreviewed selected local root'):self.verify_archive()
 def test_selected_graph_missing_ahash_refuses(self):
  registry=next(n for n in self.metadata['resolve']['nodes'] if n['id'].startswith('registry+'))
  registry['deps']=[]
  with self.assertRaisesRegex(ValueError,'root roster missing/extra'):self.verify_archive()
 def test_selected_graph_extra_dev_probe_normal_edge_refuses(self):
  root=next(n for n in self.metadata['resolve']['nodes'] if n['id']==self.metadata['resolve']['root'])
  root['deps'][-1]['dep_kinds']=[dict(kind=None,target=None)]
  with self.assertRaisesRegex(ValueError,'Unreviewed selected local root'):self.verify_archive()
 def test_external_local_manifest_refuses(self):
  self.metadata_package('ahash')['manifest_path']='/external/vendor/ahash/Cargo.toml'
  with self.assertRaisesRegex(ValueError,'External/noncanonical'):self.verify_archive()
 def test_symlink_local_root_refuses(self):
  source=self.root/'vendor/ahash';destination=self.root/'vendor/retained-ahash-fixture'
  source.rename(destination);source.symlink_to(destination,target_is_directory=True)
  with self.assertRaisesRegex(ValueError,'Symlink selected local root'):self.verify_archive()
 def test_substituted_local_package_version_refuses(self):
  self.metadata_package('ahash')['version']='0.8.11'
  with self.assertRaisesRegex(ValueError,'package identity differs'):self.verify_archive()
 def test_substituted_local_cargo_source_id_refuses(self):
  package=self.metadata_package('ahash');old=package['id'];package['id']=old+'-substituted'
  for node in self.metadata['resolve']['nodes']:
   if node['id']==old:node['id']=package['id']
   for dep in node['deps']:
    if dep['pkg']==old:dep['pkg']=package['id']
  with self.assertRaisesRegex(ValueError,'Cargo local source ID'):self.verify_archive()
 def test_ahash_manifest_substituted_outside_frozen_R_refuses(self):
  path=self.root/'vendor/ahash/Cargo.toml';path.write_bytes(path.read_bytes()+b'# unreviewed change\n')
  with self.assertRaisesRegex(ValueError,'manifest bytes differ'):self.verify_archive()
 def test_omitted_ahash_build_script_refuses(self):
  self.archive.pop('vendor/ahash/build.rs')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_changed_ahash_build_script_refuses(self):
  self.archive['vendor/ahash/build.rs']+=b'// unreviewed script\n'
  with self.assertRaisesRegex(ValueError,'bytes differ'):self.verify_archive()
 def test_missing_ahash_notice_refuses(self):
  self.archive.pop('vendor/ahash/LICENSE-MIT')
  with self.assertRaisesRegex(ValueError,'misses required'):self.verify_archive()
 def test_changed_ahash_notice_refuses(self):
  self.archive['vendor/ahash/LICENSE-APACHE']+=b'changed notice'
  with self.assertRaisesRegex(ValueError,'bytes differ'):self.verify_archive()
 def test_ahash_mode_drift_refuses(self):
  self.tar_modes['vendor/ahash/build.rs']=0o755
  with self.assertRaisesRegex(ValueError,'mode differs'):self.verify_archive()
 def test_undeclared_ahash_file_refuses(self):
  self.archive['vendor/ahash/undeclared.rs']=b'// not source-selected\n'
  with self.assertRaisesRegex(ValueError,'extra source'):self.verify_archive()
 def test_archive_lock_mismatch(self):
  self.archive['Cargo.lock']+=b' '
  with self.assertRaisesRegex(ValueError,'lock'):self.verify_archive()
 def test_archive_dependency_manifest_mutation(self):
  self.archive['crates/weft-core/Cargo.toml']+=b'# normalization is not permission\n'
  with self.assertRaisesRegex(ValueError,'bytes differ'):self.verify_archive()
 def test_archive_missing_native_resource(self):
  self.archive.pop('crates/weft-core/resources/core.json')
  with self.assertRaisesRegex(ValueError,'misses'):self.verify_archive()
 def test_archive_untracked_extra_source(self):
  self.archive['replacement.rs']=b'// not R'
  with self.assertRaisesRegex(ValueError,'extra source'):self.verify_archive()
 def test_archive_tracked_but_undeclared_extra_source(self):
  self.archive['unrelated.rs']=self.files['unrelated.rs']
  with self.assertRaisesRegex(ValueError,'inclusion policy'):self.verify_archive()
 def test_archive_wrong_metadata_version(self):
  self.archive['PKG-INFO']=self.archive['PKG-INFO'].replace(b'0.4.57',b'0.4.56')
  with self.assertRaisesRegex(ValueError,'metadata identity'):self.verify_archive()

if __name__=='__main__':unittest.main()
