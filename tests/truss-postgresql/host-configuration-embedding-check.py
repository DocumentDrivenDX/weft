"""Verify explicit conformance configuration through the actual Python ABI.
@covers US-003-AC1: exact complete compile artifacts for host-supplied compositions.
@covers US-003-AC3: malformed/version/candidate/pin refusals preserve no SQL.
This is test-original instrumentation, not released profile registration.
"""
import copy, hashlib, json, os, re, subprocess
from pathlib import Path
import weft
ROOT=Path(__file__).resolve().parents[2]
source=(ROOT/'crates/weft-postgresql/src/conformance_original.rs').read_text()
presets=dict(re.findall(r'"([0-9a-f]{64})"\s*,\s*include_str!\(\s*"([^"\n]+)"',source))
presets.update(re.findall(r'target.binding_sha256\s*==\s*"([0-9a-f]{64})"\s*\{\s*include_str!\(\s*"([^"\n]+)"',source))
base=ROOT/'target/b005/original-embedding'
cases=json.loads((base/'cases.json').read_text());reports=json.loads((base/'reports.json').read_text())
selected=[(c,json.loads(r['raw'])) for c,r in zip(cases,reports) if json.loads(r['raw'])['status']=='compiled']
assert len(selected)==84
out=ROOT/'target/b005/host-configuration';out.mkdir(parents=True,exist_ok=True)
corpus=[];actual=[];repeats=0
# The host wrapper performs no database or subprocess work.
subprocess.Popen=lambda *a,**kw: (_ for _ in ()).throw(AssertionError('subprocess forbidden'))
os.environ['PATH']=''
for case,expected in selected:
    request=copy.deepcopy(case['request']);old=request['target']['bindingSha256']
    configuration=(ROOT/'crates/weft-postgresql/src'/presets[old]).resolve().read_text()
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
summary=dict(cases=len(corpus),positiveResponses=84,refusals=len(corpus)-84,deterministicRepeats=repeats,unlistedBinding=True,subprocessDisabled=True,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
(out/'python-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
