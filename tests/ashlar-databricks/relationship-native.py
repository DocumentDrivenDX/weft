"""Authored forward/inverse relationships over pinned canonical and serving edges.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component)
"""
import hashlib,json,os,re,subprocess,sys
from pathlib import Path
from native_transport import Client
ROOT=Path(__file__).resolve().parents[2];OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);c=Client(OUT)
SCHEMA='client_dev.weft_b006_20261008_relationships_v2';MANIFEST=SCHEMA+'.publication_manifest'
layout=(ROOT/'spec/upstream/ashlar-delta-v03.sql').read_text();layout_sha=hashlib.sha256(layout.encode()).hexdigest()
assert layout_sha=='ad4a264508c971aefcd94e3ae90f8f74dcf119b7d767f6060c638f4abde3284e'
labels=['exact','orphan','wrong-endpoint','duplicate-key','duplicate-edge']
# Authored logical keys differ from physical identity. Composite target keys have Unicode and signed boundaries.
customers=[(1,10),(2,20),(3,30)];orders=[(101,'é',-128),(102,'x ',127),(103,'é',2),(104,'e\u0301',3)]
base_edges=[(1,1,101),(2,1,101),(3,1,102),(4,2,103)]
nodes=[];canonical=[];serving=[]
def node(source,type_id,id):return 'N'+str(len(source))+':'+source+':'+str(type_id)+':'+str(id)
for label in labels:
 source='weft-relationship-'+label
 for id,key in customers:
  nodes.append(dict(node_key=node(source,1,id),source_system=source,type_id=1,id=id,group_value=None,group_present=False,rank_value=key,rank_present=True,props_json=json.dumps({'23':key}),retained_json='{}'))
 for id,code,rank in orders:
  if label=='duplicate-key' and id==104:code='é';rank=-128
  nodes.append(dict(node_key=node(source,2,id),source_system=source,type_id=2,id=id,group_value=code,group_present=True,rank_value=rank,rank_present=True,props_json=json.dumps({'24':code,'25':rank},ensure_ascii=False),retained_json='{}'))
 edges=base_edges+([(5,3,999)] if label=='orphan' else [])
 for id,src,dst in edges:
  if label=='duplicate-edge' and id==2:id=1
  wrong=label=='wrong-endpoint' and id==4
  canonical.append(dict(source_system=source,rel_type_id=3,id=id,source_type=1,target_type=9 if wrong else 2,source_id=src,target_id=dst,schema_revision='fixture-schema-1',entity_version=1,props_json='{}',retained_json='{}',order_key=None,source_feed='synthetic',source_epoch='1',source_position=1,published_at='2026-10-08 00:00:00',lookup_hash='synthetic',apply_batch_id=None,source_cursor_json=None,source_delivery_id=None))
  serving.append(dict(edge_key='E'+str(len(source))+':'+source+':3:'+str(id),source_system=source,rel_type_id=3,id=id,src=node(source,1,src),dst=node(source,9 if wrong else 2,dst),source_id=src,target_id=dst,score=None,score_present=False,props_json='{}',retained_json='{}'))
payloads={'node_type_a':nodes,'edge_current':canonical,'edge_ab':serving}
schemas={'node_type_a':'node_key:STRING,source_system:STRING,type_id:BIGINT,id:BIGINT,group_value:STRING,group_present:BOOLEAN,rank_value:BIGINT,rank_present:BOOLEAN,props_json:STRING,retained_json:STRING','edge_current':'source_system:STRING,rel_type_id:BIGINT,id:BIGINT,source_type:BIGINT,target_type:BIGINT,source_id:BIGINT,target_id:BIGINT,schema_revision:STRING,entity_version:BIGINT,props_json:STRING,retained_json:STRING,order_key:STRING,source_feed:STRING,source_epoch:STRING,source_position:BIGINT,published_at:TIMESTAMP,lookup_hash:STRING,apply_batch_id:STRING,source_cursor_json:STRING,source_delivery_id:STRING','edge_ab':'edge_key:STRING,source_system:STRING,rel_type_id:BIGINT,id:BIGINT,src:STRING,dst:STRING,source_id:BIGINT,target_id:BIGINT,score:DOUBLE,score_present:BOOLEAN,props_json:STRING,retained_json:STRING'}
payload_sha=hashlib.sha256(json.dumps(payloads,ensure_ascii=False,separators=(',',':')).encode()).hexdigest()
def p(name,value):return dict(name=name,type='STRING',value=value)
def detail(table):
 data=c.sql('identity-'+table.rsplit('.',1)[-1],'DESCRIBE DETAIL '+table);assert len(data)==1
 return dict(zip([x['name'] for x in c.records[-1]['response']['manifest']['schema']['columns']],data[0]))
