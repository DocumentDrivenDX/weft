"""Pure preflight inventory controls; no Cargo/network/native/build execution."""
from pathlib import Path
from unittest.mock import patch
import copy,json,os,unittest
import prepare_current_sources_0457 as source
ROOT=Path(os.environ.get('WEFT_SOURCE_TEST_ROOT',Path(__file__).resolve().parents[2]))
R='a'*40
class CanonicalSourceInventory(unittest.TestCase):
 def setUp(self):
  self.expected=json.loads((ROOT/'scripts/release/current-inputs-0457.json').read_text())
  git=[dict(name=p['name'],version=p['version'],source=p['sourceID'],id=p['sourceID'].split('#')[0]+'#'+p['name']+'@'+p['version']) for p in self.expected['lockGitPackages']]
  self.metadata=dict(packages=git+[dict(name='weft-python',version='0.4.57',source=None,id='python-root')],resolve=dict(root='python-root',nodes=[dict(id='python-root',deps=[dict(pkg=p['id'],dep_kinds=[dict(kind=None)]) for p in git])]+[dict(id=p['id'],deps=[]) for p in git]))
  self.manifests={name:(ROOT/name).read_bytes() for name in ['Cargo.toml','crates/weft-core/Cargo.toml']}
 def check(self):
  with patch.object(source,'gitbytes',side_effect=lambda root,R,name:self.manifests[name]):return source.verify_canonical_git_inventory(self.metadata,self.expected,ROOT,R)
 def transitive(self):return next(p for p in self.metadata['packages'] if p['name']=='oxrdf')
 def test_complete_eleven_and_separate_eight(self):
  result=self.check();self.assertEqual((result['completeLockedGitCount'],result['selectedNormalBuildGitCount'],result['declaredTwoDependenciesSixPatchesCount']),(11,11,8))
 def test_missing_transitive_refuses(self):
  self.metadata['packages'].remove(self.transitive())
  with self.assertRaisesRegex(ValueError,'Complete canonical'):self.check()
 def test_extra_git_refuses(self):
  self.metadata['packages'].append(dict(name='other',version='1.0.0',source=self.expected['canonicalSourceID'],id='other'))
  with self.assertRaisesRegex(ValueError,'Complete canonical'):self.check()
 def test_wrong_transitive_version_refuses(self):
  self.transitive()['version']='0.3.6'
  with self.assertRaisesRegex(ValueError,'Complete canonical'):self.check()
 def test_wrong_transitive_source_refuses(self):
  self.transitive()['source']='git+https://github.com/other/source#'+'0'*40
  with self.assertRaisesRegex(ValueError,'Complete canonical'):self.check()
 def test_duplicate_package_identity_refuses(self):
  self.metadata['packages'].append(copy.deepcopy(self.transitive()))
  with self.assertRaisesRegex(ValueError,'Duplicate canonical'):self.check()
 def test_missing_selected_transitive_refuses(self):
  key=self.transitive()['id'];self.metadata['resolve']['nodes'][0]['deps']=[d for d in self.metadata['resolve']['nodes'][0]['deps'] if d['pkg']!=key]
  with self.assertRaisesRegex(ValueError,'Complete selected'):self.check()
 def test_dev_only_transitive_is_not_promoted(self):
  key=self.transitive()['id'];next(d for d in self.metadata['resolve']['nodes'][0]['deps'] if d['pkg']==key)['dep_kinds']=[dict(kind='dev')]
  with self.assertRaisesRegex(ValueError,'Complete selected'):self.check()
 def test_build_transitive_is_selected(self):
  key=self.transitive()['id'];next(d for d in self.metadata['resolve']['nodes'][0]['deps'] if d['pkg']==key)['dep_kinds']=[dict(kind='build')];self.check()
 def test_wrong_declared_package_id_refuses(self):
  next(p for p in self.metadata['packages'] if p['name']=='umf-core')['id']='substituted'
  with self.assertRaisesRegex(ValueError,'graph edge missing|Declared canonical'):self.check()
 def test_declared_inventory_missing_refuses(self):
  self.expected['packages'].pop()
  with self.assertRaisesRegex(ValueError,'Exact two'):self.check()
 def test_declared_inventory_extra_refuses(self):
  self.expected['packages'].append(copy.deepcopy(self.expected['packages'][0]))
  with self.assertRaisesRegex(ValueError,'Exact two'):self.check()
 def test_actual_patch_missing_refuses(self):
  n='Cargo.toml';self.manifests[n]=b'\n'.join(line for line in self.manifests[n].split(b'\n') if not line.startswith(b'xsd-types ='))
  with self.assertRaisesRegex(ValueError,'Actual two'):self.check()
 def test_actual_patch_extra_refuses(self):
  n='Cargo.toml';self.manifests[n]=self.manifests[n].replace(b'[patch.crates-io]',b'[patch.crates-io]\nextra={git="https://github.com/other/source",rev="abc"}')
  with self.assertRaisesRegex(ValueError,'Actual two'):self.check()
 def test_actual_patch_wrong_version_refuses(self):
  n='Cargo.toml';self.manifests[n]=self.manifests[n].replace(b'=0.9.6',b'=0.9.5')
  with self.assertRaisesRegex(ValueError,'Declared package/version'):self.check()
 def test_actual_direct_source_tamper_refuses(self):
  n='crates/weft-core/Cargo.toml';self.manifests[n]=self.manifests[n].replace(b'https://github.com/DocumentDrivenDX/umf.git',b'https://github.com/other/umf.git',1)
  with self.assertRaisesRegex(ValueError,'Actual declaration'):self.check()
 def test_actual_direct_missing_refuses(self):
  n='crates/weft-core/Cargo.toml';self.manifests[n]=b'\n'.join(line for line in self.manifests[n].split(b'\n') if not line.startswith(b'umf-core ='))
  with self.assertRaisesRegex(ValueError,'Actual two'):self.check()
 def test_actual_direct_extra_git_refuses(self):
  n='crates/weft-core/Cargo.toml';self.manifests[n]=self.manifests[n].replace(b'[dependencies]',b'[dependencies]\nextra={git="https://github.com/other/source",rev="abc"}')
  with self.assertRaisesRegex(ValueError,'Actual two'):self.check()
if __name__=='__main__':unittest.main()
