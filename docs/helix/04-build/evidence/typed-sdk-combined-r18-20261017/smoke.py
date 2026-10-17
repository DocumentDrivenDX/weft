import json,hashlib,pathlib,zipfile,weft
root=pathlib.Path('/private/tmp/weft-sdk-prefix-20261016');out=pathlib.Path('/private/tmp/weft-combined-sdk-local-r18')
wheel=out/'wheels/weft_sql-0.4.42.dev0-cp39-abi3-macosx_11_0_arm64.whl';sha=lambda b:hashlib.sha256(b).hexdigest();results=[]
r=json.loads((root/'tests/raw-read/postgresql-projection-request.json').read_text())
def compile(label,req,expected='compiled'):
 p=json.loads(weft.compile_raw_read_json(json.dumps(req)));assert p['status']==expected,(label,p);results.append({'name':label,'status':'pass'});(out/(label+'.json')).write_text(json.dumps(p,indent=2)+'\n');return p
p=compile('full-projection',r);assert p['outputs'][2]['decoder']['profile']=='pgvector-text-float32/0.8.6';assert len(p['obligations']['nativeProjection']['columns'])==7
r['keyset']=True;r['streamPageSize']=True;r['binding']['schemaBinding']='qualified';p=compile('full-projection-stream',r);assert p['obligations']['streamPageSize']['maximum']==10
r['binding']['schemaBinding']='search-path';compile('schema-conflict',r,'blocked');r['binding']['schema']=None;compile('full-projection-search-path',r)
r['keyset']=False;compile('stream-nonkeyset-refused',r,'blocked')
for name in ['unity-ann-request','unity-hybrid-request','unity-shaped-request']:
 req=json.loads((root/'tests/native-search'/f'{name}.json').read_text());p=json.loads(weft.compile_unity_search_json(json.dumps(req)));assert p['status']=='compiled',(name,p);results.append({'name':name,'status':'pass'});(out/(name+'-compiled.json')).write_text(json.dumps(p,indent=2)+'\n')

import copy
def api(label,method,req):
 p=json.loads(getattr(weft,method)(json.dumps(req)));assert p['status']=='compiled',(label,p);results.append({'name':label,'status':'pass'});(out/(label+'.json')).write_text(json.dumps(p,indent=2)+'\n');return p
source=json.loads((root/'tests/raw-postgresql/read-request.json').read_text());source['parameters']=[];source['sql']="SELECT o.id AS id FROM commerce.\"Order\" o WHERE o.status='open' ORDER BY o.id DESC"
target=copy.deepcopy(source);target['sql']='SELECT c.id AS id, c.name AS name FROM commerce."Customer" c ORDER BY c.id LIMIT 10'
related={'version':'weft.related-read/0.1','source':source,'target':target,'relationship':{'module':'commerce','id':'customer','direction':'forward'}}
p=api('related-forward','compile_related_read_json',related);assert p['relatedRequest']==related;assert 'LIMIT' not in p['relatedSourceSql']
related['source'],related['target']=related['target'],related['source'];related['relationship']['direction']='inverse';api('related-inverse','compile_related_read_json',related)
spark=json.loads((root/'tests/spark-pages/read-request.json').read_text());api('spark-page','compile_raw_read_json',spark);spark['streamPageSize']=True;p=api('spark-stream','compile_raw_read_json',spark);assert p['obligations']['streamPageSize']['maximum']>0
action=json.loads((root/'tests/native-actions/action-request.json').read_text());action['target']='databricks-managed-delta';action['binding']['schema']='managed_fixture';api('managed-action','compile_managed_action_json',{'version':'weft.managed-action/0.1','catalog':'client_dev','action':action})
bulk={'version':'weft.managed-bulk/0.1','catalog':'client_dev','bulk':{'version':'weft.native-bulk/0.1','action':action,'selection':{'version':'weft.raw-read/0.1','target':'databricks-managed-delta','binding':action['binding'],'sql':'SELECT o.id AS id, o.status AS status FROM sales.order AS o ORDER BY o.id ASC','parameters':[],'rowCap':11,'keyset':False},'maxItems':10,'actionInputs':{},'parameters':[]}}
api('managed-bulk','compile_managed_bulk_json',bulk)

