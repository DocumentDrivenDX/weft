from pathlib import Path
import collections,hashlib,json,zipfile,sys,subprocess,os,secrets,time,uuid
base=Path('/private/tmp/umf-current-scalar-edge-20261015');run_id=str(uuid.uuid4());root=base/'attempts'/run_id;root.mkdir(parents=True,exist_ok=False);(base/'current-attempt.json').write_text(json.dumps({'runId':run_id,'directory':str(root),'status':'pending'})+'\n');frozen=Path('/private/tmp/weft-current-scalar-frozen-20261015');installed=Path('/private/tmp/weft-current-scalar-installed');wheel=Path('/private/tmp/weft-current-scalar-wheels/weft_sql-0.4.32-cp39-abi3-macosx_11_0_arm64.whl')
sha=lambda b:hashlib.sha256(b).hexdigest()
manifest_raw=(frozen/'source-manifest.json').read_bytes();assert sha(manifest_raw)=='6a901c4bdb5c9a5b1e6c2b2dd40c9a19df374584250e65c8d1290263722670d8';manifest=json.loads(manifest_raw)
for p,h in manifest['files'].items():assert sha((frozen/p).read_bytes())==h,p
assert sha(wheel.read_bytes())=='5c3ec17bad0fb8ea12c66d0b027fef6ef4be1ca0b05343ed00df96b26f90bbcf'
modules=list(installed.rglob('*.so'));assert len(modules)==1;module_sha=sha(modules[0].read_bytes());assert module_sha=='991f8e5d8c203e8c084f14aa4e5ce0e4b1d31de814b480b17ca4120658b5e95a'
with zipfile.ZipFile(wheel) as z:
 names=[n for n in z.namelist() if n.endswith('.so')];assert len(names)==1 and sha(z.read(names[0]))==module_sha
shim=installed/'weft/__init__.py';shim_sha=sha(shim.read_bytes())
with zipfile.ZipFile(wheel) as z:assert sha(z.read('weft/__init__.py'))==shim_sha
sys.path.insert(0,str(installed));import weft,psycopg,importlib
native=importlib.import_module('weft.weft');assert Path(native.__spec__.origin).resolve()==modules[0].resolve();assert sha(Path(native.__spec__.origin).read_bytes())==module_sha;assert weft.prepare_application_lineage is native.prepare_application_lineage
assert Path(weft.__file__).resolve().is_relative_to(installed)
source_path=frozen/'tests/current-query-designation/fixtures/mixed-original-uses.request.json';source_raw=source_path.read_bytes();source=json.loads(source_raw);oracle_raw=(base/'oracle.json').read_bytes();oracle=json.loads(oracle_raw);checks=[];refusals=[];harness_sources={str(Path(__file__).resolve()):Path(__file__).read_bytes(),'/private/tmp/prepare-current-scalar-edge-oracle.py':Path('/private/tmp/prepare-current-scalar-edge-oracle.py').read_bytes(),str(base/'oracle.json'):oracle_raw};(root/'oracle.json').write_bytes(oracle_raw);(root/'run.py').write_bytes(harness_sources[str(Path(__file__).resolve())])
identity=lambda e:{'documentId':'domain','revision':'query-value-core-r1','module':'m','element':e}
class NativeRefusal(Exception):
 def __init__(self,stage,code):self.stage=stage;self.code=code
def invoke(stage,call):
 try:return call()
 except ValueError as e:raise NativeRefusal(stage,str(e).split(':')[0]) from e
