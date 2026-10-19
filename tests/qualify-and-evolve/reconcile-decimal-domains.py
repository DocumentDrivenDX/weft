"""Independent all-decimal-domain native receipt reconciliation.
@covers US-004-AC1 @covers US-004-AC3
"""
import gzip,hashlib,json,os
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2]
BASE=Path(os.environ.get('WEFT_DECIMAL_RECONCILE_INPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-decimal-domains-native')))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
summary=strict((BASE/'summary.json').read_text())
assert summary['status']=='passed' and summary['cases']==434 and summary['nativeAssertions']==1302
assert summary['nativeStatements']==58 and summary['observedAnsiMode'] is True
assert summary['harnessSha256']==sha(BASE/'harness.py')
source_inputs=strict((BASE/'source-inputs.json').read_text())
assert source_inputs['compilerSha256']==summary['compilerSha256']
assert source_inputs['command']=='cargo build -p weft-databricks --example compile --locked'
def lines(name):
    path=BASE/(name+'.gz')
    payload=gzip.decompress(path.read_bytes())
    assert len(payload)==custody['archives'][name]['bytes']
    assert hashlib.sha256(payload).hexdigest()==custody['archives'][name]['uncompressedSha256']
    return [strict(line) for line in payload.decode('utf-8').splitlines()]
artifacts=lines('compile-artifacts.jsonl')
receipts=lines('statements.jsonl')
assert len(artifacts)==434 and len(receipts)==58
index={r['label']:r for r in receipts};assert len(index)==len(receipts)
assert len({r['response']['statement_id'] for r in receipts})==58
warehouse=dict(dbr_version=None,dbsql_version='2026.39',u_build_hash='15b447529a1f55ca1ad8c73a89b8d196f6ed4904',r_build_hash='3909d148af5cb6b560624c9255f5a727f7a17a4c')
def native_rows(receipt):
    native=receipt['response'];assert native['status']['state']=='SUCCEEDED'
    manifest=native['manifest'];result=native['result']
    assert manifest['truncated'] is False and manifest['total_chunk_count']==1
    assert manifest['total_row_count']==result['row_count']==len(result['data_array'])
    assert result['row_offset']==0 and result['chunk_index']==0
    assert all(c['type_name']=='STRING' for c in manifest['schema']['columns'])
    return result['data_array']
probe="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS warehouse"
for label in ['warehouse-before-settings','warehouse-after-settings']:
    assert index[label]['sql']==probe and index[label]['parameters'] is None
    rows=native_rows(index[label]);assert len(rows)==1 and len(rows[0])==1 and strict(rows[0][0])==warehouse
assert index['read-ansi-mode']['sql']=='SET ansi_mode' and index['read-ansi-mode']['parameters'] is None
assert native_rows(index['read-ansi-mode'])==[['ansi_mode','true']]
def decimal(n,s):
    # Independent base-ten integer coefficient oracle; no compiler/harness import.
    whole,remainder=divmod(abs(n),10**s)
    return ('-' if n<0 else '')+str(whole)+(('.'+str(remainder).zfill(s)) if s else '')
