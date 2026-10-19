"""@covers US-007-AC1 @covers US-007-AC2
Reconcile saved accepted native receipts and independently authored compound values.
"""
import hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-006-compound-application-native'
def unique(items,key):
 d={}
 for item in items:assert item[key] not in d;d[item[key]]=item
 return d
def loadlines(name):return [json.loads(l) for l in (BASE/name).read_text().splitlines()]
def pairs(items):
 d={}
 for k,v in items:assert k not in d;d[k]=v
 return d
def strict(raw):
 def invalid(value):raise AssertionError(value)
 return json.loads(raw,object_pairs_hook=pairs,parse_constant=invalid)
def canonical(v):return json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':'))
def rows(r):
 response=r['response'];assert response['status']['state']=='SUCCEEDED'
 m=response['manifest'];assert not m.get('truncated',False)
 result=response.get('result',{});data=result.get('data_array',[])
 assert result.get('row_count',len(data))==len(data)==m['total_row_count']
 assert result.get('chunk_index',0)==0 and result.get('row_offset',0)==0
 assert m.get('total_chunk_count',1) in ([0,1] if not data else [1])
 return data
custody=json.loads((BASE/'custody.json').read_text())
accepted=BASE/'accepted-statements.jsonl'
assert hashlib.sha256(accepted.read_bytes()).hexdigest()==custody['acceptedSha256']
statements=unique(loadlines('accepted-statements.jsonl'),'label')
assert len(statements)==custody['statements']==240
artifacts=unique(loadlines('compile-artifacts.jsonl'),'id')
summary=json.loads((BASE/'summary.json').read_text());assert summary['state']=='passed'
outcomes=unique(summary['outcomes'],'id');assert set(artifacts)==set(outcomes) and len(artifacts)==48
checks=0;used=set();report=[]
for identifier,a in artifacts.items():
 response=a['response'];assert response['status']=='compiled'
 params=[{'name':'p'+str(p['position']),'type':'STRING','value':p['value']} for p in response['parameters']]
 query=statements[identifier+'-query'];used.add(identifier+'-query')
 assert query['sql']==response['sql'] and query['parameters']==params
 actual=rows(query);assert actual==outcomes[identifier]['rows']
 decoded=[[row[0],strict(row[1])] for row in actual]
 assert canonical(decoded)==canonical(outcomes[identifier]['expected'])
 guards=[c for o in response['obligations'] if 'checks' in o['parameters'] for c in o['parameters']['checks']]
 for i,c in enumerate(guards):
  label=identifier+'-guard-'+str(i);guard=statements[label];used.add(label)
  assert guard['sql']==c['sql'] and guard['parameters']==params and rows(guard)==[['0']];checks+=1
 report.append({'id':identifier,'statementId':query['response']['statement_id'],'expected':outcomes[identifier]['expected']})
assert used==set(statements)
OUT=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-compound-page-reconciliation';OUT.mkdir(exist_ok=True)
r={'status':'passed','cases':48,'integrityReceipts':checks,'acceptedReceipts':len(used),'reconcilerSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'sources':[{'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in [BASE/'summary.json',BASE/'compile-artifacts.jsonl',accepted,BASE/'custody.json',ROOT/'tests/ashlar-databricks/compound-application-native.py']],'scope':'Saved accepted native receipts; strict decoded expected-value comparison. Not a fresh native execution or production profile.','casesReconciled':report}
(OUT/'summary.json').write_text(json.dumps(r,indent=2,ensure_ascii=False)+'\n')
print(json.dumps({'status':'passed','cases':48,'integrityReceipts':checks,'acceptedReceipts':len(used)}))
