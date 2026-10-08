"""Independent recursive/presence/native scalar oracles on pinned owner layout.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
"""
import hashlib,json,os,re,subprocess,sys
from decimal import Decimal
from pathlib import Path
from native_transport import Client,NativeFailure
ROOT=Path(__file__).resolve().parents[2];OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
SCHEMA='client_dev.weft_b006_20261008_compounds';TABLE=SCHEMA+'.node_type_a';MANIFEST=SCHEMA+'.publication_manifest'
layout=(ROOT/'spec/upstream/ashlar-delta-v03.sql').read_text();layout_sha=hashlib.sha256(layout.encode()).hexdigest();assert layout_sha=='ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e'
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest()
def exact_json(v):
 if isinstance(v,Decimal):return format(v,'f')
 if isinstance(v,dict):return '{'+','.join(json.dumps(k,ensure_ascii=False)+':'+exact_json(x) for k,x in v.items())+'}'
 if isinstance(v,list):return '['+','.join(exact_json(x) for x in v)+']'
 return json.dumps(v,ensure_ascii=False)
absent={'state':'absent'}
def present(v):return {'state':'value','value':v}
minimum=-(1<<63);maximum=(1<<63)-1
specs=[
 dict(id='integer-list',shape='sequence',family='integer',bits=64,value=[minimum,maximum,0],expected=[str(minimum),str(maximum),'0'],bad=[maximum+1]),
 dict(id='boolean-list',shape='sequence',family='boolean',value=[False,True,False],expected=[False,True,False],bad=['false']),
 dict(id='string-list',shape='sequence',family='string',value=['é','e\u0301','x ','"\\\n'],expected=['é','e\u0301','x ','"\\\n'],bad=['x\0']),
 dict(id='decimal-list',shape='sequence',family='decimal',value=[Decimal('12345678901234567890123456.78'),Decimal('0.01'),Decimal('0.00')],expected=['12345678901234567890123456.78','0.01','0.00'],bad=[Decimal('1.001')]),
 dict(id='integer-map',shape='map',family='integer',bits=8,value={'é':-128,'e\u0301':127,'a.b"\\':0},expected={'é':'-128','e\u0301':'127','a.b"\\':'0'},bad={'x\0':1}),
 dict(id='nested-list',shape='nested',family='integer',bits=8,value=[[1,2],[],[-128,127]],expected=[['1','2'],[],['-128','127']],bad=[['17']]),
 dict(id='structured',shape='structured',family='integer',bits=64,value={'leaf':maximum,'note':'é'},expected={'leaf':str(maximum),'note':present('é')},bad={'leaf':0,'unknown':False}),
 dict(id='cyclic',shape='cyclic',family='integer',bits=8,value={'leaf':7,'next':{'leaf':-128,'note':'é'}},expected={'leaf':'7','note':absent,'next':present({'leaf':'-128','note':present('é'),'next':absent})},bad={'leaf':1,'next':{}}),
 dict(id='optional-items',shape='sequence',family='boolean',item_optional=True,value=[False,True],expected=[present(False),present(True)],bad=[None]),
 dict(id='map-structured',shape='map-structured',family='integer',bits=8,value={'x':{'leaf':7}},expected={'x':{'leaf':'7','note':absent}},bad={'x':{'leaf':7,'unknown':False}}),
 dict(id='list-map',shape='list-map',family='decimal',value=[{'x':Decimal('0.00')},{}],expected=[{'x':'0.00'},{}],bad=[{'x':'0.00'}]),
]
cases=[];rows=[]
for spec in specs:
 for optional in [False,True]:
  for label in ['exact','empty','absent','explicit-null','wrong-root','hidden-bad-leaf']:
   source='weft-compound-'+spec['id']+'-'+str(optional)+'-'+label
   if label=='empty':
    if spec['shape'] in ['structured','cyclic']:
     value={'leaf':0};expected={'leaf':'0','note':absent}
     if spec['shape']=='cyclic':expected['next']=absent
    else:value={} if spec['shape'] in ['map','map-structured'] else [];expected=value
   else:value=spec['value'];expected=spec['expected']
   props={'23':1,'28':value}
   if label=='absent':props.pop('28')
   if label=='explicit-null':props['28']=None
   if label=='wrong-root':props['28']=False
   corrupt=label in ['explicit-null','wrong-root','hidden-bad-leaf'] or (label=='absent' and not optional)
   incoming=[props]
   if label=='hidden-bad-leaf':incoming.append({'23':2,'28':spec['bad']})
   for i,properties in enumerate(incoming,1):rows.append(dict(node_key='N'+str(len(source))+':'+source+':1:'+str(i),source_system=source,type_id=1,id=i,group_value=None,group_present=False,rank_value=None,rank_present=False,props_json=exact_json(properties),retained_json='{}'))
   cases.append(dict(id=spec['id']+'-'+str(optional)+'-'+label,spec=spec,optional=optional,source=source,corrupt=corrupt,expected=absent if label=='absent' else present(expected),label=label))
