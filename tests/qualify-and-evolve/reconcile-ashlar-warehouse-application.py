"""Reconcile current warehouse application receipts with independent bags/pages.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 @covers US-007-AC1 @covers US-007-AC2 @covers US-007-AC3
No fresh SQL execution, candidate promotion or production qualification.
"""
import hashlib,json,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-application-native'
sys.path.insert(0,str(ROOT/'tests/ashlar-databricks'))
from warehouse_capture import capture
from evidence_audit import strict
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
assert sha(BASE/'warehouse_capture.py')==sha(ROOT/'tests/ashlar-databricks/warehouse_capture.py')
summary=strict((BASE/'summary.json').read_text())
assert summary['state']=='passed' and summary['cases']==summary['warehouseApplicationCases']==112
assert sha(BASE/'application-native.py')==summary['harnessSha256']
assert sha(BASE/'native_transport.py')==summary['transportSha256']
warehouse=summary['warehouseIdentity']
assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
def unique(rows,key):
 result={}
 for row in rows:
  assert row[key] not in result,(key,row[key]);result[row[key]]=row
 return result
def lines(name):return [strict(line) for line in (BASE/name).read_text().splitlines() if line.strip()]
artifacts=unique(lines('compile-artifacts.jsonl'),'id')
statements=unique(lines('statements.jsonl'),'label')
assert len({s['response']['statement_id'] for s in statements.values()})==len(statements)
outcomes=unique(summary['outcomes'],'id')
identities=unique(summary['warehouseIdentityCaptures'],'id')
expected={};queries={}
for bits in [8,64]:
 values=[('same',-(1<<(bits-1))),('same',(1<<(bits-1))-1),('same',1),('é',3),('e\u0301',4),('x',5),('x ',6)]
 counts={}
 for name,_ in values:counts[name]=counts.get(name,0)+1
 for name_home in ['props','column']:
  for rank_home in ['props','column']:
   prefix=f'{bits}-{name_home}-{rank_home}'
   for kind,sql,rows in [
    ('count','SELECT COUNT(*) AS total FROM Thing t',[['7']]),
    ('join-count','SELECT COUNT(*) AS total FROM Thing t JOIN Thing u ON t.name = u.name',[[str(sum(a==b for a,_ in values for b,_ in values))]]),
    ('group-count','SELECT t.name, COUNT(*) AS total FROM Thing t GROUP BY t.name ORDER BY t.name ASC',[[n,str(c)] for n,c in sorted(counts.items())]),
    ('empty-count','SELECT COUNT(*) AS total FROM Thing t',[['0']])]:
    expected[prefix+'-'+kind]=rows;queries[prefix+'-'+kind]=sql
   for composite in [False,True]:
    ordered=sorted(values,key=(lambda r:r) if composite else (lambda r:r[1]))
    for page in range(5):
     identifier=prefix+('-composite' if composite else '-single')+'-page-'+str(page)
     expected[identifier]=[[name,str(rank),'true'] for name,rank in ordered[page*2:page*2+2]]
     sql='SELECT t.* FROM Thing t'
     if page:sql+=' WHERE (t.name, t.rank) > (:name, :rank)' if composite else ' WHERE t.rank > :rank'
     sql+=' ORDER BY t.name ASC, t.rank ASC LIMIT 2' if composite else ' ORDER BY t.rank ASC LIMIT 2'
     queries[identifier]=sql
assert len(expected)==112 and set(expected)==set(artifacts)==set(outcomes)==set(identities)
used=set();checks=0;same_statement=0;separate_probe=0
def rows(label):
 assert label not in used,label;used.add(label)
 receipt=statements[label];response=receipt['response'];assert response['status']['state']=='SUCCEEDED'
 manifest=response['manifest'];assert not manifest.get('truncated',False)
 result=response.get('result',{});data=result.get('data_array',[])
 assert result.get('chunk_index',0)==result.get('row_offset',0)==0
 assert result.get('row_count',len(data))==manifest['total_row_count']==len(data)
 assert manifest.get('total_chunk_count',1) in ([0,1] if not data else [1])
 return receipt,data
for identifier,artifact in artifacts.items():
 request=artifact['request'];response=artifact['response'];assert response['status']=='compiled'
 assert request['sql']==queries[identifier]
 assert response['modelPins']==[module['pin'] for module in request['modules']]
 for module in request['modules']:
  assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']==response['bindingSha256']
 parameters=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 query,data=rows(identifier+'-user-query')
 assert query['sql']==capture(response['sql']) and query['parameters']==parameters
 columns=query['response']['manifest']['schema']['columns']
 assert (columns[0]['name'],columns[0]['type_name'])==('__weft_warehouse','STRING')
 assert [(c['name'],c['type_name']) for c in columns[1:]]==[(c['outputName'],'STRING') for c in response['columns']]
 assert identities[identifier]['warehouse']==warehouse
 if data:
  assert identities[identifier]['method']=='same-statement'
  assert all(strict(row[0])==warehouse for row in data)
  same_statement+=1
 else:
  assert expected[identifier]==[] and identities[identifier]['method']=='separate-probe-after-empty-query'
  probe,observed=rows(identifier+'-empty-warehouse-probe')
  assert probe['sql']=="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_warehouse" and probe['parameters'] is None
  assert len(observed)==1 and len(observed[0])==1 and strict(observed[0][0])==warehouse
  separate_probe+=1
 assert [r[1:] for r in data]==expected[identifier]==outcomes[identifier]['rows'],identifier
 for obligation in response['obligations']:
  if obligation['id'] not in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.keyIntegrity']:continue
  for index,check in enumerate(obligation['parameters']['checks']):
   guard,observed=rows(identifier+'-'+obligation['id']+'-'+str(index))
   assert guard['sql']==check['sql'] and guard['parameters']==parameters and observed==[['0']]
   checks+=1
assert used==set(statements) and same_statement==96 and separate_probe==16
print(json.dumps(dict(status='passed',cases=112,independentRowComparisons=112,sameStatementWarehouseCases=same_statement,separateEmptyQueryProbes=separate_probe,integrityStatements=checks,terminalStatements=len(statements),warehouseIdentity=warehouse,scope='Independent COUNT bags/group order and full single/composite keyset page sequences across four typed/props homes and signed8/64. Empty pages have separately labeled warehouse probes; no same-statement identity claim for them, no fresh execution, host DB execution or candidate promotion.'),indent=2))
