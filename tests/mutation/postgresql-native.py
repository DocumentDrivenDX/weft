# SPDX-License-Identifier: MIT OR Apache-2.0
"""Independent PostgreSQL17.11 witness for generated selection/preflight only.
No accepted catalog, original authority, seal, immutable-ID premise or bulk effects.
"""
import copy,csv,hashlib,importlib.metadata,importlib.resources,io,json,pathlib,subprocess,sys,tempfile
import pgserver
from urllib.parse import urlsplit,urlunsplit
root=pathlib.Path(__file__).resolve().parents[2]
capture=pathlib.Path(sys.argv[1]);programs={c['id']:c['lowered'] for c in json.loads(capture.read_text())}
assert importlib.metadata.version('pgserver')=='0.1.4+truss.pg17.11'
psql=pathlib.Path(str(importlib.resources.files('pgserver')))/'pginstall/bin/psql'
quote=lambda text:"'"+text.replace("'","''")+"'"
def execute(uri,sql):return subprocess.check_output([str(psql),uri,'-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql,text=True,timeout=60)
def params(p):
 values=[]
 for slot in p['parameters']:
  literal=slot['value'];value=literal.get('string',literal.get('integerToken'))
  if 'boolean' in literal:value='true' if literal['boolean'] else 'false'
  assert type(value)is str;values.append(quote(value))
 return values
base=[{'id':'2','ver':'7','props':{'0':'2','20':'a'}},{'id':'10','ver':'8','props':{'0':'10','20':'Foo😀'}},{'id':'11','ver':'9','props':{'0':'99','20':'z'}},{'id':'12','ver':'10','props':{'0':'101','20':'excluded'}}]
results=[]
with tempfile.TemporaryDirectory(prefix='weft-mutation-pg17-') as directory:
 server=pgserver.get_server(pathlib.Path(directory)/'data',cleanup_mode='stop');uri=server.get_uri()
 try:
  version=execute(uri,'SHOW server_version;').strip().splitlines()[-1];assert version=='17.11';assert execute(uri,"SELECT current_setting('listen_addresses')='' AS unix_only;").strip().splitlines()[-1]=='t'
  native_controls = execute(uri, "SELECT pg_catalog.pg_input_is_valid('0e999999999','pg_catalog.numeric') AS huge_zero, pg_catalog.pg_input_is_valid("+quote('1.'+'0'*17000)+",'pg_catalog.numeric') AS long_fraction;").strip().splitlines()[-1];assert native_controls=='t,f',native_controls
  cases=[('ordered-overflow','integer',base,['2','10','11'],True),('mixed-precedence','mixed',base,['2','10','11'],True),('unicode-scalar-order','string',base,[],False),('native-large-identity','integer',[{'id':'9007199254740993','ver':'9007199254740995','props':{'0':'2','20':'large'}}],['9007199254740993'],False)]
  for bad in ['missing','null','boolean','fraction','malformed','nonfinite','unsigned-overflow','root-array','key-missing','key-null','key-wrong-kind','version-zero','identity-negative']:
   rows=copy.deepcopy(base);row=rows[-1];row['props']['0']='101'
   if bad=='missing':row['props'].pop('0')
   elif bad=='null':row['props']['0']=None
   elif bad=='boolean':row['props']['0']=False
   elif bad=='fraction':row['props']['0']='101.1'
   elif bad=='malformed':row['props']['0']='not-a-number'
   elif bad=='nonfinite':row['props']['0']='NaN'
   elif bad=='unsigned-overflow':row['props']['0']='18446744073709551616'
   elif bad=='root-array':row['props']=[]
   elif bad=='key-missing':row['props'].pop('20')
   elif bad=='key-null':row['props']['20']=None
   elif bad=='key-wrong-kind':row['props']['20']=False
   elif bad=='version-zero':row['ver']='0'
   elif bad=='identity-negative':row['id']='-12'
   cases.append(('masked-'+bad,'integer',rows,None,False))
   cases.append(('masked-or-'+bad,'mixed',copy.deepcopy(rows),None,False))
  for ordinal,(name,program,rows,expected,overflow) in enumerate(cases):
   database=f'weft_mutation_{ordinal}';role=f'weft_mutation_role_{ordinal}';execute(uri,f'CREATE ROLE {role} LOGIN;CREATE DATABASE {database} OWNER {role};');parts=urlsplit(uri);test_uri=urlunsplit((parts.scheme,role+'@'+parts.netloc.split('@')[-1],'/'+database,parts.query,parts.fragment))
   p=programs[program];sql='BEGIN ISOLATION LEVEL REPEATABLE READ;CREATE TEMP TABLE object(id bigint,type_id int,ver bigint,props jsonb);'
   for row in rows:sql+=f"INSERT INTO object VALUES ({row['id']},-1,{row['ver']},{quote(json.dumps(row['props'],ensure_ascii=False))}::jsonb);"
   # A malformed unselected type must not pollute the admitted owner view.
   sql+="INSERT INTO object VALUES (1000,999,1,'[]'::jsonb);"
   for i,step in enumerate(p['preflight']):assert not step['parameters'];sql+=step['sql']+';'
   if expected is not None:
    values=params(p);declaration='('+','.join(['text']*len(values))+')' if values else ''
    sql+='PREPARE selected'+declaration+' AS '+p['sql']+';EXECUTE selected'+('('+','.join(values)+')' if values else '')+';'
   sql+='ROLLBACK;';out=execute(test_uri,sql);execute(uri,f'DROP DATABASE {database};DROP ROLE {role};');csv_rows=list(csv.reader(io.StringIO(out)));checks=len(p['preflight']);counts=[int(csv_rows[i*2+1][0]) for i in range(checks)]
   assert all(csv_rows[i*2]==['violations'] for i in range(checks))
   if expected is None:assert any(counts),(name,out);assert len(csv_rows)==checks*2
   else:
    assert not any(counts),(name,out);selected=csv_rows[checks*2:];assert selected[0]==['identity','version','key0','key1'];actual=[r[0] for r in selected[1:]];assert actual==expected,(name,actual,expected)
    assert (len(actual)>p['bounds']['maxItems'])==overflow
    if name=='native-large-identity':assert selected[1]==['9007199254740993','9007199254740995','large','2']
   results.append(dict(id=name,preflightViolations=counts,queryExecuted=expected is not None,identities=expected,overflow=overflow,sqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
 finally:server.cleanup()
report=dict(scope='Original mapping/codecs plus actual Truss object-column declaration; native selection/preflight on synthetic rows, no protected authority/guard/seal/execution',nativeRepresentationControls={'hugeZeroExponentAccepted':True,'longFractionAccepted':False},serverVersion=version,pgserver=importlib.metadata.version('pgserver'),cases=len(results),passed=True,captureSha256=hashlib.sha256(capture.read_bytes()).hexdigest(),harnessSha256=hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),results=results)
output=root/'target/mutation/postgresql-native-results.json';output.parent.mkdir(parents=True,exist_ok=True);output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({k:v for k,v in report.items() if k!='results'}))
