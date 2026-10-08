"""Independent recursive/presence expectations and native hidden-value refusals.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 @covers US-007-AC1 @covers US-007-AC2
"""
import hashlib,json,re,sys
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-values-native'
sys.path.insert(0,str(ROOT/'tests/ashlar-databricks'))
from warehouse_capture import capture
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
summary=strict((BASE/'summary.json').read_text());assert summary['state']=='passed' and summary['cases']==133
assert sha(BASE/'compound-native.py')==summary['harnessSha256']
warehouse=summary['warehouseIdentity']
assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
def unique(items,key):
 result={}
 for item in items:assert item[key] not in result;result[item[key]]=item
 return result
def lines(name):return [strict(l) for l in (BASE/name).read_text().splitlines() if l.strip()]
artifacts=unique(lines('compile-artifacts.jsonl'),'id');receipts=unique(lines('statements.jsonl'),'label');outcomes=unique(summary['outcomes'],'id');identities=unique(summary['warehouseIdentityCaptures'],'label')
assert len(receipts)==len({r['response']['statement_id'] for r in receipts.values()})
absent={'state':'absent'}
def present(value):return {'state':'value','value':value}
values={'integer-list':[str(-(1<<63)),str((1<<63)-1),'0'],'boolean-list':[False,True,False],
 'string-list':['é','e\u0301','x ',chr(34)+chr(92)+chr(10)],
 'decimal-list':['12345678901234567890123456.78','0.01','0.00'],
 'integer-map':{'é':'-128','e\u0301':'127','a.b'+chr(34)+chr(92):'0'},
 'nested-list':[['1','2'],[],['-128','127']],
 'structured':{'leaf':str((1<<63)-1),'note':present('é')},
 'cyclic':{'leaf':'7','note':absent,'next':present({'leaf':'-128','note':present('é'),'next':absent})},
 'optional-items':[present(False),present(True)],
 'map-structured':{'x':{'leaf':'7','note':absent}},'list-map':[{'x':'0.00'},{}]}
expected={};refused=set();suffixes={}
for shape,value in values.items():
 empty=({'leaf':'0','note':absent,'next':absent} if shape=='cyclic' else {'leaf':'0','note':absent} if shape=='structured' else {} if shape in ['integer-map','map-structured'] else [])
 for optional in [False,True]:
  for label in ['exact','empty','absent','explicit-null','wrong-root','hidden-bad-leaf']:
   id=f'{shape}-{optional}-{label}';suffixes[id]=label
   if label in ['explicit-null','wrong-root','hidden-bad-leaf'] or (label=='absent' and not optional):refused.add(id)
   else:expected[id]=[absent if label=='absent' else present(empty if label=='empty' else value)]
expected['empty-owner']=[];suffixes['empty-owner']='empty-owner'
assert len(expected)==56 and len(refused)==77 and set(artifacts)==set(outcomes)==set(expected)|refused
used=set();same=set();probes=set();checks=0

def rows(label):
 native=receipts[label]['response'];manifest=native['manifest'];result=native.get('result',{});data=result.get('data_array',[])
 assert native['status']['state']=='SUCCEEDED' and not manifest.get('truncated',False)
 assert manifest['total_row_count']==result.get('row_count',len(data))==len(data)
 assert result.get('row_offset',0)==result.get('chunk_index',0)==0
 assert manifest.get('total_chunk_count',1) in ([0,1] if not data else [1])
 return data

def observed(label,sql,params):
 used.add(label);receipt=receipts[label];assert receipt['sql']==capture(sql) and receipt['parameters']==params
 data=rows(label);identity=identities[label];assert identity['warehouse']==warehouse
 columns=receipt['response']['manifest']['schema']['columns'];assert (columns[0]['name'],columns[0]['type_name'])==('__weft_warehouse','STRING')
 if data:
  assert identity['method']=='same-statement' and all(strict(r[0])==warehouse for r in data);same.add(label)
 else:
  assert label=='empty-owner-user-query' and identity['method']=='separate-probe-after-empty-query'
  probe=label+'-empty-warehouse-probe';used.add(probe)
  assert receipts[probe]['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse" and receipts[probe]['parameters'] is None
  value=rows(probe);assert len(value)==1 and len(value[0])==1 and strict(value[0][0])==warehouse;probes.add(label)
 return [r[1:] for r in data]
for identifier,a in artifacts.items():
 request=a['request'];response=a['response'];assert response['status']=='compiled'
 assert request['sql']=='SELECT t.payload FROM Thing t'+(' WHERE t.id = 1' if suffixes[identifier]=='hidden-bad-leaf' else '')
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']==response['bindingSha256']
 assert len(response['columns'])==1 and response['columns'][0]['representation']['kind']=='value' and response['columns'][0]['representation']['nativeNull'] is False
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 guards=[check for o in response['obligations'] if o['id'] in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.compoundIntegrity'] for check in o['parameters']['checks']]
 counts=[]
 for index,check in enumerate(guards):
  value=observed(identifier+'-guard-'+str(index),check['sql'],params)
  assert len(value)==1 and len(value[0])==1 and re.fullmatch(r'[0-9]+',value[0][0]);counts.append(value[0][0]);checks+=1
 outcome=outcomes[identifier]
 if identifier in refused:
  assert outcome['outcome']=='refused-before-user-query' and counts==outcome['counts'] and any(int(v)>0 for v in counts)
  assert identifier+'-user-query' not in receipts
 else:
  assert outcome['outcome']=='published-fixture-result' and all(v=='0' for v in counts)
  label=identifier+'-user-query';actual=observed(label,response['sql'],params)
  assert actual==outcome['rows'] and [strict(r[0]) for r in actual]==expected[identifier]==outcome['expected'],identifier
  columns=receipts[label]['response']['manifest']['schema']['columns'][1:]
  assert [(c['name'],c['type_name']) for c in columns]==[(c['outputName'],'STRING') for c in response['columns']]
assert set(identities)==same|probes and len(probes)==1
assert set(receipts)-used=={'identity-node_type_a','identity-publication_manifest','resolve-fixture-vector'}
print(json.dumps(dict(status='passed',cases=133,positive=56,refusals=77,independentDecodedComparisons=56,integrityStatements=checks,sameStatementWarehouseResults=len(same),separateEmptyQueryProbes=1,terminalStatements=len(receipts),warehouseIdentity=warehouse,scope='Independent exact scalar/list/map/nested/structured/recursive values, optional root/item states, empty owners and hidden corrupt leaves across all eleven authored value shapes. No fresh execution or candidate promotion.'),indent=2))
