"""@covers US-004-AC1 @covers US-004-AC3
Read-only emitted-SQL owner substitutions for every UMF decimal domain.
No table writes, publication custody, host policy or support promotion.
"""
import copy, hashlib, json, os, subprocess
from pathlib import Path
from native_transport import Client

ROOT = Path(__file__).resolve().parents[2]
OUT = Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
OUT.mkdir(parents=True, exist_ok=True)
BINARY = Path(os.environ['WEFT_ASHLAR_COMPILER'])
BINARY_SHA = hashlib.sha256(BINARY.read_bytes()).hexdigest()
BASE = json.loads((ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-scalar-native/compile-artifacts.jsonl').read_text().splitlines()[0])['request']

def quoted(value):
    return "'"+value.replace("'","''")+"'"

def token(coefficient, scale):
    sign = '-' if coefficient < 0 else ''
    digits = str(abs(coefficient)).zfill(scale+1)
    return sign + (digits if scale == 0 else digits[:-scale]+'.'+digits[-scale:])

def prepare(precision, scale):
    request = copy.deepcopy(BASE)
    request['sql'] = 'SELECT SUM(o.total) AS total FROM Orders o'
    module = request['modules'][0]
    doc = json.loads(module['documentJson'])
    field = next(f for f in doc['modules'][0]['elements'] if f['id']=='order-total')
    field['facets'] = dict(precision=precision,scale=scale)
    module['documentJson'] = json.dumps(doc)
    module['pin']['sha256'] = hashlib.sha256(module['documentJson'].encode()).hexdigest()
    binding = json.loads(request['target']['bindingJson'])
    binding['modelPins'] = [module['pin']]
    request['target']['bindingJson'] = json.dumps(binding)
    request['target']['bindingSha256'] = hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()
    response = json.loads(subprocess.check_output([str(BINARY)],input=json.dumps(request).encode()))
    assert response['status']=='compiled', response
    record = next(r for r in binding['records'] if r['logical']['element']=='orders')
    table = binding['publication']['tables'][record['table']]
    physical = '.'.join('`'+part.replace('`','``')+'`' for part in table['name'])+' VERSION AS OF '+str(table['version'])
    def substitute(sql, payloads):
        owner = '('+' UNION ALL '.join(
            'SELECT '+str(i)+' AS id, '+quoted(record['sourceSystem'])+' AS source_system, CAST('+record['typeId']+' AS BIGINT) AS type_id, '+quoted(record['schemaRevision'])+' AS schema_revision, '+"'"+payload.replace("'","''")+"' AS props_json"
            for i,payload in enumerate(payloads))+')'
        assert physical in sql
        return sql.replace(physical,owner)
    maximum = 10**precision-1
    # Repeated extrema cancel exactly, leaving the least positive domain value.
    valid_coefficients = [-maximum,maximum,-maximum,maximum,1]
    valid = ['{"27":'+token(n,scale)+'}' for n in valid_coefficients]
    invalid = ['{"27":'+token(n,scale)+'}' for n in [-10**precision,10**precision]]
    invalid += ['{"27":'+token(1,scale+1)+'}', '{"27":null}', '{}']
    checks = next(o for o in response['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks']
    assert len(checks)==1
    params = [dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
    queries = [dict(id=f'{precision}-{scale}-'+label,sql=substitute(sql,payloads),expected=value)
        for label,sql,payloads,value in [
            ('valid-guard',checks[0]['sql'],valid,'0'),
            ('valid-sum',response['sql'],valid,token(1,scale)),
            ('invalid-guard',checks[0]['sql'],invalid,'5')]]
    return dict(id=f'decimal-{precision}-{scale}',precision=precision,scale=scale,request=request,response=response,validPayloads=valid,invalidPayloads=invalid,queries=queries,parameters=params)

artifacts = [prepare(p,s) for p in range(1,29) for s in range(p+1)]
assert len(artifacts)==434
assert all(a['parameters']==artifacts[0]['parameters'] for a in artifacts)
(OUT/'compile-artifacts.jsonl').write_text('\n'.join(json.dumps(a) for a in artifacts)+'\n')
if os.environ.get('WEFT_DECIMAL_PREPARE_ONLY')=='1':
    print(json.dumps(dict(status='prepared',cases=len(artifacts),compilerSha256=BINARY_SHA)))
else:
    client = Client(OUT)
    probe="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS warehouse"
    before=client.sql('warehouse-before-settings',probe)
    assert client.sql('read-ansi-mode','SET ansi_mode')==[['ansi_mode','true']]
    after=client.sql('warehouse-after-settings',probe)
    assert before==after and len(before)==1 and len(before[0])==1
    admitted_warehouse=json.loads(before[0][0])
    assert admitted_warehouse==dict(dbr_version=None,dbsql_version='2026.39',u_build_hash='15b447529a1f55ca1ad8c73a89b8d196f6ed4904',r_build_hash='3909d148af5cb6b560624c9255f5a727f7a17a4c')
    results=[]
    for start in range(0,len(artifacts),8):
        batch=artifacts[start:start+8]
        parts=[];expected={}
        for artifact in batch:
            for q in artifact['queries']:
                parts.append("SELECT '"+q['id']+"' AS case_id, observed.* FROM ("+q['sql']+") observed")
                expected[q['id']]=q['expected']
        # Same native statement supplies engine identity to every observed case.
        sql="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_engine, cases.* FROM ("+' UNION ALL '.join(parts)+') cases'
        rows=client.sql(f'decimal-batch-{start//8}',sql,artifacts[0]['parameters'])
        assert len(rows)==len(expected) and all(len(r)==3 for r in rows)
        actual={r[1]:r[2] for r in rows};assert len(actual)==len(rows)
        # SUM's exact STRING transport may pad its declared scale; tokens already do.
        assert actual==expected,(start,actual,expected)
        engines={r[0] for r in rows};assert len(engines)==1
        warehouse=json.loads(next(iter(engines)))
        assert warehouse==admitted_warehouse
        results.append(dict(label=f'decimal-batch-{start//8}',cases=list(expected),warehouse=warehouse))
        print(f'completed decimal domains {min(start+8,len(artifacts))}/434',flush=True)
    assert len({json.dumps(r['warehouse'],sort_keys=True) for r in results})==1
    assert hashlib.sha256(BINARY.read_bytes()).hexdigest()==BINARY_SHA
    summary=dict(observedAnsiMode=True,settingsObservation='Separate read-only probe between matching builds, not inferred for arbitrary hosts',status='passed',cases=434,nativeAssertions=1302,nativeStatements=len(client.records),batches=results,compilerSha256=BINARY_SHA,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),scope='All precision 1..28 and scale 0..precision. Exact repeated-extrema SUM, valid guards, precision/scale/null/absent refusals on props JSON; same-statement warehouse identity. Synthetic read-only owner substitutions, not table/publication or supported registration qualification.')
    (OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
    print(json.dumps(summary))