# A separately authored zero-row context proves no fabricated compound cell.
spec=specs[0];cases.append(dict(id='empty-owner',spec=spec,optional=False,source='weft-compound-empty-owner',corrupt=False,expected=None,label='empty-owner'))
payload=exact_json(rows);payload_sha=hashlib.sha256(payload.encode()).hexdigest()
def request(case,state):
 spec=case['spec'];shape=spec['shape']
 def logical(element):return dict(documentId='compound-fixture',revision='1',module='main',element=element)
 def field(id,family,optional=False):
  v=dict(id=id,name=id,kind='field',cardinality='one',nullability='absent-allowed' if optional else 'required',scalarType=family,extensions={})
  if family=='integer':v['facets']={'integerWidth':{'bits':spec.get('bits',8),'signed':True}}
  if family=='decimal':v['facets']={'precision':28,'scale':2}
  return v
 root=dict(id='payload',name='payload',kind='field',cardinality='map' if shape in ['map','map-structured'] else 'array',nullability='absent-allowed' if case['optional'] else 'required',itemType=dict(module='main',element='leaf'),extensions={})
 extras=[];leaf=field('leaf',spec['family'],spec.get('item_optional',False))
 if shape in ['structured','cyclic','map-structured']:
  members=[dict(module='main',element='leaf'),dict(module='main',element='note')];extras.append(field('note','string',True))
  if shape=='cyclic':
   members.append(dict(module='main',element='next'));extras.append(dict(id='next',name='next',kind='field',cardinality='one',nullability='absent-allowed',references=[dict(module='main',element='record',role='record-type')],extensions={}))
  extras.append(dict(id='record',name='PayloadRecord',kind='record',members=members,extensions={}))
  if shape=='map-structured':
   root['itemType']=dict(module='main',element='item');extras.append(dict(id='item',name='item',kind='field',cardinality='one',nullability='required',references=[dict(module='main',element='record',role='record-type')],extensions={}))
  else:
   root.pop('itemType');root['cardinality']='one';root['references']=[dict(module='main',element='record',role='record-type')]
 elif shape in ['nested','list-map']:
  root['itemType']=dict(module='main',element='item');extras.append(dict(id='item',name='item',kind='field',cardinality='array' if shape=='nested' else 'map',nullability='required',itemType=dict(module='main',element='leaf'),extensions={}))
 id_field=field('id','integer');id_field['facets']={'integerWidth':{'bits':64,'signed':True}}
 doc=json.dumps(dict(umf='0.7.0',id='compound-fixture',vocabularies={},modules=[dict(id='main',namespace='compounds',elements=[dict(id='thing',name='Thing',kind='record',members=[dict(module='main',element='id'),dict(module='main',element='payload')],extensions={}),id_field,root,leaf]+extras)],extensions={}))
 pin=dict(documentId='compound-fixture',revision='1',umfVersion='0.7.0',sha256=hashlib.sha256(doc.encode()).hexdigest())
 binding=json.dumps(dict(profile='ashlar-databricks-candidate/0.1.0',layoutRevision='ashlar-delta/0.3',layoutSha256=layout_sha,modelPins=[pin],publication=dict(id=state['publicationId'],manifestUuid=state['manifestUuid'],tables=[dict(name=TABLE.split('.'),uuid=state['tableUuid'],version=state['version'])]),records=[dict(logical=logical('thing'),table=0,kind='nodeProjection',sourceSystem=case['source'],typeId='1',schemaRevision='fixture-schema-1',properties=[dict(logical=logical('id'),home=dict(kind='props',propertyId='23')),dict(logical=logical('payload'),home=dict(kind='props',propertyId='28',encoding='ashlar-weft-json-value/0.1-candidate'))])]))
 sql='SELECT t.payload FROM Thing t'+(' WHERE t.id = 1' if case['label']=='hidden-bad-leaf' else '')
 return dict(interfaceVersion='weft-compile/0.2.0',dialect='weft-sql/0.2.0',sql=sql,modules=[dict(documentJson=doc,pin=pin,selectedModuleIds=['main'])],target=dict(backendId='ashlar.databricks',backendVersion='0.1.0-candidate',targetProfile='dbsql-candidate',bindingJson=binding,bindingSha256=hashlib.sha256(binding.encode()).hexdigest()),options=dict(allowCandidate=True))
def compile(req):return json.loads(subprocess.run([str(binary)],input=json.dumps(req),text=True,capture_output=True,check=True).stdout)
# Validate all authored input models/mappings before any database write.
placeholder=dict(publicationId='synthetic',manifestUuid='synthetic',tableUuid='synthetic',version=0)
for case in cases:assert compile(request(case,placeholder))['status']=='compiled',case['id']
if '--compile-only' in sys.argv:print('All '+str(len(cases))+' authored compound requests compile');sys.exit(0)
c=Client(OUT)
def p(name,value):return dict(name=name,type='STRING',value=value)
def detail(table):
 data=c.sql('identity-'+table.rsplit('.',1)[-1],'DESCRIBE DETAIL '+table);assert len(data)==1
 return dict(zip([x['name'] for x in c.records[-1]['response']['manifest']['schema']['columns']],data[0]))
