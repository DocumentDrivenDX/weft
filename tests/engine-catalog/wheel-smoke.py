# SPDX-License-Identifier: MIT OR Apache-2.0
"""Exact installed-wheel SDK compiler custody; no native authority is granted."""
import argparse,copy,hashlib,importlib,json,pathlib,subprocess,sys,tempfile,tomllib,zipfile
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
def no_executable(value):
    if isinstance(value,dict):
        assert not any(value.get(k) is not None for k in ('artifact','sql','parameters'))
        for child in value.values():no_executable(child)
    elif isinstance(value,list):
        for child in value:no_executable(child)
def diagnostic_codes(value):
    if isinstance(value,dict):
        return ([value['code']] if isinstance(value.get('code'),str) else [])+sum((diagnostic_codes(v) for v in value.values()),[])
    if isinstance(value,list):return sum((diagnostic_codes(v) for v in value),[])
    return []
def run(label,fn,*args,status='compiled'):
    out=json.loads(fn(*(text(v) for v in args)));assert out['status']==status,(label,out)
    if status=='blocked':assert not any(k in out for k in ('artifact','sql','parameters'))
    checks.append(dict(id=label,status=status));return out
provenance=json.loads((root/'tests/engine-catalog/owner-page-profile-provenance.json').read_bytes())
def owner_page_request(name,retained):
    item=next(v for v in provenance['cases'] if v['file']==name)
    raw=(root/'tests/engine-catalog'/name).read_bytes()
    assert sha(raw)==item['sha256']
    assert sha((root/'tests/engine-catalog'/item['retainedSource']).read_bytes())==item['retainedSourceSHA256']
    request=json.loads(raw)
    # Full original source/binding custody, with the owner's explicit current profile.
    assert request['modules']==retained['modules']
    assert request['target']['bindingJson']==retained['target']['bindingJson']
    assert request['target']['bindingSha256']==retained['target']['bindingSha256']==item['bindingSHA256']
    assert sha(request['target']['bindingJson'].encode())==item['bindingSHA256']
    assert [sha(m['documentJson'].encode()) for m in request['modules']]==item['moduleDocumentSHA256s']
    for key in ('sql','parameters','options','dialect'):assert request.get(key)==retained.get(key)
    assert request['interfaceVersion']=='weft-compile/0.2.7'
    assert request['timestampComparisonProfile']=='umf-rfc3339-exact-instant/1.0.0'
    for key in ('backendId','backendVersion','targetProfile'):assert request['target'][key]==item['construction'][key]
    return request
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
    retained=copy.deepcopy(r)
    r['interfaceVersion']='weft-compile/0.2.2';r['target']['backendVersion']=backend
    # Preserve the exact old smoke construction as an authentic negative.
    refused=run(compiler+'-page-unsupported-2.2',weft.compile_engine_page_json,dict(version='weft.engine-page/0.1',compiler=page_compiler,request=r,pageSize=2),status='blocked')
    assert diagnostic_codes(refused)==['WFT-BACKEND-VERSION'];no_executable(refused)
    positive='owner-'+('spark4' if compiler=='ashlar-spark4' else 'dbsql-20261016')+'-page-profile-request.json'
    r=owner_page_request(positive,retained)
    current_read=run(compiler+'-current-profile-read',weft.compile_engine_read_json,dict(version='weft.engine-read/0.1',compiler=page_compiler,request=r))
    assert current_read['request']==r
    assert run(compiler+'-current-profile-read-check',weft.check_engine_read_json,current_read,r)==current_read
    page=run(compiler+'-page',weft.compile_engine_page_json,dict(version='weft.engine-page/0.1',compiler=page_compiler,request=r,pageSize=2))
    assert page['request']==r and page['after'] is None
    assert run(compiler+'-page-check',weft.check_engine_page_json,page,r,None)==page
    assert any(o['id']=='ashlar.candidate.keyIntegrity' for o in page['artifact']['obligations'])
    visible=len(page['outputs']);assert page['page']['visibleOutputs']==visible
    assert page['page']['keyOutputs']==[visible] and len(page['artifact']['columns'])==visible+1
    after=run(compiler+'-page-continuation',weft.check_engine_page_json,page,r,['bar'])
    assert after['after']==['bar'] and after['request']==r and after['outputs']==page['outputs']
    for key in ('visibleOutputs','keyOutputs','pageSize','generatedOutputPositions','generatedOrderPositions'):
        assert after['page'][key]==page['page'][key]
    assert page['page']['generatedContinuationFilter'] is None
    assert type(after['page']['generatedContinuationFilter']) is int
    assert after['artifact']['sql']!=page['artifact']['sql']
    for mode in ('sql','hidden-key','context','binding'):
        changed=copy.deepcopy(page);current=copy.deepcopy(r)
        if mode=='sql':changed['artifact']['sql']='SELECT 1'
        elif mode=='hidden-key':changed['page']['keyOutputs']=[0]
        elif mode=='context':current['sql']='SELECT 1'
        else:current['target']['bindingJson']+=' '
        refused=run(compiler+'-page-'+mode+'-drift',weft.check_engine_page_json,changed,current,None,status='blocked');no_executable(refused)
# Combined 0.4.42 adds issued presence/model/Graph APIs without changing legacy modes.
if tuple(map(int,weft.__version__.split('.'))) >= (0,4,42):
    assert callable(weft.prepare_application_lineage)
    for method in ('model_requirements_json','graph_requirements_json','lower_postgresql_current_presence_candidate_json','check_postgresql_current_presence_candidate_json'):
        assert callable(getattr(weft.CurrentPresencePreparation,method))
    raw=(root/'tests/current-query-presence/fixtures/unadmitted-context.request.json').read_text()
    parent=weft.prepare_application_lineage(raw);child=parent.prepare_current_presence()
    assert type(child) is weft.CurrentPresencePreparation and child.source_preparation() is parent
    assert json.loads(child.graph_requirements_json())['nativeAdmission']=='required'
    checks.append(dict(id='presence-issued-class',status='passed'))
    with tempfile.TemporaryDirectory(prefix='weft-presence-wheel-') as temporary:
        for label,script,mode in [('presence-model-requirements','python-model-requirements-bridge.py','combined'),('presence-graph-requirements','python-graph-requirements-bridge.py','combined'),('presence-postgresql-lower-check','python-bridge.py','combined')]:
            receipt=pathlib.Path(temporary)/(label+'.json')
            result=subprocess.run([sys.executable,'-I','-B',str(root/'tests/current-query-presence'/script),str(shim.parent.parent),mode,str(receipt)],check=True,capture_output=True,text=True)
            summary=json.loads(result.stdout)
            if script=='python-graph-requirements-bridge.py':assert type(summary['passed']) is int and summary['passed']==len(summary['checks']) and summary['passed']>0 and summary['nativeAdmission'] is False
            else:assert summary['passed'] is True and type(summary['checks']) is int and summary['checks']>0 and summary['nativeDatabase'] is False
            if receipt.exists():
                detail=json.loads(receipt.read_bytes());assert detail['passed'] is True
                if 'nativeModuleSha256' in detail:assert detail['nativeModuleSha256']==before['nativeModuleSha256']
            checks.append(dict(id=label,status='passed'))
assert pins()==before
report=dict(version=weft.__version__,sourceCommit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),wheel=a.wheel.name,pinsBeforeAfter=before,controls=len(checks),checks=checks,nativeAdmission=False,securityReady=False,temporalQualification='held')
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(dict(controls=len(checks),nativeAdmission=False,temporalQualification='held')))
