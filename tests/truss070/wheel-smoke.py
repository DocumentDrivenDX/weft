"""Exact wheel compiler checks against genuine native70 owner inputs; no native action admission."""
import argparse,hashlib,importlib,json,pathlib,tomllib,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();root=pathlib.Path(__file__).resolve().parents[2]
def sha(b):return hashlib.sha256(b).hexdigest()
def text(v):return json.dumps(v,separators=(',',':'))
native=pathlib.Path(importlib.import_module('weft.weft').__file__)
with zipfile.ZipFile(a.wheel)as z:
 names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1
 assert native.read_bytes()==z.read(names[0])and pathlib.Path(weft.__file__).read_bytes()==z.read('weft/__init__.py')
pins=dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(pathlib.Path(weft.__file__).read_bytes()))
assert weft.__version__==tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text())['project']['version']
bundle=json.loads((root/'tests/truss070/native-query-bundle.json').read_bytes())
manifest=(root/'tests/truss070/native-runtime-manifest.json').read_bytes()
assert sha(manifest)==bundle['binding']['releaseSha256']=='0d292c3681986b3664ff3af9927039d10cf20ef029511e33ccd7d76c566041e3'
assert json.loads(bundle['bindingJson'])==bundle['binding']
assert bundle['binding']['releaseId']=='truss-catalog-runtime/0.70-candidate'
def request(sql,version='0.4.40'):
 raw=bundle['bindingJson']
 return dict(interfaceVersion='weft-compile/0.2.6',dialect='weft-sql/0.2.0',timestampComparisonProfile='umf-rfc3339-exact-instant/1.0.0',modules=bundle['modules'],sql=sql,parameters={},exactParameters={},options=dict(allowCandidate=True),target=dict(backendId='truss.postgresql.runtime',backendVersion=version,targetProfile='truss-runtime-pg18.6/0.4',bindingJson=raw,bindingSha256=sha(raw.encode())))
checks=[]
def run(label,r,expected='compiled',page=False):
 out=json.loads(weft.compile_truss_runtime_page_json(text(r),2) if page else weft.compile_truss_runtime_json(text(r)))
 assert out['status']==expected,(label,out)
 artifact=out.get('artifact',out)
 if expected=='compiled':
  assert artifact['backend']['backendVersion']=='0.4.40'
  assert any(p['value']==r['target']['bindingJson'] for p in artifact['parameters'])
 else:assert 'sql' not in out
 checks.append(dict(id=label,status=expected))
run('projection',request('SELECT p.name FROM products p'))
run('count',request('SELECT COUNT(*) FROM products p'))
run('page',request('SELECT p.* FROM products p'),page=True)
run('old-backend',request('SELECT p.name FROM products p','0.4.37'),'blocked')
r=request('SELECT p.name FROM products p');b=json.loads(r['target']['bindingJson']);b['releaseSha256']='0'*64;r['target']['bindingJson']=text(b);r['target']['bindingSha256']=sha(text(b).encode());run('wrong-runtime-digest',r,'blocked')
assert pins==dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(pathlib.Path(weft.__file__).read_bytes()))
import subprocess
report=dict(version=weft.__version__,sourceCommit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),wheel=a.wheel.name,pinsBeforeAfter=pins,controls=len(checks),checks=checks,nativeAdmission=False,securityReady=False,scope='Genuine native70 public catalog/write/read binding through exact wheel compiler; compilation only, no action or security admission')
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'controls':len(checks),'nativeAdmission':False}))
