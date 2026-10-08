"""@covers US-007-AC1 @covers US-007-AC2 @covers US-007-AC3
Saved native receipt custody reconciliation, not a new execution or result oracle.
"""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-006-application-native'
def lines(name):return [json.loads(x) for x in (BASE/name).read_text().splitlines() if x.strip()]
def unique(rows,key):
 result={}
 for row in rows:
  assert row[key] not in result,(key,row[key]);result[row[key]]=row
 return result
artifacts=unique(lines('compile-artifacts.jsonl'),'id')
statements=unique(lines('statements.jsonl'),'label')
summary=json.loads((BASE/'summary.json').read_text());assert summary['state']=='passed'
outcomes=unique(summary['outcomes'],'id')
assert len(artifacts)==len(outcomes)==summary['cases']==112
checks=0;records=[]
def rows(record):
 response=record['response'];assert response['status']['state']=='SUCCEEDED'
 manifest=response['manifest'];assert not manifest.get('truncated',False)
 result=response.get('result',{});data=result.get('data_array',[])
 assert result.get('chunk_index',0)==0 and result.get('row_offset',0)==0
 assert result.get('row_count',len(data))==len(data)
 assert manifest['total_row_count']==len(data)
 assert manifest.get('total_chunk_count',1) in ([0,1] if not data else [1])
 return data
for identifier,artifact in artifacts.items():
 response=artifact['response'];assert response['status']=='compiled'
 parameters=[{'name':'p'+str(p['position']),'type':'STRING','value':p['value']} for p in response['parameters']]
 query=statements[identifier+'-user-query']
 assert query['sql']==response['sql'] and query['parameters']==parameters
 assert rows(query)==outcomes[identifier]['rows']
 columns=query['response']['manifest']['schema']['columns']
 assert [(c['name'],c['type_name']) for c in columns]==[(c['outputName'],'STRING') for c in response['columns']]
 for obligation in response['obligations']:
  if obligation['id'] not in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.keyIntegrity']:continue
  for i,check in enumerate(obligation['parameters']['checks']):
   guard=statements[identifier+'-'+obligation['id']+'-'+str(i)]
   assert guard['sql']==check['sql'] and guard['parameters']==parameters
   assert rows(guard)==[['0']];checks+=1
 records.append({'id':identifier,'sqlSha256':hashlib.sha256(query['sql'].encode()).hexdigest(),'statementId':query['response']['statement_id'],'query':artifact['request']['sql']})
OUT=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-application-reconciliation';OUT.mkdir(exist_ok=True)
report={'status':'passed','cases':len(records),'integrityReceipts':checks,'scope':'Saved native SQL/parameter/terminal-result/metadata custody only. Independent expectations remain authored in the native harness; no fresh execution or production qualification.','sources':[{'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in [BASE/'compile-artifacts.jsonl',BASE/'statements.jsonl',BASE/'summary.json',ROOT/'tests/ashlar-databricks/application-native.py']],'casesReconciled':records}
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':report['status'],'cases':len(records),'integrityReceipts':checks}))
