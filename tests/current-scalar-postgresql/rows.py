"""Same-original typed Python packet and private SQL source-row refinement.
Synthetic fixture declarations; no native policy/caller/current-authority admission.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,hashlib,json,pathlib,sys
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'))
from truss import LocalPostgres
import psycopg,weft
root=pathlib.Path(__file__).resolve().parents[1]
source=json.loads((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_text());checks=[]
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
 return packet,{'version':'weft.postgresql.current-values/0.2-candidate','sourceRequestSha256':packet['sourceRequestSha256'],'designationSha256':packet['designationSha256'],'sourceBindingSha256':packet['binding']['sha256'],'scans':scans,'values':values}
rows=[(True,'k1','r1',100,'[private]','opaque-a'),(False,'k2','hidden',100,'[private]','opaque-hidden'),(True,'k3','r3',100,'[private]','opaque-z'),(True,'k4','r4',200,'[private]','opaque-a')]
original={r[1]:{'resourceId':r[2],'salary':r[3],'salary_public':r[4],'token':r[5]} for r in rows}
cases=[
 ('mixed','SELECT r.salary FROM Resource r WHERE r.salary=100 ORDER BY r.token DESC',{},[(('[private]',),('k3',)),(('[private]',),('k1',))],False),
 ('selfjoin','SELECT a.token FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('opaque-a',),(x,y)) for x in ['k1','k4'] for y in ['k1','k4']]+[(('opaque-z',),('k3','k3'))],True),
 ('two-masked-cells','SELECT a.salary AS first_salary,b.salary AS second_salary FROM Resource a JOIN Resource b ON a.token=b.token',{},[(('[private]','[private]'),(x,y)) for x in ['k1','k4'] for y in ['k1','k4']]+[(('[private]','[private]'),('k3','k3'))],True),
 ('false','SELECT r.salary FROM Resource r WHERE FALSE AND r.salary=100',{},[],False),
 ('limit-zero','SELECT r.salary FROM Resource r WHERE r.salary=100 LIMIT 0',{},[],False),
 ('asc','SELECT r.token FROM Resource r ORDER BY r.token ASC',{},[(('opaque-a',),('k1',)),(('opaque-a',),('k4',)),(('opaque-z',),('k3',))],True),
 ('named','SELECT r.salary FROM Resource r WHERE r.salary=:minimum',{'minimum':{'family':'integer','value':'100'}},[(('[private]',),('k1',)),(('[private]',),('k3',))],True),
 ('quoted-data','SELECT r.token FROM Resource r WHERE r.token=:token',{'token':{'family':'string','value':"x' OR TRUE --"}},[],False),
]
runtime=LocalPostgres(a.output/'postgres')
with runtime:
 with psycopg.connect(runtime.info.connection_uri,autocommit=True,cursor_factory=psycopg.RawCursor) as connection:
  version=connection.execute('SHOW server_version_num').fetchone()[0];assert version=='180006'
  connection.execute('CREATE SCHEMA private_stage')
  connection.execute('CREATE TABLE private_stage.current_values (eligible boolean NOT NULL,private_key text COLLATE "C" NOT NULL UNIQUE,"resourceId" text COLLATE "C" NOT NULL,salary bigint NOT NULL,salary_public text COLLATE "C" NOT NULL,token text COLLATE "C" NOT NULL)')
  for row in rows:connection.execute('INSERT INTO private_stage.current_values VALUES ($1,$2,$3,$4,$5,$6)',row)
  connection.execute('SET default_transaction_read_only=on')
  for name,sql,parameters,expected,unordered in cases:
   request=dict(source,sql=sql,parameters=parameters);parent=weft.prepare_application_lineage(json.dumps(request,separators=(',',':')));child=parent.prepare_current_query();assert child.source_preparation() is parent
   packet,b=binding(child);assert packet['authority'] is False and packet['securityReady'] is False
   compiled=json.loads(child.lower_postgresql_current_rows_candidate_json(json.dumps(b,separators=(',',':'))));assert compiled['nativeAdmission']=='required' and compiled['securityReady'] is False
   assert packet['sourceRequestSha256']==hashlib.sha256(parent.source_request_json().encode()).hexdigest()
   scans=[s['scan'] for s in packet['scans']];width=len(compiled['resultColumns']);private=compiled['privateResultColumns'];assert len(private)==1 and private[0]['position']==width and private[0]['scans']==scans
   cursor=connection.execute(compiled['sql'],[p['value'] for p in compiled['parameters']] or None);assert all(c.type_code==25 for c in cursor.description);actual=[]
   for row in cursor.fetchall():
    assert len(row)==width+1;carrier=json.loads(row[-1]);assert set(carrier)=={'version','rows'} and carrier['version']=='weft.stage-row-tuple/0.1-candidate'
    assert len(carrier['rows'])==len(scans) and [r['scan'] for r in carrier['rows']]==scans
    for r in carrier['rows']:assert set(r)=={'scan','key'} and type(r['key']) is str and 0<len(r['key'].encode())<=256 and r['key'] in original and r['key']!='k2'
    keys={r['scan']:r['key'] for r in carrier['rows']};public=row[:width]
    for cell in compiled['publicCellSources']:
     field=cell['field']['element'];column='salary_public' if field=='salary' else field
     assert public[cell['position']]==str(original[keys[cell['scan']]][column])
    actual.append((public,tuple(r['key'] for r in carrier['rows'])))
   if unordered:actual=sorted(actual);expected=sorted(expected)
   assert actual==expected,(name,actual,expected)
   if name in ['false','limit-zero']:assert packet['routes'] and compiled['parameters']
   checks.append({'id':name,'rows':actual,'sourceRequestSha256':packet['sourceRequestSha256'],'sqlSha256':hashlib.sha256(compiled['sql'].encode()).hexdigest(),'passed':True})
# Exact native method signatures do not accept an imported packet or self.
for name,call in [('no-packet-import',lambda:child.native_request_json(packet)),('no-foreign-self',lambda:weft.CurrentQueryPreparation.native_request_json(None))]:
 try:call()
 except TypeError:checks.append({'id':name,'refused':True})
 else:raise AssertionError(name)
for sql in ['SELECT COUNT(*) FROM Resource r','SELECT COUNT(DISTINCT r.token) FROM Resource r','SELECT r.token FROM Resource r GROUP BY r.token']:
 parent=weft.prepare_application_lineage(json.dumps(dict(source,sql=sql,parameters={})));child=parent.prepare_current_query();_,b=binding(child)
 try:child.lower_postgresql_current_rows_candidate_json(json.dumps(b))
 except ValueError as error:assert str(error).startswith('WFT-CURRENT-SCALAR-SQL:');checks.append({'id':'aggregate-refusal:'+sql,'refused':True})
 else:raise AssertionError(sql)
assert runtime._server is None and runtime._lease is None
receipt={'scope':'Actual original typed Python computation packet and PG18 nonaggregate source-row contribution over synthetic declared staging. Privileged fixture connection. No authenticated source/caller/policy/current authority, native runtime pairing or guarded public release.','postgresVersionNum':version,'checks':checks,'serverStopped':True,'nativeAdmission':False,'securityReady':False}
(a.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','controls':len(checks),'serverStopped':True}))
