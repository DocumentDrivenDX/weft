"""Actual same-original Python/Rust -> managed SQL, using synthetic stage premises.
No policy/native caller/source admission or Ashlar security claim.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,collections,hashlib,json,pathlib,subprocess,sys,time,uuid,zipfile
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);p.add_argument('--profile',default='aidev-cus');p.add_argument('--warehouse',default='2439e1f2e37ac563');a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
import weft
root=pathlib.Path(__file__).resolve().parents[1];checks=[];statements=[];schema='weft_b008_current_'+uuid.uuid4().hex;qualified='`client_dev`.`'+schema+'`.`current_values`';created=False;cleaned=False
sources={False:json.loads((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_text()),True:json.loads((root/'current-query-designation/fixtures/live-original-integer-sum.request.json').read_text())}
def sha(b):return hashlib.sha256(b).hexdigest()
def pins():
 modules=[m for n,m in sys.modules.items() if n.startswith('weft') and getattr(m,'__file__','').endswith('.so')];assert len(modules)==1
 with zipfile.ZipFile(a.wheel) as z:
  names=[n for n in z.namelist() if n.endswith('.so')];assert len(names)==1
  native=pathlib.Path(modules[0].__file__).read_bytes();shim=pathlib.Path(weft.__file__).read_bytes();assert native==z.read(names[0]) and shim==z.read('weft/__init__.py')
 return {'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native),'shimSha256':sha(shim),'runnerSha256':sha(pathlib.Path(__file__).read_bytes())}
before=pins()
def api(method,path,body=None):
 args=['databricks','api',method,path,'--profile',a.profile,'--output','json']
 if body is not None:
  f=a.output/'request.json';f.write_text(json.dumps(body));args+=['--json','@'+str(f)]
 r=subprocess.run(args,capture_output=True,text=True,timeout=45)
 if r.returncode:raise RuntimeError('Databricks CLI failed: '+r.stderr[-1500:])
 return json.loads(r.stdout)
def sql(statement,parameters=None,*,expect_error=False):
 req={'warehouse_id':a.warehouse,'statement':statement,'catalog':'client_dev','wait_timeout':'10s','disposition':'INLINE','format':'JSON_ARRAY'}
 if parameters:req['parameters']=parameters
 r=api('post','/api/2.0/sql/statements',req);end=time.monotonic()+180
 while r['status']['state'] in ('PENDING','RUNNING'):
  if time.monotonic()>end:raise TimeoutError(r['statement_id'])
  time.sleep(1);r=api('get','/api/2.0/sql/statements/'+r['statement_id'])
 record={'statement':statement,'parameters':parameters,'response':r};statements.append(record);(a.output/'statements.json').write_text(json.dumps(statements,indent=2)+'\n')
 if expect_error:assert r['status']['state']=='FAILED';return r
 assert r['status']['state']=='SUCCEEDED',r['status']
 manifest=r.get('manifest',{});result=r.get('result',{})
 assert not manifest.get('truncated',False) and not result.get('next_chunk_internal_link') and not result.get('next_chunk_external_link')
 assert result.get('row_count',0)==manifest.get('total_row_count',0)
 return r
maximum=9223372036854775807;minimum=-9223372036854775808
rows=[(True,'k1','r1',maximum,'[private]','opaque-a'),(False,'k2','hidden',17,'[private]','opaque-hidden'),(True,'k3','r3',maximum,'[private]','opaque-z'),(True,'k4','r4',minimum,'[private]','opaque-a'),(True,'k5','r5',minimum,'[private]','opaque-z')]
original={r[1]:dict(resourceId=r[2],salary=r[3],salary_public=r[4],token=r[5]) for r in rows};one=[('k1',),('k3',),('k4',),('k5',)];joined=[(x,y) for group in[['k1','k4'],['k3','k5']] for x in group for y in group]
cases=[
 ('count',False,'SELECT COUNT(*) AS n FROM Resource r',{},[(('4',),one)],False),
 ('distinct',False,'SELECT COUNT(DISTINCT r.token) AS n FROM Resource r',{},[(('2',),one)],False),
 ('integer-distinct',False,'SELECT COUNT(DISTINCT r.salary) AS n FROM Resource r',{},[(('2',),one)],False),
 ('group',False,'SELECT r.token,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC',{},[(('opaque-z','2'),[('k3',),('k5',)]),(('opaque-a','2'),[('k1',),('k4',)])],False),
 ('hidden-group',False,'SELECT COUNT(*) AS n FROM Resource r GROUP BY r.salary',{},[(('2',),[('k1',),('k3',)]),(('2',),[('k4',),('k5',)])],True),
 ('joined-count',False,'SELECT COUNT(*) AS n FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('8',),joined)],False),
 ('joined-distinct',False,'SELECT COUNT(DISTINCT a.token) AS n FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('2',),joined)],False),
 ('empty-count',False,'SELECT COUNT(*) AS n FROM Resource r WHERE FALSE',{},[(('0',),[])],False),
 ('empty-distinct',False,'SELECT COUNT(DISTINCT r.token) AS n FROM Resource r WHERE FALSE',{},[(('0',),[])],False),
 ('empty-group',False,'SELECT r.token,COUNT(*) AS n FROM Resource r WHERE FALSE GROUP BY r.token',{},[],False),
 ('limit-zero',False,'SELECT COUNT(*) AS n FROM Resource r LIMIT 0',{},[],False),
 ('masked-raw-parameter',False,'SELECT r.salary FROM Resource r WHERE r.salary=:bound',{'bound':{'family':'integer','value':str(maximum)}},[(('[private]',),[('k1',)]),(('[private]',),[('k3',)])],True),
 ('parameter-is-data',False,'SELECT COUNT(*) AS n FROM Resource r WHERE r.token=:token',{'token':{'family':'string','value':"x' OR TRUE --"}},[(('0',),[])],False),
 ('sum-total',True,'SELECT SUM(r.salary) AS total FROM Resource r',{},[(('-2',),one)],False),
 ('sum-beyond-positive-int64',True,'SELECT SUM(r.salary) AS total FROM Resource r WHERE r.salary=:bound',{'bound':{'family':'integer','value':str(maximum)}},[(('18446744073709551614',),[('k1',),('k3',)])],False),
 ('sum-beyond-negative-int64',True,'SELECT SUM(r.salary) AS total FROM Resource r WHERE r.salary=:bound',{'bound':{'family':'integer','value':str(minimum)}},[(('-18446744073709551616',),[('k4',),('k5',)])],False),
 ('sum-empty-null',True,'SELECT SUM(r.salary) AS total,COUNT(*) AS n FROM Resource r WHERE FALSE',{},[((None,'0'),[])],False),
 ('sum-empty-group',True,'SELECT r.token,SUM(r.salary) AS total FROM Resource r WHERE FALSE GROUP BY r.token',{},[],False),
 ('sum-group',True,'SELECT r.token,SUM(r.salary) AS total,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC',{},[(('opaque-z','-1','2'),[('k3',),('k5',)]),(('opaque-a','-1','2'),[('k1',),('k4',)])],False),
 ('sum-join',True,'SELECT SUM(a.salary) AS left_total,SUM(b.salary) AS right_total,COUNT(*) AS n FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('-4','-4','8'),joined)],False),
 ('sum-same-public-distinct-bags',True,'SELECT SUM(r.salary) AS total FROM Resource r GROUP BY r.token',{},[(('-1',),[('k1',),('k4',)]),(('-1',),[('k3',),('k5',)])],True),
 ('sum-with-distinct',True,'SELECT SUM(r.salary) AS total,COUNT(DISTINCT r.token) AS n FROM Resource r',{},[(('-2','2'),one)],False),
]
def identity(element):return {'documentId':'domain','revision':'query-value-core-r1','module':'m','element':element}
def binding(child,sum_context):
 packet=json.loads(child.native_request_json());values=[]
 for operation in packet['operations']:
  for operand in operation['operands']:
   v=operand['input']
   if v['kind']!='field':continue
   action=v['originalAction'];values.append(dict(path=operand['path'],scan=v['scan'],field=v['identity'],column=v['identity']['element'],domain=v['identity'],comparisonScope='a'*64,mode='original-authorized' if action else 'disclosed',originalAction=action,exposure=None))
 for projection in packet['projections']:
  field=projection['field'];salary=field['element']=='salary';values.append(dict(path=projection['path'],scan=projection['scan'],field=field,column='salary_public' if salary else field['element'],domain=identity('salaryMaskLabel') if salary else field,comparisonScope='a'*64,mode=None,originalAction=None,exposure='transformed' if salary else 'original'))
 scans=[dict(scan=s['scan'],target=s['record'],relation=dict(catalog='client_dev',schema=schema,table='current_values',uuid=table_id,version=version,schemaSha256=schema_sha,eligibleColumn='eligible',rowKeyColumn='private_key')) for s in packet['scans']]
 return packet,dict(version='weft.databricks.current-values/'+('0.2' if sum_context else '0.1')+'-candidate',sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],sourceBindingSha256=packet['binding']['sha256'],scans=scans,values=values)
def canonical(row,width,scans):
 assert len(row)==width+1;carrier=json.loads(row[-1]);assert set(carrier)=={'version','tuples'} and carrier['version']=='weft.stage-contributor-bag/0.1-candidate'
 bag=[]
 for item in carrier['tuples']:
  assert [x['scan'] for x in item]==scans
  for x in item:assert set(x)=={'scan','key'} and type(x['key']) is str and 0<len(x['key'].encode())<=256 and x['key'] in original and x['key']!='k2'
  bag.append(tuple(x['key'] for x in item))
 return tuple(row[:width]),sorted(bag)
try:
 sql('CREATE SCHEMA `client_dev`.`'+schema+'`');created=True
 sql('CREATE TABLE '+qualified+' (eligible BOOLEAN NOT NULL, private_key STRING NOT NULL, resourceId STRING NOT NULL, salary BIGINT NOT NULL, salary_public STRING NOT NULL, token STRING NOT NULL) USING DELTA')
 parameters=[];clauses=[]
 for i,row in enumerate(rows):
  expr=[]
  for j,value in enumerate(row):
   name='fixture_'+str(i)+'_'+str(j);ty='BOOLEAN' if type(value) is bool else 'BIGINT' if type(value) is int else 'STRING';parameters.append({'name':name,'value':str(value).lower() if type(value) is bool else str(value),'type':'STRING'});expr.append('CAST(:'+name+' AS '+ty+')')
  clauses.append('('+','.join(expr)+')')
 sql('INSERT INTO '+qualified+' VALUES '+','.join(clauses),parameters)
 detail=sql('DESCRIBE DETAIL '+qualified);columns=[c['name'] for c in detail['manifest']['schema']['columns']];native_detail=dict(zip(columns,detail['result']['data_array'][0]));table_id=native_detail['id']
 history=sql('DESCRIBE HISTORY '+qualified+' LIMIT 1');version=history['result']['data_array'][0][0]
 description=sql('DESCRIBE TABLE EXTENDED '+qualified+' AS JSON');schema_json=description['result']['data_array'][0][0];schema_sha=sha(schema_json.encode())
 for name,sum_context,query,params,expected,unordered in cases:
  parent=weft.prepare_application_lineage(json.dumps(dict(sources[sum_context],sql=query,parameters=params),separators=(',',':')));child=parent.prepare_current_query();assert child.source_preparation() is parent
  packet,b=binding(child,sum_context);method=child.lower_databricks_current_sum_candidate_json if sum_context else child.lower_databricks_current_contributors_candidate_json
  compiled=json.loads(method(json.dumps(b,separators=(',',':'))));assert compiled['authority'] is False and compiled['securityReady'] is False and compiled['nativeAdmission']=='required'
  native_parameters=[{'name':v['marker'],'value':v['value'],'type':'STRING'} for v in compiled['parameters']]
  response=sql(compiled['sql'],native_parameters);scans=[s['scan'] for s in packet['scans']];width=len(compiled['resultColumns']);assert all(c['type_name']=='STRING' for c in response['manifest']['schema']['columns']);assert len(response['manifest']['schema']['columns'])==width+1
  actual=[canonical(row,width,scans) for row in response.get('result',{}).get('data_array',[])];values={(v['path'],v['scan']):v for v in b['values']}
  def value(use,key):return original[key[use['scan']]][values[(use['path'],use['scan'])]['column']]
  for public,bag in actual:
   keys=[dict(zip(scans,item)) for item in bag]
   for group in compiled['privateGroupSources']:assert keys and len({value(group,key) for key in keys})==1
   for cell in compiled['publicCellSources']:
    observed=public[cell['position']]
    if cell['kind']=='count':assert cell['sources']==[] and observed==str(len(bag))
    elif cell['kind']=='count-distinct':assert len(cell['sources'])==1 and observed==str(len({value(cell['sources'][0],key) for key in keys}))
    elif cell['kind']=='sum':
     assert len(cell['sources'])==1;use=cell['sources'][0];declaration=values[(use['path'],use['scan'])];assert declaration['mode']=='original-authorized' and declaration['originalAction']=='query-original-salary-sum';assert observed==(str(sum(value(use,key) for key in keys)) if keys else None)
    else:assert cell['kind'] in ('field','group-field') and len(cell['sources'])==1 and keys and all(observed==str(value(cell['sources'][0],key)) for key in keys)
  expected=[(public,sorted(bag)) for public,bag in expected]
  if unordered:actual.sort();expected.sort()
  assert actual==expected,(name,actual,expected)
  if name=='joined-distinct':
   wrong=compiled['sql'].replace('`wft_s1`.`private_key`','`wft_s0`.`private_key`');assert wrong!=compiled['sql'];bad=sql(wrong,native_parameters);changed=[canonical(row,width,scans) for row in bad['result']['data_array']];assert [x[0] for x in changed]==[x[0] for x in actual] and changed!=actual;checks.append({'id':'same-count-wrong-contributor-detected','passed':True})
  checks.append({'id':name,'passed':True,'rows':actual,'compiled':compiled,'materializationBinding':b,'statementId':response['statement_id']});print(json.dumps({'id':name,'passed':True}),flush=True)
 # These are actual pure-compiler refusals, never native permission decisions.
 for context in (False,True):
  child=weft.prepare_application_lineage(json.dumps(dict(sources[context],sql='SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0',parameters={}))).prepare_current_query();_,b=binding(child,context)
  wrong=child.lower_databricks_current_contributors_candidate_json if context else child.lower_databricks_current_sum_candidate_json
  try:wrong(json.dumps(b))
  except ValueError:checks.append({'id':'no-context-downgrade:'+str(context),'refused':True})
  else:raise AssertionError('Unexpected mixed profile admission')
finally:
 if created:
  sql('DROP SCHEMA `client_dev`.`'+schema+'` CASCADE');remaining=sql("SHOW SCHEMAS IN client_dev LIKE '"+schema+"'");assert remaining.get('result',{}).get('data_array',[])==[] and remaining.get('manifest',{}).get('total_row_count',0)==0;cleaned=True
 after=pins();assert before==after
 receipt={'scope':'Actual original CQP Python/Rust conditional Databricks SQL over explicitly synthetic staging/mode/scope declarations. One approved test connection, not actual UMF native policy/caller/source admission, Ashlar accepted layout, two-caller isolation or guarded release.','checks':checks,'schema':schema,'schemaCreated':created,'schemaRemoved':cleaned,'pinsBeforeAfter':before,'nativeAdmission':False,'securityReady':False}
 (a.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'status':'passed','checks':len(checks),'schemaRemoved':cleaned}))
