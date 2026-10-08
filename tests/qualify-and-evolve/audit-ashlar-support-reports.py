"""Audit actual warehouse-linked results; internal claims do not promote candidates.
@covers US-006-AC1 @covers US-004-AC1 @covers US-004-AC3
"""
import contextlib,hashlib,io,json,runpy,tempfile
from pathlib import Path
from evidence_audit import audit,strict
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-boundaries-native'
with contextlib.redirect_stdout(io.StringIO()):
    runpy.run_path(str(Path(__file__).with_name('reconcile-ashlar-warehouse-boundaries.py')))
summary=strict((BASE/'summary.json').read_text())
warehouse=strict(next(r['sameStatementEngine'] for r in summary['outcomes'] if 'sameStatementEngine' in r))
artifacts=[strict(line) for line in (BASE/'compile-artifacts.jsonl').read_text().splitlines()]
receipts={r['label']:r for r in (strict(line) for line in (BASE/'statements.jsonl').read_text().splitlines())}
results=[]
with tempfile.TemporaryDirectory() as folder:
    for artifact,bits in zip(artifacts,[1,2,3,8,16,32,63,64],strict=True):
        if bits==64:continue # Explicit compile refusal; not native support evidence.
        request=artifact['request'];response=artifact['response'];context=response['targetContext']
        assert len(response['modelPins'])==1 and 'COLLATE UTF8_BINARY' in response['sql']
        profile={'compilerVersion':response['compilerVersion'],'dialect':response['dialect'],'irVersion':response['logicalPlan']['irVersion'],'backendVersion':response['backend']['backendVersion'],'targetProfile':response['backend']['targetProfile'],'engineVersion':'Databricks SQL '+warehouse['dbsql_version'],'layoutRevision':context['storageLayoutRevision'],'modelSha256':response['modelPins'][0]['sha256'],'bindingSha256':response['bindingSha256'],'settings':{'comparison':'explicit UTF8_BINARY in emitted SQL','resultTransport':'JSON_ARRAY exact STRING','warehouseBuild':{'u':warehouse['u_build_hash'],'r':warehouse['r_build_hash']},'scope':'Synthetic unsigned owner substitutions; session ANSI setting not inferred.'}}
        cases=[];observed=[]
        for suffix,expected in [('valid-guard','0'),('valid-sum',str(2**bits)),('invalid-guard',str(2 if bits<63 else 1))]:
            id=f'{bits}-{suffix}';record=receipts[id];actual=record['response']['result']['data_array']
            if suffix=='valid-sum':
                assert strict(actual[0][0])==warehouse
                actual=[row[1:] for row in actual]
            cases.append({'id':id,'comparison':'ordered','expected':[[expected]]})
            observed.append({'id':id,'status':'passed','actual':actual})
        if bits==63:
            record=receipts['63-carrier-overflow'];assert record['response']['status']['state']=='FAILED'
            assert 'CAST_OVERFLOW' in record['response']['status']['error']['message']
            refusal=[[{'expectedNativeRefusal':'CAST_OVERFLOW'}]]
            cases.append({'id':'63-carrier-overflow','comparison':'ordered','expected':refusal})
            observed.append({'id':'63-carrier-overflow','status':'passed','actual':refusal})
        raw=json.dumps({'status':'passed','layer':'native','profile':profile,'cases':observed},ensure_ascii=False).encode()
        path=Path(folder)/f'{bits}.json';path.write_bytes(raw)
        claim={'status':'supported','profile':profile,'requiredLayers':['native'],'cases':cases,'evidence':[{'path':str(path),'sha256':hashlib.sha256(raw).hexdigest()}]}
        result=audit(claim)
        results.append({'bits':bits,'profile':profile,'audit':result,'reportSha256':hashlib.sha256(raw).hexdigest()})
assert len(results)==7 and sum(r['audit']['cases'] for r in results)==22
print(json.dumps({'status':'passed','scopesAudited':7,'casesAudited':22,'successfulNativeStatements':21,'expectedNativeFailures':1,'results':results,'sourceHashes':{name:hashlib.sha256((BASE/name).read_bytes()).hexdigest() for name in ['compile-artifacts.jsonl','statements.jsonl','custody.json']},'qualification':'Real retained report consistency with independent exact unsigned expectations and warehouse builds. Supported-claim inputs exercise the verifier only. Candidate inventory unchanged; no complete backend domain, publication, host execution or production qualification.'},indent=2))