state_path=OUT/'fixture-state.json'
if '--resume-pinned' in sys.argv:
 state=json.loads(state_path.read_text());assert state['phase']=='ready' and state['payloadSha256']==payload_sha
else:
 assert not state_path.exists(),'Inspect fixture custody before replay'
 assert c.sql('schema-custody',"SHOW SCHEMAS IN client_dev LIKE 'weft_b006_20261008_relationships_v2'")==[]
 c.sql('create-owned-schema','CREATE SCHEMA '+SCHEMA)
 identities={};versions={}
 for table in list(payloads)+['publication_manifest']:
  ddl=re.search(r'CREATE TABLE '+table+r' \(.*?;',layout,re.S).group(0)
  c.sql('create-'+table,ddl.replace('CREATE TABLE '+table,'CREATE TABLE '+SCHEMA+'.'+table,1))
  if table in payloads:
   c.sql('insert-'+table,'INSERT INTO '+SCHEMA+'.'+table+' ('+', '.join(s.split(':')[0] for s in schemas[table].split(','))+')'+" SELECT r.* FROM (SELECT explode(from_json(:rows,'ARRAY<STRUCT<"+schemas[table]+">>')) r)",[p('rows',json.dumps(payloads[table],ensure_ascii=False,separators=(',',':')))])
   versions[table]=int(c.sql('history-'+table,'DESCRIBE HISTORY '+SCHEMA+'.'+table+' LIMIT 1')[0][0])
  identities[table]=detail(SCHEMA+'.'+table)['id']
 publication='weft-relationship-fixtures'
 c.sql('publish-fixture-vector','INSERT INTO '+MANIFEST+" VALUES (:id,'ashlar-delta/0.3',:vector,'{}',:revisions,:report,TIMESTAMP '2026-10-08 00:00:00')",[p('id',publication),p('vector',json.dumps({SCHEMA+'.'+t:v for t,v in versions.items()})),p('revisions',json.dumps({'relationship-fixture':'fixture-schema-1'})),p('report',json.dumps({'qualification':'synthetic relationship controls','payloadSha256':payload_sha}))])
 state=dict(phase='ready',payloadSha256=payload_sha,uuids=identities,versions=versions,publicationId=publication);state_path.write_text(json.dumps(state,indent=2)+'\n')
