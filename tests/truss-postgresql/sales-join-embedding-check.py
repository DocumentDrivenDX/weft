"""Verify explicit conformance configuration through the actual Python ABI.
@covers US-003-AC1: original sales grouped join full compile artifacts for all sixteen row/props cuts.
@covers US-003-AC3: malformed/version/candidate/pin refusals preserve no SQL.
This is test-original instrumentation, not released profile registration.
"""
import copy, hashlib, json, os, re, subprocess
from pathlib import Path
import weft
ROOT=Path(__file__).resolve().parents[2]
F=ROOT/'tests/truss-postgresql/fixtures'
selected=[]
for case in json.loads((F/'original-sales-join-inputs.json').read_text()):
 label=''.join(h[0] for h in case['homes'])
 case['id']=label;case['configuration']=json.dumps(case['configuration'])
 selected.append((case,next(x['response'] for x in json.loads((F/'original-sales-join-rust-public.json').read_text()) if x['homes']==case['homes'])))
assert len(selected)==16
out=ROOT/'target/b005/sales-join-embedding';out.mkdir(parents=True,exist_ok=True)
corpus=[];actual=[];repeats=0
# The host wrapper performs no database or subprocess work.
subprocess.Popen=lambda *a,**kw: (_ for _ in ()).throw(AssertionError('subprocess forbidden'))
os.environ['PATH']=''
for case,expected in selected:
    request=copy.deepcopy(case['request']);old=request['target']['bindingSha256']
    configuration=case['configuration']
    binding=request['target']['bindingJson']+' '
    new=hashlib.sha256(binding.encode()).hexdigest();request['target']['bindingJson']=binding;request['target']['bindingSha256']=new
    expected=json.loads(json.dumps(expected).replace(old,new))
    fixed=json.loads(weft.compile_json(json.dumps(request)));assert fixed['status']=='blocked' and 'sql' not in fixed
    raw=weft.compile_json_with_conformance_configuration(json.dumps(request),configuration)
    value=json.loads(raw);assert value==expected,(case['id'],value)
    assert raw==weft.compile_json_with_conformance_configuration(json.dumps(request),configuration);repeats+=1
    variants=[('compiled',request,configuration),('malformed',request,'{}'),('wrong-version',request,configuration.replace('weft-original-conformance-composition/0.1.0','unsupported/1'))]
    disabled=copy.deepcopy(request);disabled['options']['allowCandidate']=False;variants.append(('candidate-disabled',disabled,configuration))
    invalid=copy.deepcopy(request);invalid['target']['bindingSha256']='0'*64;variants.append(('wrong-digest',invalid,configuration))
    for kind,q,cfg in variants:
        response=weft.compile_json_with_conformance_configuration(json.dumps(q),cfg);v=json.loads(response)
        if kind!='compiled':assert v['status']=='blocked' and 'sql' not in v
        corpus.append(dict(id=case['id']+'-'+kind,request=q,configuration=cfg));actual.append(dict(raw=response))
(out/'cases.json').write_text(json.dumps(corpus)+'\n');(out/'reports.json').write_text(json.dumps(actual)+'\n')
summary=dict(cases=len(corpus),positiveResponses=16,refusals=len(corpus)-16,deterministicRepeats=repeats,unlistedBinding=True,subprocessDisabled=True,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
(out/'python-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