def compile_case(case):
 request=json.loads(source_raw);request['sql']=case['sql'];request['parameters']=case['parameters'];raw=json.dumps(request,separators=(',',':'));parent=invoke('prepare-application-lineage',lambda:weft.prepare_application_lineage(raw));child=invoke('prepare-current-query',parent.prepare_current_query);assert child.source_preparation() is parent
 lineage=json.loads(parent.json());plan=json.loads(lineage['applicationPlanJson']);designation=json.loads(child.designation_json());uses={(v['path'],v['scan']):v for v in designation['uses']};values=[]
 for use in lineage['uses']:
  action=uses[(use['path'],use['scan'])]['originalAction'];values.append({'path':use['path'],'scan':use['scan'],'field':use['fieldIdentity'],'column':use['fieldIdentity']['element'],'domain':use['fieldIdentity'],'comparisonScope':'a'*64,'mode':'original-authorized' if action else 'disclosed','originalAction':action,'exposure':None})
 for field in lineage['readRequirements']['fields']:
  if field['role']!='projection':continue
  salary=field['fieldIdentity']['element']=='salary';values.append({'path':field['path'],'scan':field['scan'],'field':field['fieldIdentity'],'column':'salary_public' if salary else field['fieldIdentity']['element'],'domain':identity('salaryMaskLabel') if salary else field['fieldIdentity'],'comparisonScope':'a'*64,'mode':None,'originalAction':None,'exposure':'transformed' if salary else 'original'})
 binding={'version':'weft.postgresql.current-values/0.1-candidate','sourceRequestSha256':sha(parent.source_request_json().encode()),'designationSha256':sha(child.designation_json().encode()),'sourceBindingSha256':designation['binding']['sha256'],'scans':[{'scan':s['occurrence'],'target':s['record'],'relation':{'schema':'private_stage','table':'current_values','eligibleColumn':'eligible'}} for s in [plan['source']]+[j['right'] for j in plan['joins']]],'values':values}
 compiled_json=invoke('lower-postgresql-scalar',lambda:child.lower_postgresql_scalar_candidate_json(json.dumps(binding,separators=(',',':'))));compiled=json.loads(compiled_json);assert compiled['authority'] is False and compiled['securityReady'] is False and compiled['nativeAdmission']=='required';assert parent.source_request_json()==raw
 return {'request':request,'binding':binding,'compiled':compiled,'sameOriginalParent':True}
# Compile all original-object cases before native fixture creation. No imported report becomes a context.
artifacts=[]
for case in oracle['cases']:
 artifact=compile_case(case);artifacts.append((case,artifact));(root/(case['id']+'.compiled.json')).write_text(json.dumps(artifact,ensure_ascii=True,indent=2)+'\n')
for case in oracle['refusals']:
 try:compile_case(case)
 except NativeRefusal as e:
  expected='WFT-PARAMETER' if case['id']=='text-nul-even-empty' else 'WFT-NUMERIC-DOMAIN';assert e.code==expected and e.stage=='prepare-application-lineage',(case['id'],e.stage,e.code);refusals.append({'id':case['id'],'refused':True,'expectedDiagnosticCode':expected,'diagnosticCode':e.code,'originalBoundary':e.stage,'sqlNotDispatched':True})
 else:raise AssertionError('Expected whole-compile refusal: '+case['id'])
(root/'compile-preflight.json').write_text(json.dumps({'sourceManifestFiles':len(manifest['files']),'positiveCases':len(artifacts),'refusals':refusals},indent=2)+'\n')
if '--compile-only' in sys.argv:print(json.dumps({'compilePreflight':'passed','cases':len(artifacts),'refusals':len(refusals)}));sys.exit(0)
name='umf-current-edge-'+run_id;
# Exact pinned image selected below, not a mutable version tag.
image='postgres@sha256:74935e72241653ca55e0414067e6d8763aceb8a810eb51b452253ec3dcfc4336'
admin_password=secrets.token_hex(32);reader_password=secrets.token_hex(32);container=None;attempted=False;receipt=None;initial_mounts=[]

def command(args,**kw):return subprocess.run(args,capture_output=True,text=True,timeout=kw.pop('timeout',45),**kw)
def require(result):
 if result.returncode:raise RuntimeError('Owned Docker operation failed: '+result.stderr[:400])
 return result.stdout.strip()