for t,uuid in state['uuids'].items():assert detail(SCHEMA+'.'+t)['id']==uuid
assert c.sql('resolve-fixture-vector','SELECT table_versions_json FROM '+MANIFEST+' WHERE publication_id=:id',[p('id',state['publicationId'])])==[[json.dumps({SCHEMA+'.'+t:v for t,v in state['versions'].items()})]]
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);binary_sha=hashlib.sha256(binary.read_bytes()).hexdigest();outcomes=[];guard_cache={};(OUT/'compile-artifacts.jsonl').write_text('')
def request(sql,label,edge,homes,multiplicity=None,params=None):
 def logical(element):return dict(documentId='relationship-fixture',revision='1',module='main',element=element)
 def scalar(id,family,facets=None):
  v=dict(id=id,name=id,kind='field',cardinality='one',nullability='required',scalarType=family,extensions={})
  if facets:v['facets']=facets
  return v
 source_key=dict(id='customer-pk',name='primary',primary=True,fields=[dict(module='main',element='customer_id')])
 target_key=dict(id='order-pk',name='primary',primary=True,fields=[dict(module='main',element='code'),dict(module='main',element='rank')])
 rel=dict(id='customer-orders',name='orders',inverse='customers',source=[dict(module='main',element='customer')],target=[dict(module='main',element='orders',key='order-pk')],directed=True,targetLifecycle='independent',sourceMultiplicity=dict(min=0,max='*'),targetMultiplicity=multiplicity or dict(min=0,max='*'))
 elements=[dict(id='customer',name='Customer',kind='record',members=[dict(module='main',element='customer_id')],keys=[source_key],extensions={}),dict(id='orders',name='Orders',kind='record',members=[dict(module='main',element='code'),dict(module='main',element='rank')],keys=[target_key],extensions={}),scalar('customer_id','integer',{'integerWidth':{'bits':64,'signed':True}}),scalar('code','string'),scalar('rank','integer',{'integerWidth':{'bits':8,'signed':True}})]
 document=json.dumps(dict(umf='0.7.0',id='relationship-fixture',vocabularies={},modules=[dict(id='main',namespace='relationships',elements=elements,relationships=[rel])],extensions={}))
 pin=dict(documentId='relationship-fixture',revision='1',umfVersion='0.7.0',sha256=hashlib.sha256(document.encode()).hexdigest());source='weft-relationship-'+label
 def home(field):
  if homes=='props':return dict(kind='props',propertyId={'customer_id':'23','code':'24','rank':'25'}[field])
  return dict(kind='column',value='group_value' if field=='code' else 'rank_value',present='group_present' if field=='code' else 'rank_present',nativeType='STRING' if field=='code' else 'BIGINT')
 tables=['node_type_a',edge]
 binding=json.dumps(dict(profile='ashlar-databricks-candidate/0.1.0',layoutRevision='ashlar-delta/0.3',layoutSha256=layout_sha,modelPins=[pin],publication=dict(id=state['publicationId'],manifestUuid=state['uuids']['publication_manifest'],tables=[dict(name=(SCHEMA+'.'+t).split('.'),uuid=state['uuids'][t],version=state['versions'][t]) for t in tables]),records=[dict(logical=logical(record),table=0,kind='nodeProjection',sourceSystem=source,typeId=type_id,schemaRevision='fixture-schema-1',properties=[dict(logical=logical(f),home=home(f)) for f in fields]) for record,type_id,fields in [('customer','1',['customer_id']),('orders','2',['code','rank'])]],relationships=[dict(logical=dict(documentId='relationship-fixture',revision='1',module='main',relationship='customer-orders'),acceptedDefinition=rel,table=1,kind='edge' if edge=='edge_current' else 'edgeProjection',sourceSystem=source,typeId='3',schemaRevision='fixture-schema-1',source=logical('customer'),target=logical('orders'))]))
 result=dict(interfaceVersion='weft-compile/0.2.0',dialect='weft-sql/0.2.0',sql=sql,modules=[dict(documentJson=document,pin=pin,selectedModuleIds=['main'])],target=dict(backendId='ashlar.databricks',backendVersion='0.1.0-candidate',targetProfile='dbsql-candidate',bindingJson=binding,bindingSha256=hashlib.sha256(binding.encode()).hexdigest()),options=dict(allowCandidate=True))
 if params:result['parameters']=params
 return result