# Source-owned wildcard expansion retains original packet custody.
source['sql']='SELECT o.* FROM commerce."Order" o ORDER BY o.id'
target['sql']='SELECT c.* FROM commerce."Customer" c ORDER BY c.id LIMIT 10'
related={'version':'weft.related-read/0.1','source':source,'target':target,'relationship':{'module':'commerce','id':'customer','direction':'forward'}}
p=api('related-wildcards','compile_related_read_json',related);assert p['relatedRequest']==related;assert p['obligations']['nativeRelationships']['relationships']
base=json.loads((root/'tests/raw-postgresql/read-request.json').read_text());base['parameters']=[]
for label,predicate in [('negative-literal','o.customer_id = -1'),('literal-null-membership','o.customer_id NOT IN(-1,NULL)'),('empty-membership','NOT IN_EMPTY(o.status)'),('has-literal-key','HAS_RELATED(o.customer,KEY(-1))')]:
 req=copy.deepcopy(base);req['sql']=f'SELECT o.id AS id FROM commerce."Order" o WHERE {predicate} LIMIT 10';compile(label,req)
req=copy.deepcopy(base);req['parameters']=[{'name':'ids','carrier':'integer','nullable':False,'container':'list','elementNullable':True,'maxItems':1000}];req['sql']='SELECT o.id AS id FROM commerce."Order" o WHERE o.customer_id IN(:ids) LIMIT 10';p=compile('bound-list',req);assert 'pg_catalog.unnest' in p['sql']
req=copy.deepcopy(base);req['sql']='SELECT o.id AS id, RELATED_KEYS(o.customer,2) AS keys FROM commerce."Order" o LIMIT 10';p=compile('related-keys',req);assert p['outputs'][1]['decoder']['profile']=='pg-related-keys-json/0.1'
engine=json.loads((root/'tests/engine-catalog/native-ashlar-related-request.json').read_text());p=api('ashlar-related-keys','compile_engine_read_json',{'version':'weft.engine-read/0.1','compiler':'ashlar-spark4','request':engine});assert p['outputs'][1]['decoder']['representation']==p['artifact']['columns'][1]['representation']


spark_list=json.loads((root/'tests/spark-pages/read-request.json').read_text());spark_list['snapshots']={'orders':1};spark_list['parameters']=[{'name':'values','carrier':'string','nullable':False,'container':'list','elementNullable':True,'maxItems':1000}];spark_list['sql']='SELECT o.id AS id FROM commerce."Order" o WHERE o.status IN(:values) ORDER BY o.id LIMIT 10'
p=compile('spark-string-list',spark_list);assert p['obligations']['listPredicateParameters']['version']=='weft.spark-list-json/0.1';assert 'from_json' in p['sql']
for field,carrier in [('big','integer'),('amount','decimal')]:
 req=json.loads((root/'tests/spark-pages/full-shape-list-request.json').read_text());req['parameters']=[{'name':'values','carrier':carrier,'nullable':False,'container':'list','elementNullable':True,'maxItems':1000}];selection,ordering=req['sql'].split(' ORDER BY ',1);req['sql']=selection+f' WHERE o.{field} IN(:values) ORDER BY '+ordering;p=compile('spark-'+carrier+'-list',req);assert 'numericPredicateParameters' not in p['obligations'];assert 'ARRAY<STRING>' in p['sql']
engine_nulls=copy.deepcopy(engine);engine_nulls['sql']+=' ASC NULLS LAST';p=api('ashlar-required-nulls','compile_engine_read_json',{'version':'weft.engine-read/0.1','compiler':'ashlar-spark4','request':engine_nulls});assert p['request']==engine_nulls


