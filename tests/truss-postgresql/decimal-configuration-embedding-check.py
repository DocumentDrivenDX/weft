"""Verify decimal conformance configuration through the actual Python ABI.
@covers US-003-AC1: exact complete decimal SUM artifacts for every admitted precision/scale and both homes.
@covers US-003-AC3: malformed/version/candidate/pin refusals preserve no SQL.
This is test-original instrumentation, not released profile registration.
"""
import base64, copy, hashlib, json, os, re, subprocess
from pathlib import Path
import weft
ROOT=Path(__file__).resolve().parents[2]
CAPTURES=Path(os.environ['WEFT_ORIGINAL_DECIMAL_COMPOSITION_CAPTURE'])
NATIVE_CAPTURES=Path(os.environ['WEFT_ORIGINAL_DECIMAL_CAPTURE_DIR'])
selected=[]
for precision in range(1,29):
    for scale in range(precision+1):
        transports=json.loads((CAPTURES/f'original-decimal-{precision}-{scale}-transports.json').read_text())
        assert len(transports)==2
        for transport in transports:
            home=transport['home'];assert home in ['row','props'] and transport['kind']=='sum'
            native=json.loads((NATIVE_CAPTURES/f'original-decimal-{precision}-{scale}-{home}-sum.json').read_text())
            assert transport['response']['sql']==native['sql']
            assert transport['response']['parameters']==native['parameters']
            guards=[o['parameters']['sql'] for o in transport['response']['obligations'] if 'sql' in o['parameters']]
            assert guards==native['checks']
            configuration=(CAPTURES/f'original-decimal-{precision}-{scale}-{home}-composition.json').read_text()
            document=json.loads(transport['request']['modules'][0]['documentJson'])
            field=next(f for f in document['modules'][0]['elements'] if f['id']=='order-total')
            assert field['facets']==dict(precision=precision,scale=scale)
            cfg=json.loads(configuration)
            for definition in list(cfg.get('comparators',{}).values()) + [leaf for prop in cfg['properties'] for leaf in prop['leafCodecs'].values()]:
                original=json.loads(definition['originalJson'])
                for pointer,encoded in definition['originalArtifacts'].items():
                    artifact=original
                    for part in pointer.split('/'):
                        artifact=artifact[int(part)] if isinstance(artifact,list) else artifact[part]
                    actual_bytes=base64.b64decode(encoded,validate=True)
                    pinned_bytes=base64.b64decode(artifact['bytesBase64'],validate=True)
                    assert actual_bytes==pinned_bytes and hashlib.sha256(actual_bytes).hexdigest()==artifact['sha256']
            selected.append((dict(id=f'decimal-{precision}-{scale}-{home}',request=transport['request'],configuration=configuration),transport['response']))
assert len(selected)==868
out=ROOT/'target/b005/decimal-configuration';out.mkdir(parents=True,exist_ok=True)
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
assert len(corpus)==4340
summary=dict(domainPairs=434,storageHomes=['row','props'],cases=len(corpus),positiveResponses=868,refusals=len(corpus)-868,deterministicRepeats=repeats,unlistedBinding=True,subprocessDisabled=True,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest())
(out/'python-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
