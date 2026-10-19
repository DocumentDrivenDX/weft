"""@covers US-007-AC4: saved native relationship receipts and exact bags."""
import json,hashlib,re
from pathlib import Path
from collections import Counter
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-006-relationship-native'
def unique(items,key):
 d={}
 for x in items:assert x[key] not in d;d[x[key]]=x
 return d
def lines(name):return [json.loads(x) for x in (BASE/name).read_text().splitlines()]
def pairs(items):
 d={}
 for k,v in items:assert k not in d;d[k]=v
 return d
def strict(raw):
 def invalid(v):raise AssertionError(v)
 return json.loads(raw,object_pairs_hook=pairs,parse_constant=invalid)
def bag(rows):return Counter(json.dumps(r,sort_keys=True,ensure_ascii=False,separators=(',',':')) for r in rows)
def data(record):
 r=record['response'];assert r['status']['state']=='SUCCEEDED'
 m=r['manifest'];assert not m.get('truncated',False)
 result=r.get('result',{});rows=result.get('data_array',[])
 assert result.get('row_count',len(rows))==len(rows)==m['total_row_count']
 assert m.get('total_chunk_count',1) in ([0,1] if not rows else [1])
 assert result.get('chunk_index',0)==0 and result.get('row_offset',0)==0
 return rows
s=json.loads((BASE/'summary.json').read_text());assert s['state']=='passed'
a=unique(lines('compile-artifacts.jsonl'),'id');outcomes=unique(s['outcomes'],'id')
needed={label for o in outcomes.values() for label in o['guardReceipts']}|{o['id']+'-user-query' for o in outcomes.values()}
groups={}
for r in lines('statements.jsonl'):
 if r['label'] in needed:groups.setdefault(r['label'],[]).append(r)
receipts={}
for label,group in groups.items():
 first=group[0]
 for repeated in group[1:]:assert repeated['sql']==first['sql'] and repeated['parameters']==first['parameters'] and data(repeated)==data(first)
 receipts[label]=first
assert set(a)==set(outcomes) and len(a)==s['cases']==52
used_guards=set();positive=refused=0;records=[]
for identifier,artifact in a.items():
 response=artifact['response'];assert response['status']=='compiled'
 params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in response['parameters']]
 checks=[check for o in response['obligations'] if 'checks' in o['parameters'] for check in o['parameters']['checks']]
 o=outcomes[identifier];assert len(checks)==len(o['guardReceipts']);counts=[]
 for check,label in zip(checks,o['guardReceipts'],strict=True):
  guard=receipts[label];used_guards.add(label)
  names=set(re.findall(r':(p[0-9]+)\b',check['sql']))
  assert guard['sql']==check['sql'] and guard['parameters']==[p for p in params if p['name'] in names]
  observed=data(guard);assert len(observed)==1 and len(observed[0])==1
  count=observed[0][0];assert isinstance(count,str) and re.fullmatch(r'[0-9]+',count)
  counts.append(count)
 label=identifier+'-user-query'
 if o['outcome']=='refused-before-user-query':
  assert counts==o['counts'] and any(int(x)>0 for x in counts) and label not in receipts
  refused+=1
 else:
  assert o['outcome']=='published-fixture-result' and all(x=='0' for x in counts)
  query=receipts[label];assert query['sql']==response['sql'] and query['parameters']==params
  actual=data(query);assert actual==o['rows']
  if 'HAS_RELATED' in artifact['request']['sql']:decoded=actual
  else:decoded=[r[:-1]+[strict(r[-1])] for r in actual]
  assert bag(decoded)==bag(o['expected']);positive+=1
 records.append({'id':identifier,'outcome':o['outcome'],'guardReceipts':o['guardReceipts'],'guardStatementIds':[[r['response']['statement_id'] for r in groups[label]] for label in o['guardReceipts']]})
assert positive==s['positive']==28 and refused==s['refusals']==24
assert len(used_guards)==s['uniqueNativeGuards']==272
OUT=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-relationship-reconciliation';OUT.mkdir(exist_ok=True)
report={'status':'passed','cases':52,'positive':positive,'refusals':refused,'uniqueNativeGuards':len(used_guards),'reconcilerSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'sources':[{'path':str(p.relative_to(ROOT)),'sha256':hashlib.sha256(p.read_bytes()).hexdigest()} for p in [BASE/'summary.json',BASE/'compile-artifacts.jsonl',BASE/'statements.jsonl',ROOT/'tests/ashlar-databricks/relationship-native.py']],'scope':'Saved native receipt reconciliation with exact duplicate-preserving expected bags and guard-linked refusal. Not fresh execution or production qualification.','casesReconciled':records}
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:report[k] for k in ['status','cases','positive','refusals','uniqueNativeGuards']}))