# New R14 source-owned metadata, entity-set navigation and delivery controls.
for method in ['model_requirements_json','graph_requirements_json','lower_postgresql_current_presence_candidate_json','check_postgresql_current_presence_candidate_json']:
 assert callable(getattr(weft.CurrentPresencePreparation,method)),method
results.append({'name':'retained-graph-export-closure','status':'pass'})
bundle=json.loads((root/'tests/truss-query-abi/fixtures/native-query-bundle.json').read_text());raw=bundle['bindingJson']
truss={'interfaceVersion':'weft-compile/0.2.6','dialect':'weft-sql/0.2.0','timestampComparisonProfile':'umf-rfc3339-exact-instant/1.0.0','modules':bundle['modules'],'sql':'SELECT RELATED_KEYS(a.children,4) AS related FROM "A" a','parameters':{},'exactParameters':{},'options':{'allowCandidate':True},'target':{'backendId':'truss.postgresql.runtime','backendVersion':'0.4.39','targetProfile':'truss-runtime-pg18.6/0.5','bindingJson':raw,'bindingSha256':sha(raw.encode())}}
p=api('truss-related-key-metadata','compile_engine_read_json',{'version':'weft.engine-read/0.1','compiler':'truss-runtime','request':truss});assert p['outputs'][0]['decoder']['profile']=='truss-related-keys/0.1'
r=json.loads((root/'tests/engine-catalog/native-ashlar-related-page-request.json').read_text());p=api('ashlar-related-key-page','compile_engine_page_json',{'version':'weft.engine-page/0.1','compiler':'ashlar-spark4-pages','request':r,'pageSize':2});assert p['outputs'][1]['carrier']=='related-keys'
for grouped in [False,True]:
 r=json.loads((root/'tests/native-actions/action-request.json').read_text())['binding'];r['schema']='sdk_spark'
 sql='SELECT APPROX_COUNT_DISTINCT(o.status) AS estimated FROM sales.order o' if not grouped else 'SELECT o.status AS status, APPROX_COUNT_DISTINCT(o.id) AS estimated FROM sales.order o GROUP BY o.status ORDER BY estimated DESC LIMIT 3'
 req={'version':'weft.raw-read/0.1','target':'spark-delta4','sql':sql,'parameters':[],'rowCap':10,'keyset':False,'binding':r,'snapshots':{'orders':7}};p=compile('spark-hll-grouped' if grouped else 'spark-hll-global',req);assert p['outputs'][-1]['approximation']['error']=={'kind':'relative-standard-deviation','value':'0.05','hardBound':False}
