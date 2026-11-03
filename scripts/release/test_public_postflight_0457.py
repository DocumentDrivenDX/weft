"""Pure publication/resolver/ownership refusals; NO native import/network/build."""
import copy,json,unittest
from pathlib import Path
from unittest.mock import patch
import verify_public_postflight_0457 as v
class PublicControls(unittest.TestCase):
 def setUp(self):
  self.e=json.loads((Path(__file__).with_name('public-qualified-cohort-0457.json')).read_text())
  self.name=next(n for n,r in self.e['wheels'].items() if r['target']=='x86_64-unknown-linux-gnu');self.w=self.e['wheels'][self.name]
  self.report={'install':[dict(metadata={'name':'umf-weft','version':'0.4.57'},requested=True,download_info=dict(url='https://files.pythonhosted.org/packages/'+self.name,archive_info={'hashes':{'sha256':self.e['archives'][self.name]}}))]}
 def test_actual_qualified_tuple_has_exact_three_platforms_and_four_archives(self):
  self.assertEqual(len(self.e['wheels']),3);self.assertEqual(len(self.e['archives']),4);self.assertEqual(len(self.e['assetNames']),33)
  self.assertEqual(v.selected_public_wheel(self.report,self.e)[0],self.name)
 def test_public_urls_refuse_private_file_and_lookalike_hosts(self):
  for u in ['file:///tmp/wheel','https://private.example/wheel','https://files.pythonhosted.org.evil/x','http://files.pythonhosted.org/x','https://u:p@files.pythonhosted.org/x']:
   with self.subTest(u=u),self.assertRaises(ValueError):v.public_url(u)
 def test_selected_old_version_extra_dependencies_hash_or_platform_refuse(self):
  for key in ['version','extra','hash','platform','requested']:
   r=copy.deepcopy(self.report)
   if key=='version':r['install'][0]['metadata']['version']='0.4.56'
   if key=='extra':r['install'].append(copy.deepcopy(r['install'][0]))
   if key=='hash':r['install'][0]['download_info']['archive_info']['hashes']['sha256']='0'*64
   if key=='platform':r['install'][0]['download_info']['url']='https://files.pythonhosted.org/'+next(n for n in self.e['wheels'] if n!=self.name)
   if key=='requested':r['install'][0]['requested']=False
   with self.subTest(key=key),self.assertRaises(ValueError):v.selected_public_wheel(r,self.e)
 def inventory(self):return {'info':{'name':'umf-weft','version':'0.4.57'},'urls':[dict(filename=n,digests={'sha256':h},url='https://files.pythonhosted.org/'+n,yanked=False) for n,h in self.e['archives'].items()]}
 def test_complete_public_inventory_green(self):self.assertEqual(len(v.public_inventory(self.inventory(),self.e)),4)
 def test_public_extra_missing_yanked_duplicate_or_digest_drift_refuse(self):
  for key in ['extra','missing','yanked','duplicate','hash']:
   r=self.inventory()
   if key=='extra':r['urls'].append(dict(filename='extra.whl',digests={'sha256':'0'*64},url='https://files.pythonhosted.org/x',yanked=False))
   if key=='missing':r['urls'].pop()
   if key=='yanked':r['urls'][0]['yanked']=True
   if key=='duplicate':r['urls'].append(r['urls'][0])
   if key=='hash':r['urls'][0]['digests']['sha256']='0'*64
   with self.subTest(key=key),self.assertRaises(ValueError):v.public_inventory(r,self.e)
 def receipts(self):
  common=dict(tag='v0.4.57',source_sha=v.R,distribution='umf-weft',version='0.4.57',files=self.e['archives'])
  return dict(common,schema='pypi-publication-receipt.v1',status='passed',registry='pypi',registry_bytes_verified=True,publication_mode='oidc_upload',oidc_trusted_publisher=True),dict(common,schema='pypi-release-source.v1',status='verified',repository='DocumentDrivenDX/weft')
 def test_original_terminal_publication_green_without_replaying_verifier(self):v.publication_receipts(*self.receipts(),self.e)
 def test_wrong_source_missing_byteproof_or_false_oidc_refuse(self):
  for key in ['source','bytes','oidc','files','source-status']:
   p,s=self.receipts();p=copy.deepcopy(p);s=copy.deepcopy(s)
   if key=='source':p['source_sha']='f'*40
   if key=='bytes':p['registry_bytes_verified']=False
   if key=='oidc':p['oidc_trusted_publisher']=False
   if key=='files':s['files'].pop(self.name)
   if key=='source-status':s['status']='unchecked'
   with self.subTest(key=key),self.assertRaises(ValueError):v.publication_receipts(p,s,self.e)
 def owned(self):return dict(name='umf-weft',version='0.4.57',requiresDist=[],ownedFiles=['/private/tmp/fresh/lib/weft/weft.so','/private/tmp/fresh/lib/weft/__init__.py'],nativePath='/private/tmp/fresh/lib/weft/weft.so',shimPath='/private/tmp/fresh/lib/weft/__init__.py',nativeSha256=self.w['nativeModuleSha256'],shimSha256=self.w['shimSha256'])
 def test_installed_ownership_and_both_hashes_green(self):v.installed_binding(self.owned(),self.w,'/private/tmp/fresh')
 def test_wrong_or_unowned_native_shim_path_hashes_refuse(self):
  for key in ['outside','unowned','nativehash','shimhash','version','duplicates']:
   r=self.owned()
   if key=='outside':r['nativePath']='/tmp/source/weft.so';r['ownedFiles'].append(r['nativePath'])
   if key=='unowned':r['ownedFiles'].remove(r['shimPath'])
   if key=='nativehash':r['nativeSha256']='0'*64
   if key=='shimhash':r['shimSha256']='0'*64
   if key=='version':r['version']='0.4.56'
   if key=='duplicates':r['ownedFiles'].append(r['ownedFiles'][0])
   with self.subTest(key=key),self.assertRaises(ValueError):v.installed_binding(r,self.w,'/private/tmp/fresh')
 def test_actual_api_run_head_repository_path_and_terminal_outcome(self):
  good=dict(id=42,head_sha=v.P,path='.github/workflows/publish-pypi.yml',repository={'full_name':'DocumentDrivenDX/weft'},status='completed',conclusion='success')
  v.run_record(good,42,v.P,good['path'])
  for key,bad in [('id',43),('head_sha',v.R),('path','other.yml'),('repository',{'full_name':'evil/weft'}),('conclusion','failure'),('status','in_progress')]:
   r=dict(good);r[key]=bad
   with self.subTest(key=key),self.assertRaises(ValueError):v.run_record(r,42,v.P,good['path'])
 def test_default_off_linux_only_readonly_failure_retention(self):
  text=(Path(__file__).resolve().parents[2]/v.WORKFLOW).read_text();self.assertIn('default: false',text);self.assertEqual(text.count('runs-on:'),1);self.assertIn('runs-on: ubuntu-24.04',text);self.assertIn('if: always()',text);self.assertIn('actions: read',text);self.assertNotIn('id-token:',text);self.assertNotIn('matrix:',text)
if __name__=='__main__':unittest.main()
