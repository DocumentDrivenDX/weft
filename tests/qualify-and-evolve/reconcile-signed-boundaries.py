"""Independent signed-width native receipt reconciliation; no support promotion.
@covers US-004-AC1 @covers US-004-AC3
"""
import hashlib,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=Path(os.environ.get('WEFT_SIGNED_RECONCILE_INPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-signed-all-widths-native')))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=json.loads((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
summary=json.loads((BASE/'summary.json').read_text())
assert summary['status']=='passed' and summary['cases']==64 and summary['nativeStatements']==194
assert summary['engineProbe']=='current_version' and summary['harnessSha256']==sha(BASE/'harness.py')
artifacts=[json.loads(l) for l in (BASE/'compile-artifacts.jsonl').read_text().splitlines()]
receipts=[json.loads(l) for l in (BASE/'statements.jsonl').read_text().splitlines()]
assert len(artifacts)==64 and len(receipts)==194
index={r['label']:r for r in receipts};assert len(index)==194
assert len({r['response']['statement_id'] for r in receipts})==194
engines={r['bits']:r['sameStatementEngine'] for r in summary['outcomes']}
assert set(engines)==set(range(1,65)) and len(set(engines.values()))==1
warehouse=json.loads(engines[1]);assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
used=set()
for artifact,bits in zip(artifacts,range(1,65),strict=True):
 assert artifact['id']=='signed-'+str(bits)
 request=artifact['request'];response=artifact['response'];module=request['modules'][0]
 assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert json.loads(module['documentJson'])['modules'][0]['elements'][1]['facets']['integerWidth']=={'bits':bits,'signed':True}
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']
 binding=json.loads(request['target']['bindingJson']);assert binding['modelPins']==[module['pin']]
 assert response['status']=='compiled' and response['modelPins']==[module['pin']] and response['bindingSha256']==request['target']['bindingSha256']
 table=binding['publication']['tables'][0]
 physical='.'.join('`'+part.replace('`','``')+'`' for part in table['name'])+' VERSION AS OF '+str(table['version'])
 system=binding['records'][0]['sourceSystem'];type_id=binding['records'][0]['typeId']
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 checks=next(o for o in response['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks'];assert len(checks)==1
 def check(label,sql,values,expected):
  assert label not in used;used.add(label);receipt=index[label]
  owner='('+' UNION ALL '.join('SELECT CAST('+('NULL' if v is None else str(v))+" AS BIGINT) AS id, '"+system.replace("'","''")+"' AS source_system, CAST("+type_id+' AS BIGINT) AS type_id' for v in values)+')'
  assert physical in sql
  wanted=sql.replace(physical,owner)
  capture=label.endswith('-valid-sum')
  if capture:wanted="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_engine, observed.* FROM ("+wanted+') observed'
  assert receipt['sql']==wanted and receipt['parameters']==params,label
  native=receipt['response']
  if expected is None:
   assert native['status']['state']=='FAILED' and 'CAST_OVERFLOW' in native['status']['error']['message'],label
  else:
   assert native['status']['state']=='SUCCEEDED',label
   manifest=native['manifest'];result=native['result'];width=2 if capture else 1
   assert manifest['truncated'] is False and manifest['total_row_count']==manifest['total_chunk_count']==1
   assert manifest['schema']['column_count']==width and all(c['type_name']=='STRING' for c in manifest['schema']['columns'])
   assert result['row_count']==1 and result['row_offset']==0 and result['data_array']==[([engines[bits],str(expected)] if capture else [str(expected)])],label
 low=-2**(bits-1);high=2**(bits-1)-1
 check(f'{bits}-valid-guard',checks[0]['sql'],[low,high,low,high],0)
 # Each min/max pair sums to -1. Repeated rows must preserve the bag and yield -2.
 check(f'{bits}-valid-sum',response['sql'],[low,high,low,high],-2)
 check(f'{bits}-invalid-guard',checks[0]['sql'],[low-1,high+1,None] if bits<64 else [None],3 if bits<64 else 1)
 if bits==64:
  check('64-carrier-overflow-below',checks[0]['sql'],[-2**63-1],None)
  check('64-carrier-overflow-above',checks[0]['sql'],[2**63],None)
assert used==set(index)
report=dict(status='passed',cases=64,nativeReceipts=194,successfulReceipts=192,expectedFailedReceipts=2,sameStatementWarehouseCases=64,warehouseIdentity=warehouse,sourceSha256=sha(Path(__file__)),inputHashes={n:sha(BASE/n) for n in ['custody.json','summary.json','compile-artifacts.jsonl','statements.jsonl']},scope='Exact signed widths 1..64, repeated signed boundary bags, range/null guards and both carrier overflow refusals. Original emitted SQL, parameters, module/binding pins and warehouse capture reconciled. Synthetic owner substitutions; no publication custody or supported registration claim.')
OUT=Path(os.environ.get('WEFT_SIGNED_RECONCILE_OUTPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-signed-all-widths-reconciliation')));OUT.mkdir(parents=True,exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