source=copy.deepcopy(engine);source['parameters']={'status':{'family':'string','value':'paid'}};source['sql']='SELECT r.id AS id FROM sales."order" r WHERE r.status=:status ORDER BY r.id LIMIT 2'
target=copy.deepcopy(source);target['sql']='SELECT c."customerId" AS id FROM sales.customer c ORDER BY c."customerId" LIMIT 10'
packet={'version':'weft.engine-related-read/0.1','compiler':'ashlar-spark4','source':source,'target':target,'relationship':{'module':'sales','id':'customers','direction':'forward'}}
p=api('ashlar-engine-related','compile_engine_related_read_json',packet);assert p['relatedRequest']==packet;assert len({o['id'] for o in p['artifact']['obligations']})==len(p['artifact']['obligations']);assert json.loads(weft.check_engine_read_json(json.dumps(p),json.dumps(target)))==p
page=copy.deepcopy(packet);page['version']='weft.engine-related-page/0.1';page['compiler']='ashlar-spark4-pages';page['pageSize']=2
for side in ['source','target']:page[side]['interfaceVersion']='weft-compile/0.2.2';page[side]['target']['backendVersion']='0.1.0-local-spark4-pages-candidate'
page['target']['sql']='SELECT c."customerId" AS id FROM sales.customer c'
p=api('ashlar-engine-related-page','compile_engine_related_page_json',page);q=json.loads(weft.check_engine_page_json(json.dumps(p),json.dumps(page['target']),'["customer-1"]'));assert q['status']=='compiled' and q['artifact']['logicalPlan']['navigation']==p['artifact']['logicalPlan']['navigation'];results.append({'name':'ashlar-related-native-continuation','status':'pass'})
source=copy.deepcopy(truss);source['parameters']={'needle':{'family':'string','value':'first'}};source['sql']='SELECT a."A-label" AS label FROM "A" a WHERE a."A-label"=:needle ORDER BY a."A-label" LIMIT 2'
target=copy.deepcopy(source);target['sql']='SELECT b."B-label" AS label FROM "B" b ORDER BY b."B-label" LIMIT 10'
module=json.loads(source['modules'][0]['documentJson'])['modules'][0]['id'];packet={'version':'weft.engine-related-read/0.1','compiler':'truss-runtime','source':source,'target':target,'relationship':{'module':module,'id':'A-B','direction':'forward'}}
p=api('truss-engine-related','compile_engine_related_read_json',packet);assert p['request']==target and p['relatedRequest']==packet
page=copy.deepcopy(packet);page['version']='weft.engine-related-page/0.1';page['pageSize']=2;page['target']['sql']='SELECT b."B-label" AS label FROM "B" b'
# Compiler-only ABI1.1 derived fixture; not a native accepted publication.
header=json.loads(json.loads((root/'tests/engine-catalog/native-truss-export-request.json').read_text())['target']['bindingJson']);binding=json.loads(page['target']['target']['bindingJson'])
for key in ['releaseId','releaseSha256','queryAbi']:binding[key]=header[key]
raw=json.dumps(binding,separators=(',',':'))
for side in ['source','target']:page[side]['interfaceVersion']='weft-compile/0.2.7';page[side]['target']['backendVersion']='0.4.41';page[side]['target']['bindingJson']=raw;page[side]['target']['bindingSha256']=sha(raw.encode())
p=api('truss-engine-related-page','compile_engine_related_page_json',page);q=json.loads(weft.compile_engine_export_json(json.dumps(p),json.dumps(page['target'])));assert q['status']=='compiled',q;assert q['artifact']['logicalPlan']['limit'] is None and q['artifact']['logicalPlan']['navigation']==p['artifact']['logicalPlan']['navigation'];(out/'truss-engine-related-export.json').write_text(json.dumps(q,indent=2)+'\n');results.append({'name':'truss-related-original-cursor','status':'pass'})
for field,value in [('id','missing'),('direction','sideways')]:
 invalid=copy.deepcopy(packet);invalid['relationship'][field]=value;p=json.loads(weft.compile_engine_related_read_json(json.dumps(invalid)));assert p['status']=='blocked';results.append({'name':'related-'+field+'-refusal','status':'pass'})

fields=json.loads((root/'tests/engine-catalog/ashlar-fields-only-request.json').read_text())
p=api('ashlar-fields38','compile_engine_read_json',{'version':'weft.engine-read/0.1','compiler':'ashlar-spark4-fields','request':fields})
assert p['artifact']['columns'][4]['representation']['logicalType']['facets']=={'precision':38,'scale':6}
q=json.loads(weft.check_engine_read_json(json.dumps(p),json.dumps(fields)));assert q==p;results.append({'name':'ashlar-fields38-checked','status':'pass'})
bad=copy.deepcopy(fields);bad['sql']+=" WHERE 1=1";q=json.loads(weft.check_engine_read_json(json.dumps(p),json.dumps(bad)));assert q['status']=='blocked';results.append({'name':'ashlar-fields38-rebind-refusal','status':'pass'})

