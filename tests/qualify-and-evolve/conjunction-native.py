"""Native conjunction truth tables on both engine-pinned registrations.
@covers US-003-AC1 @covers US-004-AC1 @covers US-006-AC1
Owned PostgreSQL temporary fixtures and read-only Ashlar owner substitutions.
"""
import ast,copy,csv,hashlib,io,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(os.environ.get('WEFT_CONJUNCTION_OUT','/private/tmp/weft-b007-conjunction-native'));OUT.mkdir(parents=True,exist_ok=True)
BINARY=Path(os.environ.get('WEFT_CONJUNCTION_BINARY','/private/tmp/weft-b007-parser-target/debug/examples/compile_native_review'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
SEED=[(True,'hit'),(False,'hit'),(True,'miss'),(False,'miss'),(True,'hit')]
# Authored truth-table expectations; repeated true/hit rows preserve bag semantics.
TRUTH=[(True,'hit',2),(False,'hit',1),(True,'miss',1),(False,'miss',1)]
pg_cases=json.loads((ROOT/'tests/truss-postgresql/fixtures/application-cases.json').read_text())
ashlar_cases=[json.loads(l) for l in (ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-application-native/compile-artifacts.jsonl').read_text().splitlines()]
artifacts=[]
for backend in ['truss','ashlar']:
 for home in ['props','typed']:
  base=next(c for c in pg_cases if c['id']=='whole-entity-'+('props' if home=='props' else 'row')) if backend=='truss' else next(c for c in ashlar_cases if c['id']=='8-props-props-count')
  for dialect in ['0.1','0.2']:
   for active,name,count in TRUTH:
    request=copy.deepcopy(base['request']);request['dialect']='weft-sql/'+dialect+'.0';request['interfaceVersion']='weft-compile/'+dialect+'.0'
    request.pop('readProfile',None)
    entity='Customer' if backend=='truss' else 'Thing'
    projection='c.name' if dialect=='0.1' else 'COUNT(*) AS total'
    request['sql']=f"SELECT {projection} FROM {entity} c WHERE c.active = {str(active).upper()} AND c.name = '{name}'"
    request['target']['backendVersion']='0.1.0-native-review';request['target']['targetProfile']='pg17.9-native-review' if backend=='truss' else 'dbsql2026.39-native-review'
    if backend=='ashlar' and home=='typed':
     binding=json.loads(request['target']['bindingJson'])
     for p in binding['records'][0]['properties']:
      if p['logical']['element']=='name':
       p['home']={'kind':'column','value':'group_value','present':'group_present','nativeType':'STRING'}
     raw=json.dumps(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
    response=json.loads(subprocess.check_output([str(BINARY)],input=(json.dumps(request)+'\n').encode()))
    id=f'{backend}-{home}-{dialect}-{active}-{name}'
    assert response['status']=='compiled',(id,response)
    assert any(op['assessment']['id']=='and' for op in response['qualification']['operations']),id
    artifacts.append(dict(id=id,backend=backend,home=home,dialect=dialect,active=active,name=name,expectedCount=count,expectedRows=[[name]]*count if dialect=='0.1' else [[str(count)]],request=request,response=response,mapping=base.get('mapping')))
assert len(artifacts)==32
(OUT/'compile-artifacts.jsonl').write_text(''.join(json.dumps(a)+'\n' for a in artifacts))
if '--prepare-only' in sys.argv:
 print(json.dumps(dict(status='prepared',cases=32,compilerSha256=sha(BINARY))));sys.exit(0)
# Import only the fixture-authoring portion, never its execution loop.
harness=ROOT/'tests/truss-postgresql/application-native.py';module=ast.parse(harness.read_text().split('reports=[]\n',1)[0]);module.body=[n for n in module.body if not(isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id in {'BINARY','BINARY_SHA'} for t in n.targets))]
namespace={'__file__':str(harness)};exec(compile(module,str(harness),'exec'),namespace)
sys.path.insert(0,str(ROOT/'tests/ashlar-databricks'))
from native_transport import Client
client=Client(OUT/'ashlar');results=[];pg_records=[]
engine={'dbsql_version':'2026.39','u_build_hash':'15b447529a1f55ca1ad8c73a89b8d196f6ed4904','r_build_hash':'3909d148af5cb6b560624c9255f5a727f7a17a4c','dbr_version':None}
assert json.loads(client.sql('engine-before',"SELECT to_json(current_version(), map('ignoreNullFields','false'))")[0][0])==engine
ansi=client.sql('ansi-mode','SET ansi_mode');assert ansi==[['ansi_mode','true']],ansi
for a in artifacts:
 r=a['response'];checks=[c for o in r['obligations'] if o['id'] in ['truss.candidate.scalarIntegrity','ashlar.candidate.scalarIntegrity'] for c in o['parameters']['checks']];assert len(checks)==(1 if a['backend']=='truss' else 2),(a['id'],len(checks))
 if a['backend']=='truss':
  params=r['parameters'];types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in params);values=','.join(namespace['quote'](p['value']) for p in params)
  seed={'Customer':[{'id':str(i),'name':name,'active':str(active).lower()} for i,(active,name) in enumerate(SEED,1)]}
  sql='BEGIN ISOLATION LEVEL REPEATABLE READ;\nSET standard_conforming_strings=on;\n'+namespace['setup'](a['mapping'],seed,[],a['request']['modules'])
  sql+="SELECT json_build_object('version',current_setting('server_version'),'server_encoding',current_setting('server_encoding'),'client_encoding',current_setting('client_encoding'),'standard_conforming_strings',current_setting('standard_conforming_strings'),'transaction_isolation',current_setting('transaction_isolation'),'lc_collate',(SELECT datcollate FROM pg_database WHERE datname=current_database()),'lc_ctype',(SELECT datctype FROM pg_database WHERE datname=current_database()))::text AS profile;\n"
  for i,c in enumerate(checks):sql+=f"PREPARE check_{i}({types}) AS {c['sql']};\nEXECUTE check_{i}({values});\n"
  sql+=f"PREPARE query({types}) AS {r['sql']};\nEXECUTE query({values});\nROLLBACK;\n"
  raw=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(raw)));profile=json.loads(rows[1][0]);
  (OUT/(a['id']+'.raw.json')).write_text(json.dumps(dict(sql=sql,stdout=raw,profile=profile)))
  assert profile=={'version':'17.9 (Debian 17.9-1.pgdg13+1)','server_encoding':'UTF8','client_encoding':'UTF8','standard_conforming_strings':'on','transaction_isolation':'repeatable read','lc_collate':'C','lc_ctype':'C'}
  end=2+2*len(checks);assert rows[2:end]==[['count'],['0']]*len(checks),(a['id'],rows)
  actual=rows[end+1:];assert rows[end]==[c['outputName'] for c in r['columns']]
  pg_records.append(dict(id=a['id'],sql=sql,stdout=raw,profile=profile))
 else:
  binding=json.loads(a['request']['target']['bindingJson']);record=binding['records'][0];table=binding['publication']['tables'][0]
  physical='.'.join('`'+x.replace('`','``')+'`' for x in table['name'])+' VERSION AS OF '+str(table['version'])
  owner='('+' UNION ALL '.join("SELECT "+str(i)+" AS id, '"+record['sourceSystem']+"' AS source_system, CAST("+record['typeId']+" AS BIGINT) AS type_id, '"+json.dumps({'24':name,'25':active,'23':i})+"' AS props_json, '"+name+"' AS group_value, true AS group_present, "+str(active).lower()+" AS active_value, true AS active_present" for i,(active,name) in enumerate(SEED,1))+')'
  def substitute(sql):
   assert physical in sql;return sql.replace(physical,owner)
  params=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in r['parameters']]
  for i,c in enumerate(checks):assert client.sql(a['id']+'-guard-'+str(i),substitute(c['sql']),params)==[['0']]
  query="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS __weft_engine, observed.* FROM ("+substitute(r['sql'])+") observed"
  observed=client.sql(a['id']+'-query',query,params);assert observed and all(json.loads(row[0])==engine for row in observed)
  actual=[row[1:] for row in observed];profile=engine
 assert actual==a['expectedRows'],(a['id'],actual,a['expectedRows'])
 results.append(dict(id=a['id'],actual=actual,profile=profile));print(a['id']+' passed',flush=True)
assert json.loads(client.sql('engine-after',"SELECT to_json(current_version(), map('ignoreNullFields','false'))")[0][0])==engine
(OUT/'postgresql-receipts.json').write_text(json.dumps(pg_records,indent=2)+'\n')
summary=dict(status='passed',cases=32,postgresqlCases=16,ashlarCases=16,ashlarStatements=len(client.records),results=results,compilerSha256=sha(BINARY),harnessSha256=sha(Path(__file__)),scope='Native required AND capability in both dialects, props and typed scalar homes. Four Boolean/string conjunction cells with duplicate true/hit rows; actual query and integrity SQL, PostgreSQL same transaction settings and same-statement Databricks warehouse captures. Separate ANSI probes do not attest arbitrary sessions. No stored publication custody or supported promotion.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
