"""Derived compiler-domain fixture through exact wheel; no native storage admission."""
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
bundle=json.loads((root/'tests/truss-query-abi/fixtures/native-query-bundle.json').read_bytes());doc=json.loads(bundle['modules'][0]['documentJson']);field=next(f for f in doc['modules'][0]['elements']if f['id']=='A-value');field.update(scalarType='decimal',facets={'precision':12,'scale':2})
raw_doc=text(doc);bundle['modules'][0]['documentJson']=raw_doc;bundle['modules'][0]['pin']['sha256']=sha(raw_doc.encode())
binding=json.loads(bundle['bindingJson']);binding['documents'][0]['sha256']=sha(raw_doc.encode());raw_binding=json.dumps(binding,indent=2)
def request(sql,parameters=None,version='0.4.39'):
 return dict(interfaceVersion='weft-compile/0.2.6',dialect='weft-sql/0.2.0',timestampComparisonProfile='umf-rfc3339-exact-instant/1.0.0',modules=bundle['modules'],sql=sql,parameters=parameters or{},exactParameters={},options=dict(allowCandidate=True),target=dict(backendId='truss.postgresql.runtime',backendVersion=version,targetProfile='truss-runtime-pg18.6/0.5',bindingJson=raw_binding,bindingSha256=sha(raw_binding.encode())))
checks=[]
def run(label,r,expected='compiled',page=None):
 raw=weft.compile_truss_runtime_json(text(r))if page is None else weft.compile_truss_runtime_page_json(text(r),2,page)
 out=json.loads(raw);assert out['status']==expected,(label,out)
 if expected=='compiled':
  artifact=out.get('artifact',out);assert artifact['backend']['backendVersion']==r['target']['backendVersion'];assert any(p['value']==raw_binding for p in artifact['parameters'])
 else:assert 'sql'not in out
 checks.append({'id':label,'status':expected});return out
run('projection',request('SELECT a."A-value" FROM "A" a'))
for i,value in enumerate(['9999999999.99','-9999999999.99','9999999999.9900','-0001.2300','0','-0.00']):
 r=request('SELECT a."A-value" FROM "A" a WHERE a."A-value"=:price',{'price':dict(family='decimal',value=value)});out=run('exact-parameter-'+str(i),r);assert any(p['value']==value for p in out['parameters'])
for i,value in enumerate(['10000000000.00','-10000000000.00','0.001','NaN','Infinity','1.2.3','1e2']):run('nonrepresentable-'+str(i),request('SELECT a."A-value" FROM "A" a WHERE a."A-value"=:price',{'price':dict(family='decimal',value=value)}),'blocked')
for version in ['0.4.38','0.4.37']:run('old-version-'+version,request('SELECT a."A-value" FROM "A" a',version=version),'blocked')
run('related-decimal-key',request('SELECT RELATED_KEYS(a.children,4) FROM "A" a'))
r=request('SELECT a.* FROM "A" a')
# Empty continuation list is invalid; initial page is the explicit None call.
out=json.loads(weft.compile_truss_runtime_page_json(text(r),2));assert out['status']=='compiled',out;checks.append({'id':'first-typed-page','status':'compiled'})
run('next-typed-page',r,page=['9999999999.9900','label'])
run('bad-typed-page',r,'blocked',page=['10000000000.00','label'])
assert len(checks)==20
assert pins==dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(pathlib.Path(weft.__file__).read_bytes()))
commit=None
if(root/'.git').exists():
 import subprocess
 commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
report=dict(version=weft.__version__,sourceCommit=commit,wheel=a.wheel.name,pinsBeforeAfter=pins,controls=len(checks),checks=checks,nativeAdmission=False,securityReady=False,scope=__doc__)
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'controls':len(checks),'nativeAdmission':False}))
