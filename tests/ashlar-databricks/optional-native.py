"""Independent optional-value native corpus on pinned owner projection layout.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
"""
import hashlib,json,os,re,subprocess,sys
from pathlib import Path
from native_transport import Client
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
c=Client(OUT)
SCHEMA='client_dev.weft_b006_20261008_optional'
TABLE=SCHEMA+'.node_type_a'; MANIFEST=SCHEMA+'.publication_manifest'
layout=(ROOT/'spec/upstream/ashlar-delta-v03.sql').read_text()
layout_sha=hashlib.sha256(layout.encode()).hexdigest()
assert layout_sha=='ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e'
# Expected logical values are independent of compiler SQL and carriers.
values={'string':'é"\\\nx ', 'integer':-128, 'boolean':False, 'decimal':'12345678901234567890123456.78'}
labels=['present','absent','explicit-null','wrong-type','malformed-root','hidden-corruption']
rows=[]; fixtures=[]
for family,value in values.items():
 for label in labels:
  source='weft-optional-'+family+'-'+label
  payload={'24':value}
  if family=='decimal': props='{"24":'+value+'}'
  else: props=json.dumps(payload,ensure_ascii=False)
  if label=='absent':props='{}'
  if label=='explicit-null':props='{"24":null}'
  if label=='wrong-type':props='{"24":[]}'
  if label=='malformed-root':props='[]'
  if label=='hidden-corruption':props='{"24":null}'
  present=label!='absent'
  group=value if family=='string' and label in ['present','hidden-corruption'] else None
  rank=value if family=='integer' and label in ['present','hidden-corruption'] else None
  # Hidden corrupt typed homes must also refuse, even when WHERE rejects the row.
  if label=='hidden-corruption':group=None;rank=None
  rows.append(dict(node_key='N'+str(len(source))+':'+source+':1:1',source_system=source,type_id=1,id=1,group_value=group,group_present=present,rank_value=rank,rank_present=present,props_json=props,retained_json='{}'))
  fixtures.append(dict(family=family,label=label,source=source))
# Physical presence consistency controls, independent from JSON absence.
for family in ['string','integer']:
 for label in ['false-with-value']:
  source='weft-optional-'+family+'-'+label
  rows.append(dict(node_key='N'+str(len(source))+':'+source+':1:1',source_system=source,type_id=1,id=1,group_value='stale' if family=='string' else None,group_present=False if label=='false-with-value' else None,rank_value=7 if family=='integer' else None,rank_present=False if label=='false-with-value' else None,props_json='{}',retained_json='{}'))
  fixtures.append(dict(family=family,label=label,source=source,column_only=True))
payload=json.dumps(rows,ensure_ascii=False,separators=(',',':')); digest=hashlib.sha256(payload.encode()).hexdigest()
def p(name,value):return dict(name=name,type='STRING',value=value)
def detail(table):
 data=c.sql('identity-'+table.rsplit('.',1)[-1],'DESCRIBE DETAIL '+table)
 names=[x['name'] for x in c.records[-1]['response']['manifest']['schema']['columns']]
 assert len(data)==1
 return dict(zip(names,data[0]))
state_path=OUT/'fixture-state.json'
if '--resume-pinned' in sys.argv:
 state=json.loads(state_path.read_text());assert state['phase']=='ready' and state['payloadSha256']==digest
elif '--resume-empty' in sys.argv:
 assert not state_path.exists()
 for table in [TABLE,MANIFEST]:assert c.sql('empty-fixture-custody-'+table.rsplit('.',1)[-1],'SELECT COUNT(*) FROM '+table)==[['0']]
else:
 assert not state_path.exists(),'Inspect prior fixture custody before replay'
 assert c.sql('schema-custody',"SHOW SCHEMAS IN client_dev LIKE 'weft_b006_20261008_optional'")==[]
 c.sql('create-owned-schema','CREATE SCHEMA '+SCHEMA)
 for table in ['node_type_a','publication_manifest']:
  ddl=re.search(r'CREATE TABLE '+table+r' \(.*?;',layout,re.S).group(0)
  c.sql('create-'+table,ddl.replace('CREATE TABLE '+table,'CREATE TABLE '+SCHEMA+'.'+table,1))
if '--resume-pinned' not in sys.argv:
 c.sql('insert-independent-fixtures','INSERT INTO '+TABLE+" SELECT r.* FROM (SELECT explode(from_json(:rows,'ARRAY<STRUCT<node_key:STRING,source_system:STRING,type_id:BIGINT,id:BIGINT,group_value:STRING,group_present:BOOLEAN,rank_value:BIGINT,rank_present:BOOLEAN,props_json:STRING,retained_json:STRING>>')) r)",[p('rows',payload)])
 identity=detail(TABLE);version=int(c.sql('table-history','DESCRIBE HISTORY '+TABLE+' LIMIT 1')[0][0]);publication='weft-optional-fixtures'
 c.sql('publish-fixture-vector','INSERT INTO '+MANIFEST+" VALUES (:id,'ashlar-delta/0.3',:vector,'{}',:revisions,:report,TIMESTAMP '2026-10-08 00:00:00')",[p('id',publication),p('vector',json.dumps({TABLE:version})),p('revisions',json.dumps({'optional-fixture':'fixture-schema-1'})),p('report',json.dumps({'qualification':'independent synthetic optional controls','payloadSha256':digest}))])
 state=dict(phase='ready',payloadSha256=digest,tableUuid=identity['id'],version=version,manifestUuid=detail(MANIFEST)['id'],publicationId=publication)
 state_path.write_text(json.dumps(state,indent=2)+'\n')
