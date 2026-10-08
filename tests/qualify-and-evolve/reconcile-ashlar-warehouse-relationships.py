"""Independent native relationship bags/refusals and current warehouse custody.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 @covers US-007-AC4
"""
import hashlib,json,re,sys
from pathlib import Path
from collections import Counter
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-relationship-native'
sys.path.insert(0,str(ROOT/'tests/ashlar-databricks'))
from warehouse_capture import capture
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
assert sha(BASE/'warehouse_capture.py')==sha(ROOT/'tests/ashlar-databricks/warehouse_capture.py')
summary=strict((BASE/'summary.json').read_text());assert summary['state']=='passed' and summary['cases']==52
assert sha(BASE/'relationship-native.py')==summary['harnessSha256']
warehouse=summary['warehouseIdentity']
assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
def unique(items,key):
 result={}
 for item in items:assert item[key] not in result;result[item[key]]=item
 return result
def lines(name):return [strict(line) for line in (BASE/name).read_text().splitlines() if line.strip()]
artifacts=unique(lines('compile-artifacts.jsonl'),'id');receipts=unique(lines('statements.jsonl'),'label')
outcomes=unique(summary['outcomes'],'id');identities=unique(summary['warehouseIdentityCaptures'],'label')
assert len({r['response']['statement_id'] for r in receipts.values()})==len(receipts)
def bag(rows):return Counter(json.dumps(r,sort_keys=True,ensure_ascii=False,separators=(',',':')) for r in rows)
customers=[(1,10),(2,20),(3,30)];orders=[(101,'é',-128),(102,'x ',127),(103,'é',2),(104,'e\u0301',3)]
edges=[(1,1,101),(2,1,101),(3,1,102),(4,2,103)]
positive_kinds=['forward-1','forward-2','forward-3','forward-10','inverse','exists-é','exists-absent']
negative_kinds=['orphan','wrong-endpoint','duplicate-key','duplicate-edge','maximum','minimum']
ids={f'{edge}-{home}-{kind}' for edge in ['edge_current','edge_ab'] for home in ['props','column'] for kind in positive_kinds+negative_kinds}
assert len(ids)==52 and ids==set(artifacts)==set(outcomes)
used=set();guards=set();positive=refused=0;same=set();probes=set()
def native_rows(label):
 response=receipts[label]['response'];assert response['status']['state']=='SUCCEEDED'
 manifest=response['manifest'];assert not manifest.get('truncated',False)
 result=response.get('result',{});data=result.get('data_array',[])
 assert result.get('chunk_index',0)==result.get('row_offset',0)==0
 assert result.get('row_count',len(data))==manifest['total_row_count']==len(data)
 assert manifest.get('total_chunk_count',1) in ([0,1] if not data else [1])
 return data
def observed(label,sql,params):
 receipt=receipts[label];used.add(label)
 assert receipt['sql']==capture(sql) and receipt['parameters']==params
 data=native_rows(label);columns=receipt['response']['manifest']['schema']['columns']
 assert (columns[0]['name'],columns[0]['type_name'])==('__weft_warehouse','STRING')
 identity=identities[label];assert identity['warehouse']==warehouse
 if data:
  assert identity['method']=='same-statement' and all(strict(r[0])==warehouse for r in data)
  same.add(label)
 else:
  assert identity['method']=='separate-probe-after-empty-query'
  probe_label=label+'-empty-warehouse-probe';probe=receipts[probe_label];used.add(probe_label)
  assert probe['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse" and probe['parameters'] is None
  result=native_rows(probe_label);assert len(result)==1 and len(result[0])==1 and strict(result[0][0])==warehouse
  probes.add(label)
 return [r[1:] for r in data]
for identifier,a in artifacts.items():
 request=a['request'];response=a['response'];assert response['status']=='compiled'
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']==response['bindingSha256']
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 checks=[check for o in response['obligations'] if 'checks' in o['parameters'] for check in o['parameters']['checks']]
 outcome=outcomes[identifier];assert len(checks)==len(outcome['guardReceipts'])
 counts=[]
 for check,label in zip(checks,outcome['guardReceipts']):
  names=set(re.findall(r':(p[0-9]+)\b',check['sql']))
  data=observed(label,check['sql'],[p for p in params if p['name'] in names]);guards.add(label)
  assert len(data)==1 and len(data[0])==1 and isinstance(data[0][0],str) and re.fullmatch(r'[0-9]+',data[0][0])
  counts.append(data[0][0])
 kind=next(k for k in positive_kinds+negative_kinds if identifier.endswith('-'+k))
 label=identifier+'-user-query'
 if kind in negative_kinds:
  assert outcome['outcome']=='refused-before-user-query' and counts==outcome['counts'] and any(int(v)>0 for v in counts) and label not in receipts
  refused+=1;continue
 assert outcome['outcome']=='published-fixture-result' and all(v=='0' for v in counts)
 actual=observed(label,response['sql'],params);assert actual==outcome['rows']
 columns=receipts[label]['response']['manifest']['schema']['columns'][1:]
 assert [(c['name'],c['type_name']) for c in columns]==[(c['outputName'],'STRING') for c in response['columns']]
 if kind.startswith('exists-'):
  expected=[['10']] if kind=='exists-é' else []
  assert 'HAS_RELATED' in request['sql'];decoded=actual
 else:
  assert response['columns'][-1]['representation']['kind']=='relatedKeys'
  bound=2 if kind=='inverse' else int(kind.split('-')[1]);expected=[]
  if kind=='inverse':
   for id,code,rank in orders:
    tuples=sorted([[str(next(k for i,k in customers if i==src))] for _,src,dst in edges if dst==id],key=lambda k:int(k[0]))
    expected.append([code,str(rank),dict(items=tuples[:bound],truncated=len(tuples)>bound)])
  else:
   for id,key in customers:
    tuples=sorted([[code,str(rank)] for _,src,dst in edges if src==id for oid,code,rank in orders if oid==dst],key=lambda k:(k[0],int(k[1])))
    expected.append([str(key),dict(items=tuples[:bound],truncated=len(tuples)>bound)])
  decoded=[r[:-1]+[strict(r[-1])] for r in actual]
 assert bag(decoded)==bag(expected)==bag(outcome['expected']),identifier
 positive+=1
assert positive==summary['positive']==28 and refused==summary['refusals']==24
assert len(guards)==summary['uniqueNativeGuards']==272
assert set(identities)==same|probes and len(same)==296 and len(probes)==4
# The remaining receipts resolve the pinned fixture UUIDs and manifest vector.
setup=set(receipts)-used
assert setup=={'identity-node_type_a','identity-edge_current','identity-edge_ab','identity-publication_manifest','resolve-fixture-vector'}
for label in setup:assert native_rows(label)
print(json.dumps(dict(status='passed',cases=52,positive=positive,refusals=refused,uniqueNativeGuards=len(guards),sameStatementWarehouseResults=len(same),separateEmptyQueryProbes=len(probes),warehouseIdentity=warehouse,terminalStatements=len(receipts),scope='Independent forward/inverse duplicate bags, bound lookahead, composite-key EXISTS and pre-query integrity refusals across canonical/serving edges and props/typed homes. Empty EXISTS uses separate probes. No fresh SQL execution, host DB execution or candidate promotion.'),indent=2))
