"""Reconcile saved unsigned receipts against emitted SQL and integer expectations.
@covers US-004-AC1 @covers US-004-AC3
No new native execution or table/publication qualification.
"""
import hashlib,json,os,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=pathlib.Path(os.environ.get('WEFT_UNSIGNED_RECONCILE_INPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-unsigned-boundaries-native')))
artifacts=[json.loads(l) for l in (BASE/'compile-artifacts.jsonl').read_text().splitlines()]
receipts=[json.loads(l) for l in (BASE/'statements.jsonl').read_text().splitlines()]
assert len(artifacts)==8 and len(receipts)==22
by_label={r['label']:r for r in receipts};assert len(by_label)==22
assert len({r['response']['statement_id'] for r in receipts})==22
engine_receipts=os.environ.get('WEFT_UNSIGNED_ENGINE_RECEIPTS')=='1'
warehouse_receipts=os.environ.get('WEFT_UNSIGNED_WAREHOUSE_RECEIPTS')=='1'
assert not warehouse_receipts or engine_receipts
engines={}
if engine_receipts:
 custody=json.loads((BASE/'custody.json').read_text())
 for name,digest in custody['inputHashes'].items():assert hashlib.sha256((BASE/name).read_bytes()).hexdigest()==digest
 summary=json.loads((BASE/'summary.json').read_text())
 engines={row['bits']:row['sameStatementEngine'] for row in summary['outcomes'] if 'sameStatementEngine' in row}
 assert set(engines)=={1,2,3,8,16,32,63} and len(set(engines.values()))==1 and all(isinstance(v,str) and v for v in engines.values())
 if warehouse_receipts:
  assert summary['engineProbe']=='current_version'
  warehouse=json.loads(next(iter(engines.values())))
  assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
  assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
used=set()
for artifact,bits in zip(artifacts,[1,2,3,8,16,32,63,64],strict=True):
 assert artifact['id']=='unsigned-'+str(bits)
 request=artifact['request'];response=artifact['response']
 module=request['modules'][0]
 assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 binding=json.loads(request['target']['bindingJson'])
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']
 assert binding['modelPins']==[module['pin']]
 assert json.loads(module['documentJson'])['modules'][0]['elements'][1]['facets']['integerWidth']['bits']==bits
 if bits==64:
  assert response['status']=='blocked' and response['diagnostics'][0]['code']=='WFT-BINDING' and 'sql' not in response
  continue
 assert response['status']=='compiled'
 table=binding['publication']['tables'][0]
 physical='.'.join('`'+x.replace('`','``')+'`' for x in table['name'])+' VERSION AS OF '+str(table['version'])
 system=binding['records'][0]['sourceSystem'];type_id=binding['records'][0]['typeId']
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 checks=next(o for o in response['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks'];assert len(checks)==1
 def check(label,sql,values,expected=None):
  assert label not in used;used.add(label);receipt=by_label[label]
  owner='('+' UNION ALL '.join("SELECT CAST("+str(v)+" AS BIGINT) AS id, '"+system.replace("'","''")+"' AS source_system, CAST("+type_id+" AS BIGINT) AS type_id" for v in values)+')'
  assert physical in sql
  wanted_sql=sql.replace(physical,owner)
  capture_engine=engine_receipts and label.endswith('-valid-sum')
  if capture_engine:
   probe="to_json(current_version(), map('ignoreNullFields','false'))" if warehouse_receipts else 'version()'
   wanted_sql='SELECT '+probe+' AS __weft_engine, observed.* FROM ('+wanted_sql+') observed'
  assert receipt['sql']==wanted_sql
  assert receipt['parameters']==params
  native=receipt['response']
  if expected is None:
   assert native['status']['state']=='FAILED' and 'CAST_OVERFLOW' in native['status']['error']['message']
  else:
   assert native['status']['state']=='SUCCEEDED'
   manifest=native['manifest'];result=native['result']
   assert manifest['truncated'] is False and manifest['total_row_count']==1 and manifest['total_chunk_count']==1
   width=2 if capture_engine else 1
   assert manifest['schema']['column_count']==width and all(c['type_name']=='STRING' for c in manifest['schema']['columns'])
   wanted_row=[engines[bits],str(expected)] if capture_engine else [str(expected)]
   assert result['row_count']==1 and result['row_offset']==0 and result['data_array']==[wanted_row]
 maximum=2**bits-1;valid=[0,1,maximum]
 check(f'{bits}-valid-guard',checks[0]['sql'],valid,0)
 check(f'{bits}-valid-sum',response['sql'],valid,2**bits)
 invalid=[-1,maximum+1] if bits<63 else [-1]
 check(f'{bits}-invalid-guard',checks[0]['sql'],invalid,len(invalid))
 if bits==63:check('63-carrier-overflow',checks[0]['sql'],[2**63])
assert used==set(by_label)
report={'status':'passed','cases':8,'nativeReceipts':22,'sameStatementEngineResults':len(engines),'warehouseIdentity':warehouse if warehouse_receipts else None,'successfulReceipts':21,'expectedFailedReceipts':1,'sourceSha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'inputHashes':{n:hashlib.sha256((BASE/n).read_bytes()).hexdigest() for n in ['compile-artifacts.jsonl','statements.jsonl']},'scope':'Saved synthetic unsigned boundary receipts reconciled to emitted SQL, exact parameters and independent integer expectations; no new engine execution or publication qualification.'}
OUT=pathlib.Path(os.environ.get('WEFT_UNSIGNED_RECONCILE_OUTPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-unsigned-reconciliation')));OUT.mkdir(parents=True,exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps(report))
