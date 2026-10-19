# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual same-object compiler -> PG18 SQL refinement over declared private values.
Not current native admission, UMF decision correctness, caller isolation or release.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,hashlib,json,pathlib,sys
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'))
from truss import LocalPostgres
import psycopg,weft
root=pathlib.Path(__file__).resolve().parents[1];source=json.loads((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_text());checks=[]
def identity(element):return {'documentId':'domain','revision':'query-value-core-r1','module':'m','element':element}
def binding(parent,child):
 lineage=json.loads(parent.json());plan=json.loads(lineage['applicationPlanJson']);d=json.loads(child.designation_json());values=[];designations={(v['path'],v['scan']):v for v in d['uses']}
 for use in lineage['uses']:
  action=designations[(use['path'],use['scan'])]['originalAction']
  values.append({'path':use['path'],'scan':use['scan'],'field':use['fieldIdentity'],'column':use['fieldIdentity']['element'],'domain':use['fieldIdentity'],'comparisonScope':'a'*64,'mode':'original-authorized' if action else 'disclosed','originalAction':action,'exposure':None})
 for field in lineage['readRequirements']['fields']:
  if field['role']!='projection':continue
  salary=field['fieldIdentity']['element']=='salary'
  values.append({'path':field['path'],'scan':field['scan'],'field':field['fieldIdentity'],'column':'salary_public' if salary else field['fieldIdentity']['element'],'domain':identity('salaryMaskLabel') if salary else field['fieldIdentity'],'comparisonScope':'a'*64,'mode':None,'originalAction':None,'exposure':'transformed' if salary else 'original'})
 scans=[{'scan':s['occurrence'],'target':s['record'],'relation':{'schema':'private_stage','table':'current_values','eligibleColumn':'eligible'}} for s in [plan['source']]+[j['right'] for j in plan['joins']]]
 return {'version':'weft.postgresql.current-values/0.1-candidate','sourceRequestSha256':hashlib.sha256(parent.source_request_json().encode()).hexdigest(),'designationSha256':hashlib.sha256(child.designation_json().encode()).hexdigest(),'sourceBindingSha256':d['binding']['sha256'],'scans':scans,'values':values}
# Eligibility, disclosure modes and scope tags below are explicit test premises.
# They are NOT source-derived UMF/native admission and are never reported as such.
cases=[
 ('masked-read-raw-predicate',source['sql'],{},[('[private]','opaque-z'),('[private]','opaque-a')],False),
 ('self-join-distinct','SELECT COUNT(DISTINCT s.token) FROM Resource r JOIN Resource s ON r.token=s.token',{},[('2',)],False),
 ('self-join-bag-count','SELECT COUNT(*) FROM Resource r JOIN Resource s ON r.token=s.token',{},[('5',)],False),
 ('group-order','SELECT r.token,COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token DESC',{},[('opaque-z','1'),('opaque-a','2')],False),
 ('hidden-group-count','SELECT COUNT(*) AS n FROM Resource r GROUP BY r.salary',{},[('1',),('2',)],True),
 ('false-zero-count','SELECT COUNT(*) AS n FROM Resource r WHERE FALSE LIMIT 0',{},[],False),
 ('original-predicate-zero','SELECT COUNT(*) AS n FROM Resource r WHERE r.salary=100 LIMIT 0',{},[],False),
 ('named-parameter','SELECT r.salary FROM Resource r WHERE r.salary=:minimum',{'minimum':{'family':'integer','value':'100'}},[('[private]',),('[private]',)],True),
 ('parameter-is-data','SELECT r.token FROM Resource r WHERE r.token=:token',{'token':{'family':'string','value':"opaque-a' OR TRUE --"}},[],False),
]
with LocalPostgres(a.output/'postgres') as runtime:
 with psycopg.connect(runtime.info.connection_uri,autocommit=True,cursor_factory=psycopg.RawCursor) as connection:
  version=connection.execute('SHOW server_version_num').fetchone()[0];assert version=='180006',version
  connection.execute('CREATE SCHEMA private_stage')
  connection.execute('CREATE TABLE private_stage.current_values(eligible boolean NOT NULL,"resourceId" text COLLATE "C" NOT NULL,salary bigint NOT NULL,salary_public text COLLATE "C" NOT NULL,token text COLLATE "C" NOT NULL)')
  connection.execute("INSERT INTO private_stage.current_values VALUES(TRUE,'r1',100,'[private]','opaque-a'),(FALSE,'hidden',100,'[private]','opaque-hidden'),(TRUE,'r3',100,'[private]','opaque-z'),(TRUE,'r4',200,'[private]','opaque-a')")
  connection.execute('SET default_transaction_read_only=on')
  for name,sql,params,expected,unordered in cases:
   request=json.loads(json.dumps(source));request['sql']=sql;request['parameters']=params;parent=weft.prepare_application_lineage(json.dumps(request,separators=(',',':')));child=parent.prepare_current_query();assert child.source_preparation() is parent
   raw_binding=json.dumps(binding(parent,child),separators=(',',':'));compiled=json.loads(child.lower_postgresql_scalar_candidate_json(raw_binding));assert compiled['authority'] is False and compiled['securityReady'] is False and compiled['nativeAdmission']=='required'
   values=[slot['value'] for slot in compiled['parameters']]
   cursor=connection.execute(compiled['sql'],values or None);rows=cursor.fetchall();assert all(c.type_code==25 for c in cursor.description)
   if unordered:rows=sorted(rows);expected=sorted(expected)
   assert rows==expected,(name,rows,expected)
   if name=='parameter-is-data':assert params['token']['value'] not in compiled['sql']
   if 'count' in name or 'distinct' in name:
    assert compiled['resultColumns'][0]['logicalType']['facets']=={},name
   checks.append({'id':name,'sqlSha256':hashlib.sha256(compiled['sql'].encode()).hexdigest(),'compiledSha256':hashlib.sha256(json.dumps(compiled,sort_keys=True).encode()).hexdigest(),'rows':rows,'nativeAdmissionStillRequired':True,'sameOriginalParent':True,'passed':True})
assert runtime._server is None and runtime._lease is None
receipt={'scope':'Actual same-object Weft typed compiler -> PG18 SQL refinement on synthetic declared per-use values only. Privileged fixture connection; no authentication/current-authority/UMF source decision/paired runtime proof or protected publication. Scope/mode/eligibility are premises, including counterfactual source modes. Not original native backend acceptance.','postgresVersionNum':version,'weftVersion':weft.__version__,'psycopgVersion':psycopg.__version__,'serverStopped':True,'checks':checks}
(a.output/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'status':'passed','checks':len(checks),'serverStopped':True}))