def quote(s):return "'"+s.replace("'","''")+"'"
queries=[];shared_params=None
for artifact,(p,s) in zip(artifacts,[(p,s) for p in range(1,29) for s in range(p+1)],strict=True):
    assert artifact['id']==f'decimal-{p}-{s}' and artifact['precision']==p and artifact['scale']==s
    request=artifact['request'];response=artifact['response'];module=request['modules'][0]
    assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
    doc=strict(module['documentJson']);field=next(e for e in doc['modules'][0]['elements'] if e['id']=='order-total')
    assert field['facets']==dict(precision=p,scale=s)
    assert request['sql']=='SELECT SUM(o.total) AS total FROM Orders o'
    binding=strict(request['target']['bindingJson'])
    assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']
    assert binding['modelPins']==response['modelPins']==[module['pin']]
    assert response['status']=='compiled' and response['bindingSha256']==request['target']['bindingSha256']
    record=next(r for r in binding['records'] if r['logical']['element']=='orders')
    home=next(prop['home'] for prop in record['properties'] if prop['logical']['element']=='order-total')
    assert home==dict(kind='props',propertyId='27')
    assert response['columns'][0]['decoder']=='exact-decimal' and response['columns'][0]['logicalType']['facets']==dict(scale=s)
    table=binding['publication']['tables'][record['table']]
    physical='.'.join('`'+part.replace('`','``')+'`' for part in table['name'])+' VERSION AS OF '+str(table['version'])
    params=[dict(name='p'+str(x['position']),type='STRING',value=x['value']) for x in response['parameters']]
    assert artifact['parameters']==params
    if shared_params is None:shared_params=params
    assert shared_params==params
    max_coefficient=10**p-1
    valid=['{"27":'+decimal(n,s)+'}' for n in [-max_coefficient,max_coefficient,-max_coefficient,max_coefficient,1]]
    invalid=['{"27":'+decimal(n,s)+'}' for n in [-10**p,10**p]]+['{"27":'+decimal(1,s+1)+'}','{"27":null}','{}']
    assert artifact['validPayloads']==valid and artifact['invalidPayloads']==invalid
    checks=next(o for o in response['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks'];assert len(checks)==1
    def substitute(sql,payloads):
        assert physical in sql
        owner='('+' UNION ALL '.join('SELECT '+str(i)+' AS id, '+quote(record['sourceSystem'])+' AS source_system, CAST('+record['typeId']+' AS BIGINT) AS type_id, '+quote(record['schemaRevision'])+' AS schema_revision, '+quote(payload)+' AS props_json' for i,payload in enumerate(payloads))+')'
        return sql.replace(physical,owner)
    expected=[dict(id=f'{p}-{s}-'+label,sql=substitute(sql,payloads),expected=value) for label,sql,payloads,value in [('valid-guard',checks[0]['sql'],valid,'0'),('valid-sum',response['sql'],valid,decimal(1,s)),('invalid-guard',checks[0]['sql'],invalid,'5')]]
    assert artifact['queries']==expected
    queries+=expected
used={'warehouse-before-settings','warehouse-after-settings','read-ansi-mode'}
for start in range(0,434,8):
    label=f'decimal-batch-{start//8}';receipt=index[label];used.add(label)
    batch=queries[start*3:min(start+8,434)*3]
    parts=["SELECT '"+q['id']+"' AS case_id, observed.* FROM ("+q['sql']+") observed" for q in batch]
    sql="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_engine, cases.* FROM ("+' UNION ALL '.join(parts)+') cases'
    assert receipt['sql']==sql and receipt['parameters']==shared_params
    rows=native_rows(receipt);assert len(rows)==len(batch)
    assert all(len(row)==3 and strict(row[0])==warehouse for row in rows)
    actual={row[1]:row[2] for row in rows};assert len(actual)==len(rows)
    assert actual=={q['id']:q['expected'] for q in batch}
assert used==set(index)
report=dict(status='passed',cases=434,nativeAssertions=1302,nativeReceipts=58,warehouseIdentity=warehouse,sourceSha256=sha(Path(__file__)),inputHashes={n:sha(BASE/n) for n in ['custody.json','summary.json','compile-artifacts.jsonl.gz','statements.jsonl.gz']},scope='All decimal precision 1..28 and valid scales; exact emitted SQL/params/pins, duplicate extrema and least-positive coefficient, five invalid values per domain, and same-statement warehouse build. Separate read-only ANSI observation. Synthetic owners, no production custody/support promotion.')
OUT=Path(os.environ.get('WEFT_DECIMAL_RECONCILE_OUTPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-decimal-domains-reconciliation')));OUT.mkdir(parents=True,exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