try:
 assert name not in require(command(['docker','container','ls','-a','--format','{{.Names}}'])).splitlines();attempted=True
 container=require(command(['docker','run','-d','--name',name,'--label','umf.security.run='+run_id,'--publish','127.0.0.1::5432','-e','POSTGRES_HOST_AUTH_METHOD=scram-sha-256','-e','POSTGRES_INITDB_ARGS=--auth-host=scram-sha-256 --auth-local=trust','-e','POSTGRES_PASSWORD',image],env={**os.environ,'POSTGRES_PASSWORD':admin_password}))
 initial_mounts=json.loads(require(command(['docker','inspect','--format','{{json .Mounts}}',container])));assert all(m['Type']=='volume' and m.get('Name') for m in initial_mounts);(root/'initial-mounts.json').write_text(json.dumps({'container':container,'runId':run_id,'mounts':initial_mounts},indent=2)+'\n')
 port=int(json.loads(require(command(['docker','inspect','--format','{{json .NetworkSettings.Ports}}',container])))['5432/tcp'][0]['HostPort']);deadline=time.monotonic()+30;admin=None
 while admin is None:
  try:admin=psycopg.connect(host='127.0.0.1',port=port,user='postgres',password=admin_password,dbname='postgres',connect_timeout=2,autocommit=True,cursor_factory=psycopg.RawCursor)
  except psycopg.OperationalError:
   if time.monotonic()>deadline:raise TimeoutError('Owned PostgreSQL readiness')
   time.sleep(.1)
 with admin:
  engine=admin.execute("SELECT current_setting('server_version_num'),version(),current_setting('server_encoding')").fetchone();assert engine[0]=='180006' and engine[2]=='UTF8',engine
  admin.execute('CREATE SCHEMA private_stage');admin.execute('CREATE TABLE private_stage.current_values(eligible boolean NOT NULL,"resourceId" text COLLATE "C" NOT NULL,salary bigint NOT NULL,salary_public text COLLATE "C" NOT NULL,token text COLLATE "C" NOT NULL)')
  with admin.cursor() as c:c.executemany('INSERT INTO private_stage.current_values VALUES($1,$2,$3::bigint,$4,$5)',[(r['eligible'],r['id'],r['salaryToken'],r['salaryPublic'],r['token']) for r in oracle['rows']])
  admin.execute('CREATE ROLE umf_edge_reader LOGIN NOSUPERUSER NOBYPASSRLS PASSWORD '+"'"+reader_password+"'");admin.execute('GRANT USAGE ON SCHEMA private_stage TO umf_edge_reader');admin.execute('GRANT SELECT ON private_stage.current_values TO umf_edge_reader')
  inventory=admin.execute("SELECT a.attname,a.atttypid::text,coalesce(c.collname,'') FROM pg_attribute a LEFT JOIN pg_collation c ON c.oid=a.attcollation WHERE a.attrelid='private_stage.current_values'::regclass AND a.attnum>0 ORDER BY a.attnum").fetchall()
 with psycopg.connect(host='127.0.0.1',port=port,user='umf_edge_reader',password=reader_password,dbname='postgres',connect_timeout=5,autocommit=True,cursor_factory=psycopg.RawCursor) as connection:
  actor=connection.execute("SELECT session_user,current_user,current_setting('is_superuser'),rolbypassrls FROM pg_roles WHERE rolname=session_user").fetchone();assert actor==('umf_edge_reader','umf_edge_reader','off',False)
  connection.execute('SET default_transaction_read_only=on');connection.execute('SET statement_timeout=5000')
  for case,artifact in artifacts:
   compiled=artifact['compiled'];cursor=connection.execute(compiled['sql'],[p['value'] for p in compiled['parameters']] or None);observed=[list(row) for row in cursor.fetchall()];oids=[c.type_code for c in cursor.description];assert all(o==25 for o in oids)
   expected=case['expectedRows'];actual=sorted(observed) if case['unordered'] else observed;wanted=sorted(expected) if case['unordered'] else expected;assert actual==wanted,(case['id'],actual,wanted)
   for column in compiled['resultColumns']:
    if column['exposure']=='computed':assert column['logicalType']['family']=='integer' and column['logicalType']['facets']=={} and column['encoding']=='native-int8-nonnegative-text'
   checks.append({'id':case['id'],'expectedRows':expected,'observedRows':observed,'unordered':case['unordered'],'nativeTypeOids':oids,'passed':True})
  count=next(a['compiled'] for c,a in artifacts if c['id']=='eligible-bag-count');weakened=count['sql'].replace('WHERE "eligible" IS TRUE','WHERE TRUE');assert weakened!=count['sql'];leaked=connection.execute(weakened).fetchall();assert leaked==[(str(len(oracle['rows'])),)] and leaked!=[(str(sum(r['eligible'] for r in oracle['rows'])),)]
  negative={'id':'removed-eligibility-filter','actualWeakenedSql':weakened,'observedRows':leaked,'expectedOriginalRows':[[str(sum(r['eligible'] for r in oracle['rows']))]],'oracleMismatchObserved':True,'scope':'SQL mutation only; authored eligibility is a premise, not proved authorization'}
 for path,h in manifest['files'].items():assert sha((frozen/path).read_bytes())==h,path
 assert sha(modules[0].read_bytes())==module_sha and sha(wheel.read_bytes())==manifest['wheelSha256'];assert sha((root/'oracle.json').read_bytes())==sha(oracle_raw)
 receipt={'status':'conditional-native-edge-checks-passed','runId':run_id,'scope':'Actual same-original typed CQP compiler to PG18 over synthetic declared stage values; no UMF/native eligibility, action permission, runtime-scope proof, source issuer, guarded release or backend acceptance','python':sys.version,'psycopg':psycopg.__version__,'weft':weft.__version__,'wheelSha256':sha(wheel.read_bytes()),'moduleSha256':module_sha,'loadedNativeOrigin':str(Path(native.__spec__.origin).resolve()),'importShimSha256':shim_sha,'shimMatchesWheel':True,'sourceManifestSha256':sha(manifest_raw),'sourceFilesVerifiedBeforeAfter':len(manifest['files']),'oracleSha256':sha(oracle_raw),'engine':engine,'ordinaryFixtureActor':actor,'nativeStageInventory':inventory,'checks':checks,'refusals':refusals,'negativeControl':negative,'backendAcceptancePromoted':False}
