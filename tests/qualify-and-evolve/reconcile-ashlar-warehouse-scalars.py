"""Independent exact scalar SUM bags and native integrity-refusal custody.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3
"""
import hashlib,json,re,sys
from pathlib import Path
from collections import Counter
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-scalar-native'
sys.path.insert(0,str(ROOT/'tests/ashlar-databricks'))
from warehouse_capture import capture
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
summary=strict((BASE/'summary.json').read_text());assert summary['state']=='passed' and summary['cases']==10
assert sha(BASE/'scalar-native.py')==summary['harnessSha256']
warehouse=summary['warehouseIdentity']
assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
def unique(items,key):
 result={}
 for item in items:assert item[key] not in result;result[item[key]]=item
 return result
def lines(name):return [strict(l) for l in (BASE/name).read_text().splitlines() if l.strip()]
artifacts=unique(lines('compile-artifacts.jsonl'),'id');receipts=unique(lines('statements.jsonl'),'label');outcomes=unique(summary['outcomes'],'id');identities=unique(summary['warehouseIdentityCaptures'],'label')
ids={'exact','empty','hidden-foreign-domain','hidden-decimal-scale','hidden-decimal-domain','hidden-number-string','hidden-exponent-carrier','hidden-null','hidden-absent','hidden-wrong-name'}
assert set(artifacts)==set(outcomes)==ids and len(receipts)==len({r['response']['statement_id'] for r in receipts.values()})==46
used=set();same=set();probes=set();checks=0
SQL='SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name'
def bag(rows):return Counter(json.dumps(r,ensure_ascii=False) for r in rows)
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
  assert label=='empty-user-query' and identity['method']=='separate-probe-after-empty-query'
  probe=label+'-empty-warehouse-probe';used.add(probe)
  assert receipts[probe]['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse" and receipts[probe]['parameters'] is None
  value=rows(probe);assert len(value)==1 and len(value[0])==1 and strict(value[0][0])==warehouse;probes.add(label)
 return [r[1:] for r in data]
for identifier,a in artifacts.items():
 request=a['request'];response=a['response'];assert response['status']=='compiled' and request['sql']==SQL
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']==response['bindingSha256']
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 obligation=next(o for o in response['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')
 counts=[]
 for index,check in enumerate(obligation['parameters']['checks']):
  value=observed(identifier+'-integrity-'+str(index),check['sql'],params)
  assert len(value)==1 and len(value[0])==1 and re.fullmatch(r'[0-9]+',value[0][0]);counts.append(value);checks+=1
 outcome=outcomes[identifier]
 if identifier in ['exact','empty']:
  assert outcome['outcome']=='published-fixture-result' and all(v==[['0']] for v in counts)
  actual=observed(identifier+'-user-query',response['sql'],params)
  # Independently fixed exact totals: the duplicated unsigned-max customer joins twice.
  expected=[] if identifier=='empty' else [['é','200000000000000000000000000.02'],['e\u0301','-1.25'],['x ','2.00']]
  assert actual==outcome['rows'] and bag(actual)==bag(expected)
  columns=receipts[identifier+'-user-query']['response']['manifest']['schema']['columns'][1:]
  assert [(c['name'],c['type_name']) for c in columns]==[('name','STRING'),('total','STRING')]
 else:
  assert outcome['outcome']=='refused-before-user-query' and outcome['counts']==counts and any(int(v[0][0])>0 for v in counts)
  assert identifier+'-user-query' not in receipts
assert checks==40 and set(identities)==same|probes and len(same)==41 and len(probes)==1
assert set(receipts)-used=={'identity-object_current','identity-publication_manifest','resolve-fixture-vector'}
print(json.dumps(dict(status='passed',cases=10,positive=2,refusals=8,integrityStatements=40,sameStatementWarehouseResults=41,separateEmptyQueryProbes=1,terminalStatements=46,warehouseIdentity=warehouse,scope='Independent exact Decimal totals and unsigned-max duplicate joins, empty grouped result and eight hidden scalar integrity refusals. Empty result has separate warehouse probe; no fresh execution or candidate promotion.'),indent=2))
