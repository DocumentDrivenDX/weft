"""Exact installed-wheel SDK compiler custody; no native authority is granted."""
import argparse,copy,hashlib,importlib,json,pathlib,subprocess,tomllib,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[2]
def sha(v):return hashlib.sha256(v).hexdigest()
def text(v):return json.dumps(v,separators=(',',':'))
native=pathlib.Path(importlib.import_module('weft.weft').__file__);shim=pathlib.Path(weft.__file__)
with zipfile.ZipFile(a.wheel) as z:
    modules=[n for n in z.namelist() if n.endswith(('.so','.pyd'))];assert len(modules)==1
    assert native.read_bytes()==z.read(modules[0]) and shim.read_bytes()==z.read('weft/__init__.py')
def pins():return dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(shim.read_bytes()))
before=pins();assert weft.__version__==tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text())['project']['version']
checks=[]
def run(label,fn,*args,status='compiled'):
    out=json.loads(fn(*(text(v) for v in args)));assert out['status']==status,(label,out)
    if status=='blocked':assert not any(k in out for k in ('artifact','sql','parameters'))
    checks.append(dict(id=label,status=status));return out
r=json.loads((root/'tests/engine-catalog/native-truss-export-request.json').read_bytes())
read=run('read',weft.compile_engine_read_json,dict(version='weft.engine-read/0.1',compiler='truss-runtime',request=r));assert read['request']==r
assert run('read-check',weft.check_engine_read_json,read,r)==read
page=run('page',weft.compile_engine_page_json,dict(version='weft.engine-page/0.1',compiler='truss-runtime',request=r,pageSize=999))
assert run('page-check',weft.check_engine_page_json,page,r,None)==page
export=run('export',weft.compile_engine_export_json,page,r)
assert export['program']==page and export['request']==r and export['checkedPage']==page
assert export['artifact']['logicalPlan']['limit'] is None
assert export['artifact']['backend']['backendVersion']=='0.4.42'
assert any(o['id']=='truss.runtime.cursor-export/0.1' for o in export['artifact']['obligations'])
assert page['artifact']['logicalPlan']['limit'] is not None
changed=copy.deepcopy(r);changed['sql']='SELECT 1'
run('read-context-drift',weft.check_engine_read_json,read,changed,status='blocked')
run('page-context-drift',weft.check_engine_page_json,page,changed,None,status='blocked')
run('export-context-drift',weft.compile_engine_export_json,page,changed,status='blocked')
tampered=copy.deepcopy(page);tampered['artifact']['sql']='SELECT 1'
run('export-program-drift',weft.compile_engine_export_json,tampered,r,status='blocked')
raw=json.loads((root/'tests/raw-postgresql/read-request.json').read_bytes())
for op,label in [('IS DISTINCT FROM','distinct'),('IS NOT DISTINCT FROM','not-distinct')]:
    raw['sql']='SELECT o.id AS id FROM "commerce"."Order" AS o WHERE o.status '+op+' :value'
    raw['parameters']=[dict(name='value',carrier='string',nullable=True)]
    out=run(label,weft.compile_raw_read_json,raw);assert op in out['sql'] and out['parameterOrder']==['value'] and out['parameters'][0]['nullable'] is True
    raw['sql']='SELECT o.id AS id FROM "commerce"."Order" AS o WHERE o.status '+op+' NULL';raw['parameters']=[]
    out=run(label+'-null',weft.compile_raw_read_json,raw);assert ('IS NOT NULL' if label=='distinct' else 'IS NULL') in out['sql']
raw['sql']='SELECT o.id AS id FROM "commerce"."Order" AS o WHERE o.amount IS DISTINCT FROM o.status'
run('distinct-carrier-mismatch',weft.compile_raw_read_json,raw,status='blocked')
for compiler,fixture,page_compiler,backend in [
    ('ashlar-spark4','native-ashlar-spark4-request.json','ashlar-spark4-pages','0.1.0-local-spark4-pages-candidate'),
    ('ashlar-dbsql-20261016','native-ashlar-dbsql-20261016-request.json','ashlar-dbsql-20261016-pages','0.1.0-managed-20261016-pages-candidate')]:
    r=json.loads((root/'tests/engine-catalog'/fixture).read_bytes())
    read=run(compiler+'-read',weft.compile_engine_read_json,dict(version='weft.engine-read/0.1',compiler=compiler,request=r));assert read['request']==r
    assert run(compiler+'-read-check',weft.check_engine_read_json,read,r)==read
    r['interfaceVersion']='weft-compile/0.2.2';r['target']['backendVersion']=backend
    page=run(compiler+'-page',weft.compile_engine_page_json,dict(version='weft.engine-page/0.1',compiler=page_compiler,request=r,pageSize=2))
    assert run(compiler+'-page-check',weft.check_engine_page_json,page,r,None)==page
    assert any(o['id']=='ashlar.candidate.keyIntegrity' for o in page['artifact']['obligations'])
assert pins()==before
report=dict(version=weft.__version__,sourceCommit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),wheel=a.wheel.name,pinsBeforeAfter=before,controls=len(checks),checks=checks,nativeAdmission=False,securityReady=False,temporalQualification='held')
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(controls=len(checks),nativeAdmission=False,temporalQualification='held')))
