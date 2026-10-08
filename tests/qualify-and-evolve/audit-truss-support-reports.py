"""Audit real retained native data with independent expectations per exact scope."""
import contextlib,hashlib,io,json,os,runpy,tempfile
from decimal import Decimal
from pathlib import Path
from evidence_audit import audit
root=Path(__file__).resolve().parents[2]
with contextlib.redirect_stdout(io.StringIO()):
    ns=runpy.run_path(str(root/'tests/qualify-and-evolve/audit-native-profile-scopes.py'))
custody=json.loads((root/'docs/helix/04-build/evidence/B-007-truss-application-native/reports-custody.json').read_text())
def exact(value):
    if isinstance(value,Decimal):
        sign,digits,exponent=value.as_tuple();digits=list(digits)
        while len(digits)>1 and digits[-1]==0:digits.pop();exponent+=1
        if not any(digits):sign=0;exponent=0
        return {'exactBaseTen':{'sign':sign,'digits':''.join(map(str,digits)),'exponent':exponent}}
    if isinstance(value,(list,tuple)):return [exact(v) for v in value]
    return value
by_id={r['id']:r for r in ns['reports']};results=[]
hosts={}
for host in ['python','browser']:
    path=Path(os.environ.get(f'WEFT_{host.upper()}_RECEIPTS',str(root/f'docs/helix/04-build/evidence/B-007-truss-{host}-case-receipts/receipts.json')))
    receipt=json.loads(path.read_text());summary=receipt['summary']
    assert summary['cases']==76 and summary['byteParity'] is True
    retained=json.loads((root/f'docs/helix/04-build/evidence/B-007-truss-embeddings/{host}-summary.json').read_text())
    keys=['extensionSha256','python','version'] if host=='python' else ['wasmSha256','browser','playwrightVersion','wasmBytes']
    assert all(summary[key]==retained[key] for key in keys),f'{host} runtime identity changed'
    if host=='python':assert summary['subprocessDisabled'] is True and len(summary['extensionSha256'])==64
    else:assert summary['nodeGlobals'] is False and len(summary['wasmSha256'])==64
    index={c['id']:c for c in receipt['cases']}
    assert len(index)==len(receipt['cases'])==76 and set(index)==set(by_id)
    for id,r in by_id.items():
        case=index[id];expected=hashlib.sha256(r['raw'].encode()).hexdigest()
        assert case['actualSha256']==case['expectedSha256']==expected
        request=json.dumps(ns['requests'][id],ensure_ascii=False,**({'separators':(',',':')} if host=='browser' else {}))
        assert case['requestSha256']==hashlib.sha256(request.encode()).hexdigest()
    hosts[host]={'casesJoined':len(index),'receiptSha256':hashlib.sha256(path.read_bytes()).hexdigest(),'runtime':summary}
with tempfile.TemporaryDirectory() as tmp:
    for index,(key,ids) in enumerate(sorted(ns['scopes'].items())):
        scope=json.loads(key);assert len(scope['modelPins'])==1
        profile={'compilerVersion':scope['compilerVersion'],'dialect':scope['dialect'],'irVersion':scope['irVersion'],'backendVersion':scope['backend']['backendVersion'],'targetProfile':scope['backend']['targetProfile'],'engineVersion':custody['engine'],'layoutRevision':'sha256:'+hashlib.sha256((root/'tests/truss-postgresql/upstream/storage-realization-pins.json').read_bytes()).hexdigest(),'modelSha256':scope['modelPins'][0]['sha256'],'bindingSha256':scope['bindingSha256'],'settings':{'encoding':custody['encoding'],'comparison':'explicit C in emitted SQL'}}
        cases=[];observed=[]
        for id in ids:
            report=by_id[id];columns=report['response']['columns'];request=ns['requests'][id];family=ns['namespace']['family'];canonical=ns['namespace']['canonical']
            actual=[exact([None if v=='__WEFT_FIXTURE_NULL__' and col['nullable'] else canonical(v,family(col)) for v,col in zip(row,columns,strict=True)]) for row in report['rows']]
            expected=[exact([canonical(v,family(col)) for v,col in zip(row,columns,strict=True)]) for row in ns['namespace']['expected_rows'][id.rsplit('-',1)[0]]]
            cases.append({'id':id,'comparison':'ordered' if 'ORDER BY' in request['sql'].upper() else 'bag','expected':expected})
            observed.append({'id':id,'status':'passed','actual':actual})
        data=json.dumps({'status':'passed','layer':'native','profile':profile,'cases':observed},ensure_ascii=False).encode();path=Path(tmp)/f'{index}.json';path.write_bytes(data)
        claim={'status':'supported','profile':profile,'requiredLayers':['native'],'cases':cases,'evidence':[{'path':str(path),'sha256':hashlib.sha256(data).hexdigest()}]}
        result=audit(claim);results.append({'scope':scope,'audit':result,'reportSha256':hashlib.sha256(data).hexdigest()})
assert sum(r['audit']['cases'] for r in results)==76
print(json.dumps({'status':'passed','scopesAudited':len(results),'casesAudited':76,'hostArtifactJoins':hosts,'results':results,'qualification':'Real retained native report consistency and independent expected rows. Internal supported-claim inputs exercise the verifier only; candidate inventory is unchanged. Layout label is candidate, producer trust and final qualification remain unresolved. Host joins prove compiler artifact parity, not separate host database execution.'},indent=2))