finally:
 if container is None and attempted:
  matches=[line.split()[0] for line in require(command(['docker','container','ls','-a','--no-trunc','--format','{{.ID}} {{.Names}}'])).splitlines() if len(line.split())==2 and line.split()[1]==name];assert len(matches)<=1
  if matches:container=matches[0]
 if container:
  assert require(command(['docker','inspect','--format','{{index .Config.Labels \"umf.security.run\"}}',container]))==run_id
  if not initial_mounts:initial_mounts=json.loads(require(command(['docker','inspect','--format','{{json .Mounts}}',container])))
  volumes=[m['Name'] for m in initial_mounts if m['Type']=='volume'];assert len(volumes)==len(initial_mounts)
  for volume in volumes:
   users=require(command(['docker','container','ls','-a','--no-trunc','--filter','volume='+volume,'--format','{{.ID}}'])).splitlines();assert set(users)<=set([container]),'Owned volume acquired another container user'
  removed=command(['docker','rm','-f','-v',container]);require(removed)
  remaining_containers=require(command(['docker','container','ls','-a','--no-trunc','--format','{{.ID}}'])).splitlines();remaining_volumes=require(command(['docker','volume','ls','--format','{{.Name}}'])).splitlines();assert container not in remaining_containers and not any(v in remaining_volumes for v in volumes)
  if receipt is not None:receipt['cleanup']={'ownedContainer':container,'runId':run_id,'initialMounts':initial_mounts,'sharedVolumeUsersBeforeRemoval':[],'removalExitCode':removed.returncode,'containerAbsent':True,'ownedVolumesAbsent':True}
for path,body in harness_sources.items():assert Path(path).read_bytes()==body,path
assert sha(shim.read_bytes())==shim_sha and sha(Path(native.__spec__.origin).read_bytes())==module_sha
if receipt is not None:receipt['harnessSourcesUnchangedThroughCleanup']={path:sha(body) for path,body in harness_sources.items()}
if receipt is None:raise RuntimeError('No passing receipt')
(root/'receipt.json').write_text(json.dumps(receipt,ensure_ascii=True,indent=2)+'\n');(base/'current-attempt.json').write_text(json.dumps({'runId':run_id,'directory':str(root),'status':'passed'})+'\n');print(json.dumps({'status':receipt['status'],'positiveCases':len(checks),'refusals':len(refusals),'ownedContainerRemoved':True}))