state_path=OUT/'fixture-state.json'
if '--resume-pinned' in sys.argv:
 state=json.loads(state_path.read_text());assert state['phase']=='ready' and state['payloadSha256']==payload_sha
else:
 assert not state_path.exists(),'Inspect fixture custody before replay';assert c.sql('schema-custody',"SHOW SCHEMAS IN client_dev LIKE 'weft_b006_20261008_compounds'")==[]
 c.sql('create-owned-schema','CREATE SCHEMA '+SCHEMA)
 for table in ['node_type_a','publication_manifest']:
  ddl=re.search(r'CREATE TABLE '+table+r' \(.*?;',layout,re.S).group(0);c.sql('create-'+table,ddl.replace('CREATE TABLE '+table,'CREATE TABLE '+SCHEMA+'.'+table,1))
 c.sql('insert-independent-fixtures','INSERT INTO '+TABLE+" (node_key,source_system,type_id,id,group_value,group_present,rank_value,rank_present,props_json,retained_json) SELECT r.* FROM (SELECT explode(from_json(:rows,'ARRAY<STRUCT<node_key:STRING,source_system:STRING,type_id:BIGINT,id:BIGINT,group_value:STRING,group_present:BOOLEAN,rank_value:BIGINT,rank_present:BOOLEAN,props_json:STRING,retained_json:STRING>>')) r)",[p('rows',payload)])
 identity=detail(TABLE);version=int(c.sql('table-history','DESCRIBE HISTORY '+TABLE+' LIMIT 1')[0][0]);publication='weft-compound-fixtures'
 c.sql('publish-fixture-vector','INSERT INTO '+MANIFEST+" VALUES (:id,'ashlar-delta/0.3',:vector,'{}',:revisions,:report,TIMESTAMP '2026-10-08 00:00:00')",[p('id',publication),p('vector',json.dumps({TABLE:version})),p('revisions',json.dumps({'compound-fixture':'fixture-schema-1'})),p('report',json.dumps({'qualification':'synthetic exact JSON recursive value controls','payloadSha256':payload_sha}))])
 state=dict(phase='ready',payloadSha256=payload_sha,tableUuid=identity['id'],version=version,manifestUuid=detail(MANIFEST)['id'],publicationId=publication);state_path.write_text(json.dumps(state,indent=2)+'\n')
assert detail(TABLE)['id']==state['tableUuid'];assert detail(MANIFEST)['id']==state['manifestUuid']
assert c.sql('resolve-fixture-vector','SELECT table_versions_json FROM '+MANIFEST+' WHERE publication_id=:id',[p('id',state['publicationId'])])==[[json.dumps({TABLE:state['version']})]]
(OUT/'compile-artifacts.jsonl').write_text('');outcomes=[]
for case in cases:
 req=request(case,state);artifact=compile(req);assert artifact['status']=='compiled',artifact
 with (OUT/'compile-artifacts.jsonl').open('a') as stream:stream.write(json.dumps(dict(id=case['id'],request=req,response=artifact),ensure_ascii=False)+'\n')
 params=[p('p'+str(s['position']),s['value']) for s in artifact['parameters']]
 checks=[check for o in artifact['obligations'] if o['id'] in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.compoundIntegrity'] for check in o['parameters']['checks']]
 counts=[c.sql(case['id']+'-guard-'+str(i),check['sql'],params)[0][0] for i,check in enumerate(checks)]
 if case['corrupt']:
  assert any(int(v)>0 for v in counts),(case['id'],counts);outcome=dict(id=case['id'],outcome='refused-before-user-query',counts=counts)
 else:
  assert all(v=='0' for v in counts),(case['id'],counts);actual=c.sql(case['id']+'-user-query',artifact['sql'],params)
  expected=[] if case['expected'] is None else [case['expected']]
  assert [json.loads(row[0]) for row in actual]==expected,(case['id'],actual,expected)
  assert artifact['columns'][0]['representation']['kind']=='value' and artifact['columns'][0]['representation']['nativeNull'] is False
  assert all(col['type_name']=='STRING' for col in c.records[-1]['response']['manifest']['schema']['columns'])
  outcome=dict(id=case['id'],outcome='published-fixture-result',rows=actual,expected=expected)
 outcomes.append(outcome)
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha
summary=dict(state='passed',cases=len(outcomes),positive=sum(o['outcome']=='published-fixture-result' for o in outcomes),refusals=sum(o['outcome']=='refused-before-user-query' for o in outcomes),outcomes=outcomes,fixture=state,compilerBinarySha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Synthetic pinned projection, exact recursive JSON profile; independent scalar/list/map/structured/cyclic and presence oracles. No host/production publication qualification; limit boundaries not included in this receipt.')
(OUT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'}))
