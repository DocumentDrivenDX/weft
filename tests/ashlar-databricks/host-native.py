"""Actual read-only host orchestration on immutable native fixture publications.
@covers US-004-AC4. Injected policy/custody faults, no production resolver claim.
"""
import copy,hashlib,json,os,re,subprocess
from pathlib import Path
from native_transport import Client
from host_obligation_fixture import execute,Refused
ROOT=Path(__file__).resolve().parents[2];E=ROOT/'docs/helix/04-build/evidence';OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);c=Client(OUT)
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest()
def p(k,v):return dict(name=k,type='STRING',value=v)
def entries(scope):return [json.loads(x) for x in (E/('B-006-'+scope)/'compile-artifacts.jsonl').read_text().splitlines()]
selected=[('scalar',json.loads((E/'B-006-scalar-native/exact-compile.json').read_text()))]
for label,scope,predicate in [('optional','optional-native',lambda x:x['id']=='string-present-column'),('relationship','relationship-native',lambda x:'edge_ab-column-forward-2'==x['id']),('entity','application-native',lambda x:x['id']=='8-props-props-composite-page-0'),('page','application-native',lambda x:x['id']=='64-column-column-composite-page-1'),('compound','compound-native',lambda x:x['id']=='cyclic-False-exact')]:
 choices=[x for x in entries(scope) if predicate(x)];assert choices,(label,scope);selected.append((label,choices[0]))
# Physical schema validation against the exact original owner DDL, independent of compiler SQL.
layout=(ROOT/'spec/upstream/ashlar-delta-v03.sql').read_text();layout_sha=hashlib.sha256(layout.encode()).hexdigest()
def detail(table,label):
 rows=c.sql(label,'DESCRIBE DETAIL '+table);require(len(rows)==1)
 return dict(zip([x['name'] for x in c.records[-1]['response']['manifest']['schema']['columns']],rows[0]))
def require(value):
 if not value:raise Refused('native fixture custody refused')
def schema(table,label):
 owner=table.rsplit('.',1)[-1];ddl=re.search(r'CREATE TABLE '+owner+r' \((.*?)\n\)',layout,re.S).group(1)
 expected=[(m.group(1),m.group(2).lower()) for m in re.finditer(r'^\s*(\w+)\s+(STRING|BIGINT|BOOLEAN|DOUBLE|TIMESTAMP)\b',ddl,re.M)]
 rows=c.sql(label,'DESCRIBE TABLE '+table);actual=[(r[0],r[1]) for r in rows[:len(expected)]];require(actual==expected and expected)
 return expected
