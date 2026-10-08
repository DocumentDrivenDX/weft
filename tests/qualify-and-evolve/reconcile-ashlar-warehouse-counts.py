"""Independent COUNT oracles plus native query/model/warehouse custody, no execution."""
import hashlib,importlib.util,json,os
from pathlib import Path
from collections import Counter
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2]
BASE=Path(os.environ.get('WEFT_ASHLAR_COUNT_RECONCILE_INPUT',str(ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-count-native')))
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert hashlib.sha256((BASE/name).read_bytes()).hexdigest()==digest
spec=importlib.util.spec_from_file_location('captured_projection',BASE/'warehouse_capture.py');capture=importlib.util.module_from_spec(spec);spec.loader.exec_module(capture)
summary=strict((BASE/'summary.json').read_text());assert summary['state']=='passed' and summary['cases']==summary['warehouseLinkedCountCases']==32
warehouse=summary['warehouseIdentity'];assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
def index(name,key):
 rows=[strict(line) for line in (BASE/name).read_text().splitlines()];result={r[key]:r for r in rows};assert len(result)==len(rows);return result
artifacts=index('compile-artifacts.jsonl','id');statements=index('statements.jsonl','label')
assert len(statements)==72 and len({r['response']['statement_id'] for r in statements.values()})==72
ids={f'{bits}-{name}-{rank}-{kind}' for bits in [8,64] for name in ['props','column'] for rank in ['props','column'] for kind in ['count','join-count','group-count','empty-count']}
assert set(artifacts)==ids
used=set();guards=0
names=['same','same','same','é','e\u0301','x','x ']
counts=Counter(names)
queries={'count':'SELECT COUNT(*) AS total FROM Thing t','join-count':'SELECT COUNT(*) AS total FROM Thing t JOIN Thing u ON t.name = u.name','group-count':'SELECT t.name, COUNT(*) AS total FROM Thing t GROUP BY t.name ORDER BY t.name ASC','empty-count':'SELECT COUNT(*) AS total FROM Thing t'}
for id,artifact in artifacts.items():
 kind=next(k for k in ['empty-count','group-count','join-count','count'] if id.endswith('-'+k))
 request=artifact['request'];response=artifact['response'];assert request['sql']==queries[kind] and response['status']=='compiled'
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256']==response['bindingSha256']
 pin=request['modules'][0]['pin']
 def column(position,name,family,element,copies=1):
  identity={'documentId':pin['documentId'],'revision':pin['revision'],'module':'main','element':element}
  return {'position':position,'outputName':name,'nullable':False,'representation':{'kind':'scalar','carrier':'text','decoder':'exact-integer' if family=='integer' else 'text','logicalType':{'family':family,'facets':{},'nullable':False}},'sourceIdentities':[identity for _ in range(copies)]}
 wanted_columns=([column(1,'name','string','name'),column(2,'total','integer','thing')] if kind=='group-count' else [column(1,'total','integer','thing',2 if kind=='join-count' else 1)])
 assert response['columns']==wanted_columns,'Logical result metadata changed'

 parameters=[{'name':'p'+str(p['position']),'type':'STRING','value':p['value']} for p in response['parameters']]
 query=statements[id+'-user-query'];used.add(id+'-user-query')
 assert query['sql']==capture.capture(response['sql']) and query['parameters']==parameters
 native=query['response'];assert native['status']['state']=='SUCCEEDED';manifest=native['manifest'];result=native['result'];rows=result['data_array']
 assert not manifest['truncated'] and manifest['total_chunk_count']==1 and manifest['total_row_count']==len(rows)==result['row_count'] and result['row_offset']==0
 columns=manifest['schema']['columns'];assert [(c['name'],c['type_name']) for c in columns]==[('__weft_warehouse','STRING')]+[(c['outputName'],'STRING') for c in response['columns']]
 assert rows and all(strict(row[0])==warehouse for row in rows)
 expected=([[name,str(count)] for name,count in sorted(counts.items())] if kind=='group-count' else [[str(sum(a==b for a in names for b in names))]] if kind=='join-count' else [['0']] if kind=='empty-count' else [['7']])
 assert [row[1:] for row in rows]==expected
 for obligation in response['obligations']:
  if obligation['id'] not in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.keyIntegrity']:continue
  for i,check in enumerate(obligation['parameters']['checks']):
   label=id+'-'+obligation['id']+'-'+str(i);used.add(label);guard=statements[label]
   assert guard['sql']==check['sql'] and guard['parameters']==parameters and guard['response']['status']['state']=='SUCCEEDED'
   gm=guard['response']['manifest'];gr=guard['response']['result']
   assert not gm['truncated'] and gm['total_row_count']==gr['row_count']==1 and gm['total_chunk_count']==1 and gr['row_offset']==0
   assert gr['data_array']==[['0']];guards+=1
assert used==set(statements) and guards==40
report={'status':'passed','cases':32,'integrityReceipts':40,'logicalMetadataCases':32,'warehouseIdentity':warehouse,'scope':'Saved native exact COUNT, duplicate join/group, Unicode comparison and empty-count results over four homes and signed8/64 with same-query warehouse builds. No fresh execution, entity/keyset, host or production qualification.'}
print(json.dumps(report))
