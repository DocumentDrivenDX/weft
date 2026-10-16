"""Fresh native SQL/checker correspondence with synthetic stage premises.
No caller/source authorization, native admission or guarded release claim.
"""
import argparse,ast,collections,copy,hashlib,importlib,json,pathlib,shutil,sys,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'))
from truss import LocalPostgres
import psycopg
root=pathlib.Path(__file__).resolve().parents[1];checks=[];source_pins={}
def text(v):return json.dumps(v,separators=(',',':'),ensure_ascii=False)
def sha(v):return hashlib.sha256(v).hexdigest()
native=pathlib.Path(importlib.import_module('weft.weft').__file__)
with zipfile.ZipFile(a.wheel)as z:
 names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1
 assert native.read_bytes()==z.read(names[0])and pathlib.Path(weft.__file__).read_bytes()==z.read('weft/__init__.py')
def profile(name):
 path=pathlib.Path(__file__).with_name(name+'.py');source_pins[name]=sha(path.read_bytes());tree=ast.parse(path.read_text())
 env={'json':json,'str':str};needed={'maximum','minimum','rows','original','one','joined','all_keys','cases'}
 for n in tree.body:
  if isinstance(n,ast.Assign)and all(isinstance(t,ast.Name)and t.id in needed for t in n.targets):exec(compile(ast.Module(body=[n],type_ignores=[]),str(path),'exec'),env)
  if isinstance(n,ast.FunctionDef)and n.name in {'identity','binding'}:exec(compile(ast.Module(body=[n],type_ignores=[]),str(path),'exec'),env)
 return env
runtime=LocalPostgres(a.output/'postgres')
with runtime:
 with psycopg.connect(runtime.info.connection_uri,autocommit=True,cursor_factory=psycopg.RawCursor)as connection:
  version=connection.execute('SHOW server_version_num').fetchone()[0];assert version=='180006'
  connection.execute('CREATE SCHEMA private_stage')
  connection.execute('CREATE TABLE private_stage.current_values (eligible boolean NOT NULL,private_key text COLLATE "C" NOT NULL UNIQUE,"resourceId" text COLLATE "C" NOT NULL,salary bigint NOT NULL,salary_public text COLLATE "C" NOT NULL,token text COLLATE "C" NOT NULL)')
  for name in ['rows','contributors','sum']:
   env=profile(name);fixture=env['rows'];source=json.loads((root/('current-query-designation/fixtures/live-original-integer-sum.request.json'if name=='sum'else'current-query-designation/fixtures/mixed-original-uses.request.json')).read_text())
   connection.execute('TRUNCATE private_stage.current_values')
   for r in fixture:
    row=r if name!='sum'else(r[0],r[1],'unused',r[2],'unused',r[3]);connection.execute('INSERT INTO private_stage.current_values VALUES ($1,$2,$3,$4,$5,$6)',row)
   for label,sql,parameters,expected,unordered in env['cases']:
    parent=weft.prepare_application_lineage(text(dict(source,sql=sql,parameters=parameters)));child=parent.prepare_current_query();assert child.source_preparation()is parent
    packet,b=env['binding'](child);binding=text(b);lower=getattr(child,'lower_postgresql_current_'+name+'_candidate_json');check=getattr(child,'check_postgresql_current_'+name+'_candidate_json')
    artifact=lower(binding);compiled=json.loads(artifact);width=len(compiled['resultColumns']);scans=[s['scan']for s in packet['scans']]
    cursor=connection.execute(compiled['sql'],[p['value']for p in compiled['parameters']]or None);assert all(c.type_code==25 for c in cursor.description);wire=[list(r)for r in cursor.fetchall()]
    # Original predeclared tuple/bag oracle, paired with complete public values.
    def canonical(row):
     carrier=json.loads(row[-1]);tuples=[carrier['rows']]if name=='rows'else carrier['tuples']
     assert all([c['scan']for c in t]==scans for t in tuples)
     return(tuple(row[:width]),tuple(sorted(tuple(c['key']for c in t)for t in tuples)))
    actual=[canonical(r)for r in wire];wanted=[(tuple(public),tuple(sorted([keys]if name=='rows'else keys)))for public,keys in expected]
    assert (collections.Counter(actual)==collections.Counter(wanted))if unordered else(actual==wanted),(name,label)
    populations=[]
    for scan in scans:
     columns={v['column']for v in b['values']if v['scan']==scan};rows=[]
     for r in fixture:
      if not r[0]:continue
      original=env['original'][r[1]]
      rows.append(dict(key=r[1],cells={c:dict(family='integer'if c=='salary'else'string',value=str(original[c]))for c in columns}))
     populations.append(dict(scan=scan,expectedRowCount=str(len(rows)),rows=rows))
    frame=dict(version='weft.current-query.refinement-input/0.1-candidate',sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],materializationBindingSha256=sha(binding.encode()),compiledArtifactSha256=sha(artifact.encode()),scans=populations,resultRows=wire)
    raw=text(frame);report=json.loads(check(binding,raw));assert report['matched']is True and report['authority']is False and report['securityReady']is False and report['nativeAdmission']=='required'and report['refinementInputSha256']==sha(raw.encode())
    checks.append({'id':name+'/'+label,'originalOracleAndCheckerMatch':True})
    if wire:
     bad=copy.deepcopy(frame);bad['resultRows'][0][0]='wrong'
     try:check(binding,text(bad))
     except ValueError as e:assert 'WFT-CURRENT-QUERY-REFINEMENT'in str(e)
     else:raise AssertionError('Wrong native public value accepted')
     checks.append({'id':name+'/'+label+'/wrong-public','refused':True})
assert runtime._server is None and runtime._lease is None and not (runtime.directory/'postmaster.pid').exists()
shutil.rmtree(runtime.directory)
assert not runtime.directory.exists()
report={'serverStopped':True,'leaseReleased':True,'runtimeDirectoryRemoved':True,'scope':__doc__,'weftVersion':weft.__version__,'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native.read_bytes()),'runnerSha256':sha(pathlib.Path(__file__).read_bytes()),'originalOracleSourcePins':source_pins,'serverVersionNum':version,'checks':checks,'nativeSecurityAcceptance':False,'casesPromoted':[]}
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'checks':len(checks),'nativeSecurityAcceptance':False}))