outcomes=[];compiled=[]
for label,saved in selected:
 artifact=json.loads(subprocess.run([str(binary)],input=json.dumps(saved['request']),text=True,capture_output=True,check=True).stdout);require(artifact['status']=='compiled')
 compiled.append(dict(id=label,request=saved['request'],response=artifact))
 publication=next(o['parameters'] for o in artifact['obligations'] if o['id']=='ashlar.candidate.publication')
 binding=json.loads(saved['request']['target']['bindingJson']);pub=publication['publication'];tables=['.'.join(t['name']) for t in pub['tables']];manifest='.'.join(pub['tables'][0]['name'][:-1])+'.publication_manifest'
 schemas=[schema(t,label+'-schema-'+str(i)) for i,t in enumerate(tables)]
 caller=c.sql(label+'-authenticated-caller','SELECT current_user()')[0][0];require(isinstance(caller,str) and caller)
 session=dict(caller=caller,policyRevision='synthetic-complete-admin-policy-1')
 # Independently pinned fixture attestations are deliberately not advertised as production policy.
 def snapshot():
  require(c.sql(label+'-context-caller','SELECT current_user()')==[[caller]])
  require(detail(manifest,label+'-manifest-identity')['id']==pub['manifestUuid'])
  versions=c.sql(label+'-manifest-vector','SELECT storage_layout_revision,table_versions_json FROM '+manifest+' WHERE publication_id=:id',[p('id',pub['id'])]);require(len(versions)==1 and versions[0][0]=='ashlar-delta/0.3')
  vector=json.loads(versions[0][1]);require(all(vector[t]==v['version'] for t,v in zip(tables,pub['tables'])))
  for i,(table,pin) in enumerate(zip(tables,pub['tables'])):
   require(detail(table,label+'-table-identity-'+str(i))['id']==pin['uuid'])
   # A native scan at the exact immutable version proves data-file availability.
   c.sql(label+'-retained-version-'+str(i),'SELECT CAST(COUNT(*) AS STRING) FROM '+table+' VERSION AS OF '+str(pin['version']))
  return dict(effectiveCaller=caller,authenticatedCaller=caller,policyRevision=session['policyRevision'],completeVisibility=True,publication=pub,modelPins=artifact['modelPins'],bindingSha256=artifact['bindingSha256'],layoutRevision=publication['layoutRevision'],layoutSha256=layout_sha,targetContext=artifact['targetContext'],manifestVerified=True,schemasVerified=True,retainedDataFiles=True,sourceIdentityVerified=True,endpointIntegrityVerified=True,projectionCoverageVerified=True,payloadValidated=True,mappingVerified=True,lifecycleVerified=True)
 query_calls=[]
 def query(sql,slots):
  params=[p('p'+str(s['position']),s['value']) for s in slots];query_calls.append(sql)
  rows=c.sql(label+'-executor-'+str(len(query_calls)),sql,params)
  response=c.records[-1]['response'];m=response.get('manifest',{})
  return dict(rows=rows,types=[col['type_name'] for col in m['schema']['columns']],complete=response['status']['state']=='SUCCEEDED',truncated=bool(m.get('truncated',False)))
 actual=execute(artifact,snapshot,query,session)
 # Compare with the independent native corpus oracle, never regenerated SQL expectations.
 if label=='scalar':expected=[['e\u0301','-1.25'],['x ','2.00'],['é','200000000000000000000000000.02']];require(sorted(actual)==sorted(expected))
 else:
  summary=json.loads((E/('B-006-'+{'optional':'optional-native','relationship':'relationship-native','entity':'application-native','page':'application-native','compound':'compound-native'}[label])/'summary.json').read_text())
  previous=next(x for x in summary['outcomes'] if x['id']==saved['id']);expected=previous.get('rows')
  # Some scalar application receipts retain only an oracle; exact saved native rows are recovered from their terminal statement.
  if expected is None:
   statements=[json.loads(x) for x in (E/('B-006-'+{'entity':'application-native','page':'application-native'}[label])/'statements.jsonl').read_text().splitlines()]
   matches=[x for x in statements if x['label'].startswith(str(saved['id'])) and 'user-query' in x['label']];require(matches);expected=matches[-1]['response'].get('result',{}).get('data_array',[])
  require(sorted(actual)==sorted(expected))
 outcomes.append(dict(id=label,outcome='buffered-exact-result',rows=actual,schema=schemas,queryCalls=len(query_calls)))
 # Post-execution custody faults must discard the actual native rows.
 for key,replacement in [('effectiveCaller','broader-principal'),('policyRevision','changed-policy'),('publication',{}),('retainedDataFiles',False)]:
  calls=0
  def fault_snapshot():
   global calls
   calls+=1;view=snapshot()
   if calls==2:view=copy.deepcopy(view);view[key]=replacement
   return view
  try:execute(artifact,fault_snapshot,query,session);raise AssertionError('published after '+key+' change')
  except Refused:require(calls==2)
  outcomes.append(dict(id=label+'-changed-'+key,outcome='discarded-buffered-native-rows'))
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha
(OUT/'compile-artifacts.jsonl').write_text(''.join(json.dumps(x,ensure_ascii=False)+'\n' for x in compiled))
summary=dict(state='passed',cases=len(outcomes),outcomes=outcomes,compilerBinarySha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Actual Databricks metadata/version/guard/data transport; synthetic complete-admin policy, independently pinned fixture custody and injected before-publication faults. No production delegation/resolver/security-service qualification.')
(OUT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'}))