owner=json.loads((root/'docs/helix/04-build/evidence/native-privilege-probes-source-20261017/actual-owner-probe.json').read_text())
packet={'version':'weft.native-privilege-probes/0.1','request':owner['request'],'program':owner['program'],'probeTable':owner['probeTable']}
p=api('native-privilege-probes','compile_native_privilege_probes_json',packet);assert p==owner
q=api('native-privilege-probes-checked','check_native_privilege_probes_json',p);assert q==p and q['complete'] is False
altered=copy.deepcopy(p);altered['statements'][0]['probes'][0]['sql']+=' AND FALSE';q=json.loads(weft.check_native_privilege_probes_json(json.dumps(altered)));assert q['status']=='blocked';results.append({'name':'native-privilege-probes-forgery-refusal','status':'pass'})


# Exact actual SDK original compiler calls, including RRF legs and action branches.
for category in ['search','action']:
 records=json.loads((out/(category+'-original-compile-calls.json')).read_text())
 for index,record in enumerate(records):
  if record['compilerApi']=='compile_raw_read_json':continue
  request=json.loads(record['requestText']);original=json.loads(record['programText'])
  assert json.loads(getattr(weft,record['compilerApi'])(record['requestText']))==original
  packet={'version':'weft.native-privilege-probes/0.1','request':request,'program':original,'probeTable':'__weft_privilege_actual'}
  label='original-'+category+'-'+str(index)
  p=api(label,'compile_native_privilege_probes_json',packet)
  assert p['complete'] is False and p['nativeAuthority'] is False
  assert p['program']==original
  q=api(label+'-checked','check_native_privilege_probes_json',p);assert p==q
  for statement in p['statements']:
   for probe in statement['probes']:
    if statement['role'] in ['action-effect','bulk-update']:
     assert probe['analysisKind']=='pg-rewrite-rule/0.1' and probe['analysisRule']['neverFire'] is True
     assert probe['setupStatements'][-2].endswith('WITH NO DATA')
# New owner bulk list slots retain every original facet; scalar source stays unchanged.
record=next(x for x in records if x['compilerApi']=='compile_native_bulk_json')
request=json.loads(record['requestText'])
parameter={'name':'statuses','carrier':'string','nullable':False,'container':'list','elementNullable':True,'maxItems':8}
request['parameters']=[parameter];request['selection']['parameters']=[copy.deepcopy(parameter)]
request['selection']['sql']='SELECT o.id AS id,o.status AS status FROM sales.order o WHERE o.status IN(:statuses) ORDER BY o.id ASC'
p=api('bulk-list-owner-facets','compile_native_bulk_json',request)
for statement in ['preview','selection']:
 slot=p[statement]['parameters'][0]
 assert all(slot[k]==parameter[k] for k in ['carrier','nullable','container','elementNullable','maxItems'])
 assert slot['source']=={'bulkInput':'statuses'}
packet={'version':'weft.native-privilege-probes/0.1','request':request,'program':p,'probeTable':'__weft_privilege_list'}
p=api('bulk-list-owner-probes','compile_native_privilege_probes_json',packet)
for field,value in [('container',None),('elementNullable',False),('maxItems',7),('nullable',True)]:
 bad=copy.deepcopy(request);bad['parameters'][0][field]=value
 q=json.loads(weft.compile_native_bulk_json(json.dumps(bad)));assert q['status']=='blocked',(field,q)
 results.append({'name':'bulk-list-'+field+'-mismatch-refused','status':'pass'})

files={};z=zipfile.ZipFile(wheel)
for name in z.namelist():
 if name.startswith('weft/') and (name.endswith('.so')or name.endswith('__init__.py')):
  data=z.read(name);installed=pathlib.Path(weft.__file__).parent/name.split('/',1)[1];assert installed.read_bytes()==data;files[name]=sha(data)
receipt={'sourceCommit':'f908e69c596257850f2389cb9764a8b39780ae27','wheel':str(wheel),'wheelSha256':sha(wheel.read_bytes()),'payloads':files,'controls':results,'controlCount':len(results),'temporalQualification':'held','managedNativeQualification':False}
(out/'pins.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt))
