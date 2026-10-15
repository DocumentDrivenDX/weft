"""Actual live-original integer SUM SQL/NULL/bag refinement; no native authority.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,hashlib,json,pathlib,sys,zipfile
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'))
from truss import LocalPostgres
import psycopg,weft
root=pathlib.Path(__file__).resolve().parents[1];source=json.loads((root/'current-query-designation/fixtures/live-original-integer-sum.request.json').read_text());checks=[]
def pins():
 native=[m for name,m in sys.modules.items() if name.startswith('weft') and getattr(m,'__file__','').endswith('.so')];assert len(native)==1
 module=pathlib.Path(native[0].__file__);shim=pathlib.Path(weft.__file__)
 with zipfile.ZipFile(a.wheel) as archive:
  names=[name for name in archive.namelist() if name.endswith('.so')];assert len(names)==1
  assert hashlib.sha256(module.read_bytes()).hexdigest()==hashlib.sha256(archive.read(names[0])).hexdigest()
  assert shim.read_bytes()==archive.read('weft/__init__.py')
 return {'wheel':hashlib.sha256(a.wheel.read_bytes()).hexdigest(),'module':hashlib.sha256(module.read_bytes()).hexdigest(),'shim':hashlib.sha256(shim.read_bytes()).hexdigest(),'runner':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
before=pins()
def identity(element):return {'documentId':'domain','revision':'query-value-core-r1','module':'m','element':element}
def binding(child):
 packet=json.loads(child.native_request_json());values=[]
 for operation in packet['operations']:
  for operand in operation['operands']:
   v=operand['input']
   if v['kind']!='field':continue
   action=v['originalAction'];values.append({'path':operand['path'],'scan':v['scan'],'field':v['identity'],'column':v['identity']['element'],'domain':v['identity'],'comparisonScope':'a'*64,'mode':'original-authorized' if action else 'disclosed','originalAction':action,'exposure':None})
 for projection in packet['projections']:
  field=projection['field'];salary=field['element']=='salary'
  values.append({'path':projection['path'],'scan':projection['scan'],'field':field,'column':'salary_public' if salary else field['element'],'domain':identity('salaryMaskLabel') if salary else field,'comparisonScope':'a'*64,'mode':None,'originalAction':None,'exposure':'transformed' if salary else 'original'})
 scans=[{'scan':s['scan'],'target':s['record'],'relation':{'schema':'private_stage','table':'current_values','eligibleColumn':'eligible','rowKeyColumn':'private_key'}} for s in packet['scans']]
 return packet,{'version':'weft.postgresql.current-values/0.4-candidate','sourceRequestSha256':packet['sourceRequestSha256'],'designationSha256':packet['designationSha256'],'sourceBindingSha256':packet['binding']['sha256'],'scans':scans,'values':values}
maximum=9223372036854775807;minimum=-9223372036854775808
rows=[(True,'k1',maximum,'opaque-a'),(False,'k2',17,'opaque-hidden'),(True,'k3',maximum,'opaque-z'),(True,'k4',minimum,'opaque-a'),(True,'k5',minimum,'opaque-z')]
original={r[1]:{'salary':r[2],'token':r[3]} for r in rows};all_keys=[('k1',),('k3',),('k4',),('k5',)]
joined=[(x,y) for group in [['k1','k4'],['k3','k5']] for x in group for y in group]
cases=[
 ('exact-total','SELECT SUM(r.salary) AS total FROM Resource r',{},[(('-2',),all_keys)],False),
 ('beyond-positive-int64','SELECT SUM(r.salary) AS total FROM Resource r WHERE r.salary=:bound',{'bound':{'family':'integer','value':str(maximum)}},[(('18446744073709551614',),[('k1',),('k3',)])],False),
 ('beyond-negative-int64','SELECT SUM(r.salary) AS total FROM Resource r WHERE r.salary=:bound',{'bound':{'family':'integer','value':str(minimum)}},[(('-18446744073709551616',),[('k4',),('k5',)])],False),
 ('empty-null','SELECT SUM(r.salary) AS total FROM Resource r WHERE FALSE',{},[((None,),[])],False),
 ('empty-null-with-count','SELECT SUM(r.salary) AS total,COUNT(*) AS n FROM Resource r WHERE FALSE',{},[((None,'0'),[])],False),
 ('empty-group','SELECT r.token,SUM(r.salary) AS total FROM Resource r WHERE FALSE GROUP BY r.token',{},[],False),
 ('zero-limit','SELECT SUM(r.salary) AS total FROM Resource r LIMIT 0',{},[],False),
 ('grouped','SELECT r.token,SUM(r.salary) AS total,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC',{},[(('opaque-z','-1','2'),[('k3',),('k5',)]),(('opaque-a','-1','2'),[('k1',),('k4',)])],False),
 ('joined','SELECT SUM(a.salary) AS left_total,SUM(b.salary) AS right_total,COUNT(*) AS n FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('-4','-4','8'),joined)],False),
 ('same-public-sum-distinct-bags','SELECT SUM(r.salary) AS total FROM Resource r GROUP BY r.token',{},[(('-1',),[('k1',),('k4',)]),(('-1',),[('k3',),('k5',)])],True),
 ('sum-and-distinct','SELECT SUM(r.salary) AS total,COUNT(DISTINCT r.token) AS distinct_tokens FROM Resource r',{},[(('-2','2'),all_keys)],False),
 ('count-capability-context','SELECT COUNT(*) AS n FROM Resource r',{},[(('4',),all_keys)],False),
]
def fixture_value(declaration,key):return original[key[declaration['scan']]][declaration['column']]
def native_refuses(name,call,code):
 try:call()
 except ValueError as error:assert str(error).startswith(code+':'),(name,str(error));checks.append({'id':name,'refused':True})
 else:raise AssertionError(name)
runtime=LocalPostgres(a.output/'postgres')
with runtime:
 with psycopg.connect(runtime.info.connection_uri,autocommit=True,cursor_factory=psycopg.RawCursor) as connection:
  postgres_version=connection.execute('SHOW server_version_num').fetchone()[0];assert postgres_version=='180006'
  connection.execute('CREATE SCHEMA private_stage')
  # No public mask column is needed for original-only SUM.
  connection.execute('CREATE TABLE private_stage.current_values (eligible boolean NOT NULL,private_key text COLLATE "C" NOT NULL UNIQUE,salary bigint NOT NULL,token text COLLATE "C" NOT NULL)')
  for row in rows:connection.execute('INSERT INTO private_stage.current_values VALUES ($1,$2,$3,$4)',row)
  connection.execute('SET default_transaction_read_only=on')
  for name,sql,parameters,expected,unordered in cases:
   parent=weft.prepare_application_lineage(json.dumps(dict(source,sql=sql,parameters=parameters),separators=(',',':')))
   try:child=parent.prepare_current_query()
   except ValueError as error:raise AssertionError((name,str(error))) from error
   assert child.source_preparation() is parent
   packet,b=binding(child);assert packet['version']=='weft.current-query.native-input/0.2-candidate' and packet['capabilities']==['live-original-integer-sum']
   for route in packet['routes']:
    if route['operation']=='sum':assert route['required']==['nullity']
   compiled=json.loads(child.lower_postgresql_current_sum_candidate_json(json.dumps(b,separators=(',',':'))));assert compiled['version']=='weft.postgresql.current-scalar/0.4-candidate' and compiled['nativeAdmission']=='required' and compiled['authority'] is False and compiled['securityReady'] is False
   expected_kinds={'empty-null-with-count':['sum','count'],'empty-group':['group-field','sum'],'grouped':['group-field','sum','count'],'joined':['sum','sum','count'],'sum-and-distinct':['sum','count-distinct'],'count-capability-context':['count']}.get(name,['sum'])
   grouped=name in ['empty-group','grouped','same-public-sum-distinct-bags']
   assert [cell['kind'] for cell in compiled['publicCellSources']]==expected_kinds
   expected_groups=[{'path':'groups/0','scan':'s0','field':identity('token')}] if grouped else []
   assert compiled['privateGroupSources']==expected_groups
   expected_sum_routes=[]
   for position,kind in enumerate(expected_kinds):
    if kind=='sum':expected_sum_routes.append({'scan':'s1' if name=='joined' and position==1 else 's0','path':f'outputs/{position}/argument','type':{'documentId':'domain','moduleId':'m','elementId':'Resource'},'field':{'documentId':'domain','moduleId':'m','elementId':'salary'},'operator':'aggregate','operation':'sum','required':['nullity']})
   assert [route for route in packet['routes'] if route['operation']=='sum']==expected_sum_routes
   sum_operations=[operation for operation in packet['operations'] if operation['kind']=='sum'];assert len(sum_operations)==len(expected_sum_routes)
   for operation,route in zip(sum_operations,expected_sum_routes):
    assert operation['path']==route['path'].removesuffix('/argument') and operation['outputType']=={'family':'integer','facets':{},'nullable':not grouped}
    assert len(operation['operands'])==1 and operation['operands'][0]['path']==route['path'] and operation['operands'][0]['input']['originalAction']=='query-original-salary-sum'
   if name=='count-capability-context':assert packet['routes']==[] and len(packet['operations'])==1 and packet['operations'][0]['kind']=='count'
   assert 'salary_public' not in compiled['sql'];scans=[s['scan'] for s in packet['scans']];width=len(compiled['resultColumns']);assert width==len(expected_kinds);values={(v['path'],v['scan']):v for v in b['values']}
   cursor=connection.execute(compiled['sql'],[p['value'] for p in compiled['parameters']] or None);assert all(c.type_code==25 for c in cursor.description);actual=[]
   for row in cursor.fetchall():
    assert len(row)==width+1;carrier=json.loads(row[-1]);assert set(carrier)=={'version','tuples'} and carrier['version']=='weft.stage-contributor-bag/0.1-candidate';bag=[]
    for item in carrier['tuples']:
     assert len(item)==len(scans) and [v['scan'] for v in item]==scans
     for v in item:assert set(v)=={'scan','key'} and type(v['key']) is str and 0<len(v['key'].encode())<=256 and v['key'] in original and v['key']!='k2'
     bag.append(tuple(v['key'] for v in item))
    bag.sort();keys=[dict(zip(scans,item)) for item in bag];public=row[:width]
    for group in compiled['privateGroupSources']:
     declaration=values[(group['path'],group['scan'])];assert keys and len({fixture_value(declaration,key) for key in keys})==1
    for cell in compiled['publicCellSources']:
     value=public[cell['position']]
     if cell['kind']=='count':assert cell['sources']==[] and value==str(len(bag))
     elif cell['kind']=='count-distinct':
      use=cell['sources'][0];declaration=values[(use['path'],use['scan'])];assert value==str(len({fixture_value(declaration,key) for key in keys}))
     elif cell['kind']=='sum':
      assert len(cell['sources'])==1;use=cell['sources'][0];declaration=values[(use['path'],use['scan'])];assert declaration['mode']=='original-authorized' and declaration['originalAction']=='query-original-salary-sum'
      expected_sum=str(sum(fixture_value(declaration,key) for key in keys)) if keys else None;assert value==expected_sum
      nullable=not grouped;result_type=compiled['resultColumns'][cell['position']];assert result_type['logicalType']=={'family':'integer','facets':{},'nullable':nullable} and result_type['encoding']==('native-numeric-integer-nullable-text' if nullable else 'native-numeric-integer-required-text')
     else:
      assert cell['kind'] in ['field','group-field'] and len(cell['sources'])==1;use=cell['sources'][0];declaration=values[(use['path'],use['scan'])];assert keys and all(value==str(fixture_value(declaration,key)) for key in keys)
    actual.append((public,bag))
   expected=[(public,sorted(bag)) for public,bag in expected]
   if unordered:actual.sort();expected.sort()
   assert actual==expected,(name,actual,expected)
   checks.append({'id':name,'rows':actual,'sqlSha256':hashlib.sha256(compiled['sql'].encode()).hexdigest(),'compiledSha256':hashlib.sha256(json.dumps(compiled,sort_keys=True).encode()).hexdigest(),'passed':True})
# Exact no-downgrade behavior, including a new-context COUNT without a SUM operand.
for version,method in [('0.1',child.lower_postgresql_scalar_candidate_json),('0.2',child.lower_postgresql_current_rows_candidate_json),('0.3',child.lower_postgresql_current_contributors_candidate_json)]:
 changed=json.loads(json.dumps(b));changed['version']='weft.postgresql.current-values/'+version+'-candidate'
 if version=='0.1':
  for scan in changed['scans']:del scan['relation']['rowKeyColumn']
 native_refuses('predecessor-refuses-new-context:'+version,lambda method=method,changed=changed:method(json.dumps(changed)),'WFT-CURRENT-SCALAR-SQL')
parent=weft.prepare_application_lineage(json.dumps(source));child=parent.prepare_current_query();packet,b=binding(child)
for name,edit in [('disclosed-sum',lambda v:v.update(mode='disclosed',originalAction=None)),('wrong-action',lambda v:v.update(originalAction='query-original-salary-predicate')),('mask-domain',lambda v:v.update(domain=identity('salaryMaskLabel')))]:
 changed=json.loads(json.dumps(b));edit(changed['values'][0]);native_refuses(name,lambda changed=changed:child.lower_postgresql_current_sum_candidate_json(json.dumps(changed)),'WFT-CURRENT-SCALAR-SQL')
old=json.loads((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_text());old['sql']='SELECT COUNT(*) AS n FROM Resource r';old_child=weft.prepare_application_lineage(json.dumps(old)).prepare_current_query();_,old_binding=binding(old_child)
native_refuses('new-consumer-refuses-legacy-context',lambda:old_child.lower_postgresql_current_sum_candidate_json(json.dumps(old_binding)),'WFT-CURRENT-SCALAR-SQL')
old['sql']='SELECT SUM(r.salary) AS total FROM Resource r';native_refuses('legacy-sum-factory-still-refuses',lambda:weft.prepare_application_lineage(json.dumps(old)).prepare_current_query(),'WFT-CURRENT-QUERY-OPERANDS')
assert runtime._server is None and runtime._lease is None;after=pins();assert before==after
receipt={'scope':'Actual explicit original integer SUM capability/nullable mathematical output/PG18 exact numeric accumulation and contributor-bag refinement over synthetic declared stages. Privileged fixture role; no actual native policy/caller/current authority, paired runtime proof or guarded public release.','postgresVersionNum':postgres_version,'checks':checks,'serverStopped':True,'pinsBeforeAfter':before,'nativeAdmission':False,'securityReady':False}
(a.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','controls':len(checks),'serverStopped':True}))
