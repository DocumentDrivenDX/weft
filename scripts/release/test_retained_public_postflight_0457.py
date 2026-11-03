"""Synthetic tiny ZIPs only. Never project/decompress original venv."""
from pathlib import Path
from unittest.mock import patch
import tempfile,zipfile,json,stat,copy,unittest
import project_retained_public_postflight_0457 as p
class Projection(unittest.TestCase):
 def fixture(self,path,unsafe=None):
  data=b'{"synthetic":"control"}';manifest=json.dumps({'files':{'report.json':{'bytes':len(data),'sha256':p.sha(data)}}}).encode()
  with zipfile.ZipFile(path,'w') as z:
   for name,raw in [('proof/report.json',data),('proof/proof-manifest.json',manifest),('proof-venv/do-not-read.py',b'NEVER IMPORT EXECUTE OR READ THIS MEMBER')]:
    i=zipfile.ZipInfo(name);i.create_system=3;i.external_attr=(stat.S_IFLNK|0o777 if unsafe==name else 0o100644)<<16;z.writestr(i,raw)
  with zipfile.ZipFile(path) as z:
   if unsafe:return None
   rows=p.inventory(z);members=[dict(r,sha256=p.sha(z.read(r['path']))) for r in rows if r['path'].startswith('proof/')];seal=dict(entries=rows,count=len(rows),expandedBytes=sum(r['bytes'] for r in rows));whitelist=dict(members=members,count=2,expandedBytes=sum(r['bytes'] for r in members),originalProofManifestPath='proof/proof-manifest.json',originalProofManifestSha256=p.sha(manifest));return seal,whitelist
 def meta(self):return {n:b'{}' for n in ['projection-provenance/original-terminal-run.json','projection-provenance/original-terminal-jobs.json','projection-provenance/original-artifact-api.json']}
 def test_selected_members_verified_and_excluded_venv_never_read(self):
  with tempfile.TemporaryDirectory() as t:
   path=Path(t)/'input.zip';seal,white=self.fixture(path)
   with zipfile.ZipFile(path) as z:
    original=z.read;seen=[]
    def guarded(name,*a,**kw):self.assertFalse(name.startswith('proof-venv/'));seen.append(name);return original(name,*a,**kw)
    with patch.object(z,'read',side_effect=guarded):payload=p.verified_members(z,seal,white)
   self.assertEqual(set(payload),{'proof/report.json','proof/proof-manifest.json'});self.assertEqual(set(seen),set(payload))
 def test_bad_member_hash_refuses_before_any_output(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);seal,white=self.fixture(d/'input.zip');white['members'][0]['sha256']='0'*64
   with zipfile.ZipFile(d/'input.zip') as z,self.assertRaises(ValueError):p.verified_members(z,seal,white)
   self.assertFalse((d/'output.zip').exists())
 def test_extra_missing_rebound_whitelist_or_manifest_refuses(self):
  with tempfile.TemporaryDirectory() as t:
   path=Path(t)/'input.zip';seal,white=self.fixture(path)
   for mode in ['extra','missing','manifest','metadata']:
    w=copy.deepcopy(white)
    if mode=='extra':w['members'].append(dict(seal['entries'][-1],sha256='0'*64));w['count']+=1
    if mode=='missing':w['members'].pop();w['count']-=1
    if mode=='manifest':w['originalProofManifestSha256']='0'*64
    if mode=='metadata':w['members'][0]['mode']=0o100777
    with self.subTest(mode=mode),zipfile.ZipFile(path) as z,self.assertRaises(ValueError):p.verified_members(z,seal,w)
 def test_sealed_complete_inventory_drift_refuses(self):
  with tempfile.TemporaryDirectory() as t:
   path=Path(t)/'input.zip';seal,white=self.fixture(path);seal['entries'][-1]['bytes']+=1
   with zipfile.ZipFile(path) as z,self.assertRaises(ValueError):p.verified_members(z,seal,white)
 def test_unsupported_type_even_excluded_venv_refuses(self):
  with tempfile.TemporaryDirectory() as t:
   path=Path(t)/'input.zip';self.fixture(path,'proof-venv/do-not-read.py')
   with zipfile.ZipFile(path) as z,self.assertRaises(ValueError):p.inventory(z)
 def test_capsule_new_identity_original_bytes_manifest_and_caps(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);seal,white=self.fixture(d/'input.zip')
   with zipfile.ZipFile(d/'input.zip') as z:payload=p.verified_members(z,seal,white)
   record=dict(schema='synthetic-control',projectionController={'kind':'new-projector'},originalConsumerProvenance={'controller':'original-V','run':1},originalArtifactAccepted=False,consumerReexecuted=False,newConsumerClaim=False)
   result=p.write_capsule(d/'capsule',payload,record,json.dumps(seal).encode(),self.meta(),dict(entries=160,expandedBytes=100*1024*1024,zipBytes=80*1024*1024))
   files=[f for f in (d/'capsule').rglob('*') if f.is_file()];self.assertEqual(len(files),8)
   for n,b in payload.items():self.assertEqual((d/'capsule'/n).read_bytes(),b)
   self.assertFalse(json.loads((d/'capsule/retention-projection.json').read_text())['originalArtifactAccepted']);self.assertFalse((d/'capsule/proof-venv').exists())
 def test_capsule_cannot_bypass_unchanged_bounds_or_relabel_original(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);record=dict(originalArtifactAccepted=False,consumerReexecuted=False,newConsumerClaim=False)
   with self.assertRaises(ValueError):p.write_capsule(d/'capsule',{'proof/x':b'x'},record,b'{}',self.meta(),dict(entries=1,expandedBytes=100,zipBytes=100))
   self.assertFalse((d/'capsule').exists())
   record['originalArtifactAccepted']=True
   with self.assertRaises(ValueError):p.write_capsule(d/'capsule',{},record,b'{}',self.meta(),dict(entries=160,expandedBytes=100*1024*1024,zipBytes=80*1024*1024))
 def test_quarantine_and_state_cannot_be_inside_upload_or_checkout(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);root=d/'checkout';p.contained_paths(d/'input',d/'state',d/'upload',root)
   for q,state,out in [(d/'upload/input',d/'state',d/'upload'),(d/'input',d/'upload/state',d/'upload'),(d/'input',d/'state',root/'upload')]:
    with self.assertRaises(ValueError):p.contained_paths(q,state,out,root)
 def test_default_off_readonly_workflow_uploads_only_capsule(self):
  root=Path(__file__).resolve().parents[2];text=(root/'.github/workflows/retained-public-postflight-0457.yml').read_text();self.assertIn('default: false',text);self.assertEqual(text.count('runs-on:'),1);self.assertIn('actions: read',text);self.assertNotIn('id-token:',text);self.assertNotIn('pip ',text);self.assertNotIn('matrix:',text);self.assertIn('path: ${{ runner.temp }}/weft-postflight-projector-upload/',text)
if __name__=='__main__':unittest.main(verbosity=2)
