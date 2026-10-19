# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual typed contributor-bag SQL refinement, not native policy admission.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,hashlib,json,pathlib,sys
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'))
from truss import LocalPostgres
import psycopg,weft
root=pathlib.Path(__file__).resolve().parents[1];source=json.loads((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_text());checks=[]
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
 return packet,{'version':'weft.postgresql.current-values/0.3-candidate','sourceRequestSha256':packet['sourceRequestSha256'],'designationSha256':packet['designationSha256'],'sourceBindingSha256':packet['binding']['sha256'],'scans':scans,'values':values}
rows=[(True,'k1','r1',100,'[private]','opaque-a'),(False,'k2','hidden',100,'[private]','opaque-hidden'),(True,'k3','r3',100,'[private]','opaque-z'),(True,'k4','r4',200,'[private]','opaque-a')]
original={r[1]:{'resourceId':r[2],'salary':r[3],'salary_public':r[4],'token':r[5]} for r in rows}
one=[('k1',),('k3',),('k4',)];joined=[(x,y) for x in ['k1','k4'] for y in ['k1','k4']]+[('k3','k3')]
cases=[
 ('count','SELECT COUNT(*) AS n FROM Resource r',{},[(('3',),one)],False),
 ('distinct','SELECT COUNT(DISTINCT r.token) AS n FROM Resource r',{},[(('2',),one)],False),
 ('joined-count','SELECT COUNT(*) AS n FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('5',),joined)],False),
 ('joined-distinct','SELECT COUNT(DISTINCT a.token) AS n FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('2',),joined)],False),
 ('group','SELECT r.token,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC',{},[(('opaque-z','1'),[('k3',)]),(('opaque-a','2'),[('k1',),('k4',)])],False),
 ('joined-group','SELECT a.token,COUNT(*) AS n,COUNT(DISTINCT b.token) AS d FROM Resource a JOIN Resource b ON a.token=b.token GROUP BY a.token ORDER BY a.token DESC',{},[(('opaque-z','1','1'),[('k3','k3')]),(('opaque-a','4','1'),joined[:-1])],False),
 ('hidden-group','SELECT COUNT(*) AS n FROM Resource r GROUP BY r.salary',{},[(('2',),[('k1',),('k3',)]),(('1',),[('k4',)])],True),
 ('empty-global-count','SELECT COUNT(*) AS n FROM Resource r WHERE FALSE',{},[(('0',),[])],False),
 ('empty-global-distinct','SELECT COUNT(DISTINCT r.token) AS n FROM Resource r WHERE FALSE',{},[(('0',),[])],False),
 ('empty-group','SELECT r.token,COUNT(*) AS n FROM Resource r WHERE FALSE GROUP BY r.token',{},[],False),
 ('original-predicate','SELECT COUNT(*) AS n FROM Resource r WHERE r.salary=100',{},[(('2',),[('k1',),('k3',)])],False),
 ('zero-limit','SELECT COUNT(*) AS n FROM Resource r LIMIT 0',{},[],False),
 ('direct-singletons','SELECT r.salary FROM Resource r WHERE r.salary=100',{},[(('[private]',),[('k1',)]),(('[private]',),[('k3',)])],True),
 ('named-count','SELECT COUNT(*) AS n FROM Resource r WHERE r.salary=:minimum',{'minimum':{'family':'integer','value':'100'}},[(('2',),[('k1',),('k3',)])],False),
 ('parameter-data','SELECT COUNT(*) AS n FROM Resource r WHERE r.token=:token',{'token':{'family':'string','value':"x' OR TRUE --"}},[(('0',),[])],False),
 ('two-hidden-group-fields','SELECT COUNT(*) AS n FROM Resource r GROUP BY r.salary,r.token',{},[(('1',),[('k1',)]),(('1',),[('k3',)]),(('1',),[('k4',)])],True),
 ('integer-distinct','SELECT COUNT(DISTINCT r.salary) AS n FROM Resource r',{},[(('2',),one)],False),
]
def canonical(row,width,scans):
 assert len(row)==width+1;carrier=json.loads(row[-1]);assert set(carrier)=={'version','tuples'} and carrier['version']=='weft.stage-contributor-bag/0.1-candidate'
 bag=[]
 for item in carrier['tuples']:
  assert len(item)==len(scans) and [v['scan'] for v in item]==scans
  for v in item:assert set(v)=={'scan','key'} and type(v['key']) is str and 0<len(v['key'].encode())<=256 and v['key'] in original and v['key']!='k2'
  bag.append(tuple(v['key'] for v in item))
 return row[:width],sorted(bag)
def fixture_value(source,keys,values):
 declaration=values[(source['path'],source['scan'])];return original[keys[source['scan']]][declaration['column']]
runtime=LocalPostgres(a.output/'postgres')
with runtime:
 with psycopg.connect(runtime.info.connection_uri,autocommit=True,cursor_factory=psycopg.RawCursor) as connection:
  postgres_version=connection.execute('SHOW server_version_num').fetchone()[0];assert postgres_version=='180006'
  connection.execute('CREATE SCHEMA private_stage')
  connection.execute('CREATE TABLE private_stage.current_values (eligible boolean NOT NULL,private_key text COLLATE "C" NOT NULL UNIQUE,"resourceId" text COLLATE "C" NOT NULL,salary bigint NOT NULL,salary_public text COLLATE "C" NOT NULL,token text COLLATE "C" NOT NULL)')
  for row in rows:connection.execute('INSERT INTO private_stage.current_values VALUES ($1,$2,$3,$4,$5,$6)',row)
  connection.execute('SET default_transaction_read_only=on')
  for name,sql,parameters,expected,unordered in cases:
   parent=weft.prepare_application_lineage(json.dumps(dict(source,sql=sql,parameters=parameters),separators=(',',':')));child=parent.prepare_current_query();assert child.source_preparation() is parent
   packet,b=binding(child);compiled=json.loads(child.lower_postgresql_current_contributors_candidate_json(json.dumps(b,separators=(',',':'))));assert compiled['nativeAdmission']=='required' and compiled['authority'] is False and compiled['securityReady'] is False
   scans=[s['scan'] for s in packet['scans']];width=len(compiled['resultColumns']);meta=compiled['privateResultColumns'];assert len(meta)==1 and meta[0]['position']==width and meta[0]['scans']==scans and meta[0]['multiplicity']=='bag'
   values={(v['path'],v['scan']):v for v in b['values']};params=[p['value'] for p in compiled['parameters']] or None
   cursor=connection.execute(compiled['sql'],params);assert all(c.type_code==25 for c in cursor.description);actual=[]
   for row in cursor.fetchall():
    public,bag=canonical(row,width,scans);keys=[dict(zip(scans,item)) for item in bag]
    for group in compiled['privateGroupSources']:assert keys and len({fixture_value(group,key,values) for key in keys})==1
    for cell in compiled['publicCellSources']:
     value=public[cell['position']]
     if cell['kind']=='count':assert cell['sources']==[] and value==str(len(bag))
     elif cell['kind']=='count-distinct':assert len(cell['sources'])==1 and value==str(len({fixture_value(cell['sources'][0],key,values) for key in keys}))
     else:assert cell['kind'] in ['field','group-field'] and len(cell['sources'])==1 and keys and all(value==str(fixture_value(cell['sources'][0],key,values)) for key in keys)
     if cell['kind'] in ['count','count-distinct']:assert compiled['resultColumns'][cell['position']]['logicalType']['facets']=={}
    actual.append((public,bag))
   expected=[(public,sorted(bag)) for public,bag in expected]
   if unordered:actual=sorted(actual);expected=sorted(expected)
   assert actual==expected,(name,actual,expected)
   if name=='joined-distinct':
    wrong=compiled['sql'].replace('"s1"."private_key"','"s0"."private_key"');assert wrong!=compiled['sql'];bad=[canonical(row,width,scans) for row in connection.execute(wrong,params).fetchall()]
    assert [row[0] for row in bad]==[row[0] for row in actual] and bad!=actual
    checks.append({'id':'counterfactual-public-count-same-contributors-wrong','detected':True})
   checks.append({'id':name,'rows':actual,'sqlSha256':hashlib.sha256(compiled['sql'].encode()).hexdigest(),'compiledSha256':hashlib.sha256(json.dumps(compiled,sort_keys=True).encode()).hexdigest(),'passed':True})
# New versions never activate the predecessor row-only path, and vice versa.
for name,method,binding_version in [('old-path-rejects-v3',child.lower_postgresql_current_rows_candidate_json,'weft.postgresql.current-values/0.3-candidate'),('new-path-rejects-v2',child.lower_postgresql_current_contributors_candidate_json,'weft.postgresql.current-values/0.2-candidate')]:
 changed=dict(b,version=binding_version)
 try:method(json.dumps(changed))
 except ValueError as error:assert str(error).startswith('WFT-CURRENT-SCALAR-SQL:');checks.append({'id':name,'refused':True})
 else:raise AssertionError(name)
assert runtime._server is None and runtime._lease is None
receipt={'scope':'Actual same-original typed compiler and PG18 contributor-bag/cell refinement over synthetic declared stages. Privileged fixture role. Includes counterfactual source modes for salary grouping/distinct; not actual UMF policy/native authority/admission or guarded release.','postgresVersionNum':postgres_version,'checks':checks,'serverStopped':True,'nativeAdmission':False,'securityReady':False}
(a.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','controls':len(checks),'serverStopped':True}))