assert detail(TABLE)['id']==state['tableUuid'];assert detail(MANIFEST)['id']==state['manifestUuid']
assert c.sql('resolve-fixture-vector','SELECT table_versions_json FROM '+MANIFEST+' WHERE publication_id=:id',[p('id',state['publicationId'])])==[[json.dumps({TABLE:state['version']})]]
(OUT/'compile-artifacts.jsonl').write_text('')
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest();outcomes=[]
for fixture in fixtures:
 family=fixture['family'];label=fixture['label']
 field=dict(id='value',name='value',kind='field',scalarType=family,nullability='absent-allowed',cardinality='one',extensions={})
 if family=='integer':field['facets']={'integerWidth':{'bits':8,'signed':True}}
 if family=='decimal':field['facets']={'precision':28,'scale':2}
 document=json.dumps(dict(umf='0.7.0',id='optional-fixture',vocabularies={},modules=[dict(id='main',namespace='optional',elements=[dict(id='thing',name='Thing',kind='record',members=[dict(module='main',element='value')],extensions={}),field])],extensions={}))
 pin=dict(documentId='optional-fixture',revision='1',umfVersion='0.7.0',sha256=hashlib.sha256(document.encode()).hexdigest())
 def logical(element):return dict(documentId='optional-fixture',revision='1',module='main',element=element)
 homes=['column'] if fixture.get('column_only') else ['props','column'] if family in ['string','integer'] else ['props']
 original_document=document;original_pin=pin.copy()
 for home in homes:
  document=original_document;pin=original_pin.copy()
  if home=='column' and label=='malformed-root':continue # unrelated JSON text does not determine selected typed field
  physical=dict(kind='props',propertyId='24') if home=='props' else dict(kind='column',value='group_value' if family=='string' else 'rank_value',present='group_present' if family=='string' else 'rank_present',nativeType='STRING' if family=='string' else 'BIGINT')
  binding=json.dumps(dict(profile='ashlar-databricks-candidate/0.1.0',layoutRevision='ashlar-delta/0.3',layoutSha256=layout_sha,modelPins=[pin],publication=dict(id=state['publicationId'],manifestUuid=state['manifestUuid'],tables=[dict(name=TABLE.split('.'),uuid=state['tableUuid'],version=state['version'])]),records=[dict(logical=logical('thing'),table=0,kind='nodeProjection',sourceSystem=fixture['source'],typeId='1',schemaRevision='fixture-schema-1',properties=[dict(logical=logical('value'),home=physical)])]))
  sql='SELECT t.value FROM Thing t'
  if label=='hidden-corruption':
   doc=json.loads(document);doc['modules'][0]['elements'][0]['members'].append(dict(module='main',element='id'))
   doc['modules'][0]['elements'].append(dict(id='id',name='id',kind='field',scalarType='integer',nullability='required',cardinality='one',facets={'integerWidth':{'bits':64,'signed':True}},extensions={}))
   document=json.dumps(doc);pin['sha256']=hashlib.sha256(document.encode()).hexdigest()
   bound=json.loads(binding);bound['modelPins']=[pin]
   bound['records'][0]['properties'].append(dict(logical=logical('id'),home=dict(kind='column',value='id',nativeType='BIGINT')))
   binding=json.dumps(bound);sql='SELECT t.value FROM Thing t WHERE t.id = 0'
  request=dict(interfaceVersion='weft-compile/0.2.0',dialect='weft-sql/0.2.0',sql=sql,modules=[dict(documentJson=document,pin=pin,selectedModuleIds=['main'])],target=dict(backendId='ashlar.databricks',backendVersion='0.1.0-candidate',targetProfile='dbsql-candidate',bindingJson=binding,bindingSha256=hashlib.sha256(binding.encode()).hexdigest()),options=dict(allowCandidate=True))
  identifier=family+'-'+label+'-'+home
  artifact=json.loads(subprocess.run([str(binary)],input=json.dumps(request),text=True,capture_output=True,check=True).stdout)
  assert artifact['status']=='compiled',(identifier,artifact)
  with (OUT/'compile-artifacts.jsonl').open('a') as stream:stream.write(json.dumps(dict(id=identifier,request=request,response=artifact),ensure_ascii=False)+'\n')
  params=[p('p'+str(slot['position']),slot['value']) for slot in artifact['parameters']]
  checks=next(o for o in artifact['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks'];assert len(checks)==(2 if label=='hidden-corruption' else 1)
  counts=[c.sql(identifier+'-integrity-'+str(i),check['sql'],params) for i,check in enumerate(checks)]
  if label not in ['present','absent']:
   assert any(n != [['0']] for n in counts),(identifier,counts)
   outcome=dict(id=identifier,outcome='refused-before-user-query',counts=counts)
  else:
   assert counts==[[['0']]],(identifier,counts)
   actual=c.sql(identifier+'-user-query',artifact['sql'],params)
   expected={'state':'absent'} if label=='absent' else {'state':'value','value':str(values[family]) if family in ['integer','decimal'] else values[family]}
   assert len(actual)==1 and json.loads(actual[0][0])==expected,(identifier,actual,expected)
   assert artifact['columns'][0]['representation']==dict(kind='value',descriptor=logical('value'),nativeNull=False)
   assert artifact['columns'][0]['nullable'] is False
   outcome=dict(id=identifier,outcome='published-fixture-result',rows=actual,expected=expected)
  outcomes.append(outcome)
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha
summary=dict(state='passed',cases=len(outcomes),positive=sum(o['outcome']=='published-fixture-result' for o in outcomes),refusals=sum(o['outcome']=='refused-before-user-query' for o in outcomes),outcomes=outcomes,fixture=state,compilerBinarySha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Synthetic pinned owner projection layout, independent optional scalar envelopes across props and STRING/BIGINT homes. No production publication/policy qualification or explicit native-null permission.')
(OUT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'}))
