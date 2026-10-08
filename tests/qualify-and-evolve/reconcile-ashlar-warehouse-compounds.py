"""Independent compound entity/page expectations plus exact native custody.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 @covers US-007-AC1 @covers US-007-AC2
No native SQL execution or profile promotion.
"""
import hashlib,json,sys
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-compound-native'
sys.path.insert(0,str(ROOT/'tests/ashlar-databricks'))
from warehouse_capture import capture
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
assert sha(BASE/'warehouse_capture.py')==sha(ROOT/'tests/ashlar-databricks/warehouse_capture.py')
summary=strict((BASE/'summary.json').read_text());assert summary['state']=='passed' and summary['cases']==48
assert sha(BASE/'compound-application-native.py')==summary['harnessSha256']
assert sha(BASE/'compound-native.py')==summary['builderSha256']
warehouse=summary['warehouseIdentity']
assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
def unique(items,key):
 result={}
 for item in items:assert item[key] not in result;result[item[key]]=item
 return result
def lines(name):return [strict(line) for line in (BASE/name).read_text().splitlines() if line.strip()]
artifacts=unique(lines('compile-artifacts.jsonl'),'id');receipts=unique(lines('statements.jsonl'),'label')
outcomes=unique(summary['outcomes'],'id');identities=unique(summary['warehouseIdentityCaptures'],'id')
assert len({r['response']['statement_id'] for r in receipts.values()})==len(receipts)
absent={'state':'absent'}
def present(value):return {'state':'value','value':value}
values={'nested-list':[['1','2'],[],['-128','127']],
 'structured':{'leaf':str((1<<63)-1),'note':present('é')},
 'cyclic':{'leaf':'7','note':absent,'next':present({'leaf':'-128','note':present('é'),'next':absent})},
 'map-structured':{'x':{'leaf':'7','note':absent}}}
expected={}
for shape,value in values.items():
 for optional,label in [(False,'exact'),(True,'exact'),(True,'absent')]:
  for home in ['props','column']:
   for page in [0,1]:
    identifier=f'{shape}-{optional}-{label}-{home}-page-{page}'
    expected[identifier]=[] if page else [['1',absent if label=='absent' else present(value)]]
assert len(expected)==48 and set(expected)==set(artifacts)==set(outcomes)==set(identities)
used=set();checks=0;same_statement=0;empty_probes=0
def rows(label):
 assert label not in used;used.add(label);receipt=receipts[label];response=receipt['response']
 assert response['status']['state']=='SUCCEEDED'
 manifest=response['manifest'];assert not manifest.get('truncated',False)
 result=response.get('result',{});data=result.get('data_array',[])
 assert result.get('chunk_index',0)==result.get('row_offset',0)==0
 assert result.get('row_count',len(data))==manifest['total_row_count']==len(data)
 assert manifest.get('total_chunk_count',1) in ([0,1] if not data else [1])
 return receipt,data
for identifier,a in artifacts.items():
 request=a['request'];response=a['response'];assert response['status']=='compiled'
 page=int(identifier[-1]);wanted_sql='SELECT t.* FROM Thing t'+(' WHERE t.id > :after' if page else '')+' ORDER BY t.id ASC LIMIT 1'
 assert request['sql']==wanted_sql
 if page:assert request['parameters']=={'after':{'family':'integer','value':'1'}}
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']==response['bindingSha256']
 assert response['columns'][1]['representation']['kind']=='value'
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 query,data=rows(identifier+'-query')
 assert query['sql']==capture(response['sql']) and query['parameters']==params
 columns=query['response']['manifest']['schema']['columns']
 assert [(c['name'],c['type_name']) for c in columns]==[('__weft_warehouse','STRING')]+[(c['outputName'],'STRING') for c in response['columns']]
 assert identities[identifier]['warehouse']==warehouse
 if data:
  assert identities[identifier]['method']=='same-statement' and all(strict(r[0])==warehouse for r in data)
  same_statement+=1
 else:
  assert page==1 and identities[identifier]['method']=='separate-probe-after-empty-query'
  probe,observed=rows(identifier+'-empty-warehouse-probe')
  assert probe['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse" and probe['parameters'] is None
  assert len(observed)==1 and len(observed[0])==1 and strict(observed[0][0])==warehouse
  empty_probes+=1
 actual=[r[1:] for r in data];assert actual==outcomes[identifier]['rows']
 decoded=[[r[0],strict(r[1])] for r in actual]
 assert decoded==expected[identifier]==outcomes[identifier]['expected'],identifier
 guards=[check for o in response['obligations'] if 'checks' in o['parameters'] for check in o['parameters']['checks']]
 for index,check in enumerate(guards):
  guard,observed=rows(identifier+'-guard-'+str(index))
  assert guard['sql']==check['sql'] and guard['parameters']==params and observed==[['0']]
  checks+=1
assert used==set(receipts) and same_statement==empty_probes==24 and checks==192
print(json.dumps(dict(status='passed',cases=48,independentDecodedComparisons=48,integrityStatements=checks,terminalStatements=len(receipts),sameStatementWarehouseCases=24,separateEmptyPageProbes=24,warehouseIdentity=warehouse,scope='Independent nested sequence, structured/recursive/map, exact integer and optional absent expectations with full page exhaustion across props/native ID homes. Empty pages use separate probes. No fresh execution, host DB execution or candidate promotion.'),indent=2))
