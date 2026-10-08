"""@covers US-004-AC1 @covers US-004-AC3
Read-only synthetic owner substitutions; not publication/storage qualification.
"""
import copy,hashlib,json,os,subprocess
from pathlib import Path
from native_transport import Client,NativeFailure
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);client=Client(OUT)
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest()
source=ROOT/'docs/helix/04-build/evidence/B-006-unsigned-columns/compile-artifacts.jsonl'
base=next(json.loads(l)['request'] for l in source.read_text().splitlines() if json.loads(l)['id']=='unsigned-63')
results=[];artifacts=[]
for bits in [1,2,3,8,16,32,63,64]:
 request=copy.deepcopy(base);doc=json.loads(request['modules'][0]['documentJson'])
 doc['modules'][0]['elements'][1]['facets']['integerWidth']['bits']=bits
 raw=json.dumps(doc);request['modules'][0]['documentJson']=raw;request['modules'][0]['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
 binding=json.loads(request['target']['bindingJson']);binding['modelPins']=[request['modules'][0]['pin']]
 raw=json.dumps(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
 response=json.loads(subprocess.check_output([str(binary)],input=json.dumps(request).encode()))
 artifacts.append(dict(id='unsigned-'+str(bits),request=request,response=response))
 if bits==64:
  assert response['status']=='blocked' and response['diagnostics'][0]['code']=='WFT-BINDING' and 'sql' not in response
  results.append(dict(bits=bits,outcome='compile-refusal'));continue
 assert response['status']=='compiled'
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 table=binding['publication']['tables'][0]
 physical='.'.join('`'+x.replace('`','``')+'`' for x in table['name'])+' VERSION AS OF '+str(table['version'])
 system=binding['records'][0]['sourceSystem'];type_id=binding['records'][0]['typeId']
 def owner(values):
  return '('+' UNION ALL '.join("SELECT CAST("+str(v)+" AS BIGINT) AS id, '"+system.replace("'","''")+"' AS source_system, CAST("+type_id+" AS BIGINT) AS type_id" for v in values)+')'
 def substitute(sql,values):
  assert physical in sql
  return sql.replace(physical,owner(values))
 checks=next(o for o in response['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks'];assert len(checks)==1
 maximum=(1<<bits)-1;valid=[0,1,maximum]
 assert client.sql(f'{bits}-valid-guard',substitute(checks[0]['sql'],valid),params)==[['0']]
 observed=client.sql(f'{bits}-valid-sum',"SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_engine, observed.* FROM ("+substitute(response['sql'],valid)+") observed",params)
 assert len(observed)==1 and len(observed[0])==2 and observed[0][0],observed
 engine=observed[0][0];warehouse=json.loads(engine)
 assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None,warehouse
 assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash']),warehouse
 actual=[row[1:] for row in observed]
 assert actual==[[str(sum(valid))]]
 invalid=[-1,maximum+1] if bits<63 else [-1]
 assert client.sql(f'{bits}-invalid-guard',substitute(checks[0]['sql'],invalid),params)==[[str(len(invalid))]]
 result=dict(bits=bits,validValues=[str(v) for v in valid],expectedSum=str(sum(valid)),invalidValues=[str(v) for v in invalid],outcome='exact-boundaries-and-domain-refusal',sameStatementEngine=engine)
 if bits==63:
  try:client.sql('63-carrier-overflow',substitute(checks[0]['sql'],[maximum+1]),params)
  except NativeFailure as error:
   assert 'CAST_OVERFLOW' in str(error),str(error)
   result['carrierOverflow']='native CAST_OVERFLOW'
  else:raise AssertionError('BIGINT overflow accepted')
 results.append(result)
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha
(OUT/'compile-artifacts.jsonl').write_text('\n'.join(json.dumps(a) for a in artifacts)+'\n')
assert len({r['sameStatementEngine'] for r in results if 'sameStatementEngine' in r})==1
summary=dict(engineProbe='current_version',status='passed',cases=len(results),nativeStatements=len(client.records),outcomes=results,compilerSha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),scope='Read-only synthetic owner SQL substitutions retain current_version() and warehouse build hashes in the same statement as each admitted SUM and exercise emitted unsigned domain guards and exact aggregate at widths 1/2/3/8/16/32/63 boundaries. No actual table/publication custody qualification; UInt64 BIGINT compile-refuses.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