queries=[]
for bound in [1,2,3,10]:queries.append(dict(id='forward-'+str(bound),sql='SELECT c.customer_id, RELATED_KEYS(c.orders, '+str(bound)+') AS related FROM Customer c',bound=bound))
queries.append(dict(id='inverse',sql='SELECT o.code, o.rank, RELATED_KEYS(o.customers, 2) AS related FROM Orders o',bound=2,inverse=True))
for key in [('é','-128'),('absent','1')]:queries.append(dict(id='exists-'+key[0],sql='SELECT c.customer_id FROM Customer c WHERE HAS_RELATED(c.orders, KEY(:code, :rank))',params=dict(code=dict(family='string',value=key[0]),rank=dict(family='integer',value=key[1])),exists=key))
for edge in ['edge_current','edge_ab']:
 for homes in ['props','column']:
  for query in queries+[dict(id=label,sql='SELECT RELATED_KEYS(c.orders, 2) AS related FROM Customer c WHERE c.customer_id = 999',label=label) for label in labels if label!='exact']+[dict(id='maximum',sql='SELECT RELATED_KEYS(c.orders, 2) AS related FROM Customer c',multiplicity=dict(min=0,max=1)),dict(id='minimum',sql='SELECT RELATED_KEYS(c.orders, 2) AS related FROM Customer c',multiplicity=dict(min=1,max='*'))]:
   label=query.get('label','exact');identifier=edge+'-'+homes+'-'+query['id'];req=request(query['sql'],label,edge,homes,query.get('multiplicity'),query.get('params'))
   artifact=json.loads(subprocess.run([str(binary)],input=json.dumps(req),text=True,capture_output=True,check=True).stdout);assert artifact['status']=='compiled',(identifier,artifact)
   with (OUT/'compile-artifacts.jsonl').open('a') as stream:stream.write(json.dumps(dict(id=identifier,request=req,response=artifact),ensure_ascii=False)+'\n')
   params=[p('p'+str(s['position']),s['value']) for s in artifact['parameters']]
   checks=[check for o in artifact['obligations'] if o['id'] in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.relationshipIntegrity'] for check in o['parameters']['checks']]
   counts=[];guard_receipts=[]
   for i,check in enumerate(checks):
    used=set(re.findall(r':(p[0-9]+)\b',check['sql']))
    bound_params=[v for v in params if v['name'] in used]
    cache_key=hashlib.sha256(json.dumps(dict(sql=check['sql'],parameters=bound_params,fixture=state),sort_keys=True).encode()).hexdigest()
    if cache_key not in guard_cache:
     guard_label=identifier+'-guard-'+str(i)
     value=c.sql(guard_label,check['sql'],bound_params)
     assert len(value)==1 and len(value[0])==1,(guard_label,value)
     guard_cache[cache_key]=dict(value=value[0][0],label=guard_label)
    counts.append(guard_cache[cache_key]['value']);guard_receipts.append(guard_cache[cache_key]['label'])
   assert len(counts)==len(checks),(identifier,counts)
   corrupt=label!='exact' or 'multiplicity' in query
   if corrupt:
    assert any(int(v)>0 for v in counts),(identifier,counts)
    outcome=dict(id=identifier,outcome='refused-before-user-query',counts=counts,guardReceipts=guard_receipts)
   else:
    assert all(v=='0' for v in counts),(identifier,counts)
    actual=c.sql(identifier+'-user-query',artifact['sql'],params)
    if 'exists' in query:
     wanted=query['exists'];ids={src for _,src,dst in base_edges if next((code,str(rank)) for id,code,rank in orders if id==dst)==wanted}
     expected=sorted([[str(key)] for id,key in customers if id in ids]);assert sorted(actual)==expected,(identifier,actual,expected)
    else:
     bound=query['bound'];expected=[]
     if query.get('inverse'):
      for id,code,rank in orders:
       tuples=sorted([[str(next(k for i,k in customers if i==src))] for _,src,dst in base_edges if dst==id],key=lambda k:int(k[0]))
       expected.append([code,str(rank),dict(items=tuples[:bound],truncated=len(tuples)>bound)])
     else:
      for id,key in customers:
       tuples=sorted([[code,str(rank)] for _,src,dst in base_edges if src==id for oid,code,rank in orders if oid==dst],key=lambda k:(k[0],int(k[1])))
       expected.append([str(key),dict(items=tuples[:bound],truncated=len(tuples)>bound)])
     decoded=[row[:-1]+[json.loads(row[-1])] for row in actual]
     assert sorted(decoded,key=lambda row:tuple(row[:-1]))==sorted(expected,key=lambda row:tuple(row[:-1])),(identifier,decoded,expected)
     assert artifact['columns'][-1]['representation']['kind']=='relatedKeys'
    assert all(x['type_name']=='STRING' for x in c.records[-1]['response']['manifest']['schema']['columns'])
    outcome=dict(id=identifier,outcome='published-fixture-result',rows=actual,expected=expected,guardReceipts=guard_receipts)
   outcomes.append(outcome)
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha
summary=dict(uniqueNativeGuards=len(guard_cache),guardReuse='Only identical SQL, used parameter values and immutable fixture publication; each outcome links the actual guard statement',state='passed',cases=len(outcomes),positive=sum(o['outcome']=='published-fixture-result' for o in outcomes),refusals=sum(o['outcome']=='refused-before-user-query' for o in outcomes),outcomes=outcomes,fixture=state,compilerBinarySha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Actual canonical/serving edge SQL and props/typed key homes; independent forward/inverse parallel bags, bounded lookahead, EXISTS, orphan/type/key/edge/multiplicity refusals. Synthetic admin fixtures; no production policy/publication qualification.')
(OUT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'}))
