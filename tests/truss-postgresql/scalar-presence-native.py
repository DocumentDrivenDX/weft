"""Independent nullable/required presence matrix for captured scalar envelopes."""
import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();emission=json.loads(raw)
quote=lambda value:"'"+str(value).replace("'","''")+"'"
slots=emission['parameters'];assert [p['origin']['use'] for p in slots]==['admitted-owner-scan','admitted-jsonb-member']
values=','.join(quote(p['value']) for p in slots)
results=[]
for profile in emission['cases']:
 for name,root in [('absent','{}'),('null','{"1":null}'),('empty','{"1":""}'),('unicode','{"1":"é  "}'),('wrong-scalar','{"1":false}'),('bad-root','[]'),('sql-null',None)]:
  valid=(name=='absent' and not profile['required']) or (name=='null' and profile['nullable']) or name in ('empty','unicode')
  expected={'state':'absent'} if name=='absent' else {'state':'null'} if name=='null' else {'state':'value','value':'' if name=='empty' else 'é  '}
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
  sql+=f"INSERT INTO object VALUES (987654321,-1,{'NULL' if root is None else quote(root)}),(123,-2,'[]');\n"
  sql+=f"PREPARE preflight(int4,text) AS {profile['check']}; EXECUTE preflight({values});\n"
  if valid:sql+=f"PREPARE query(int4,text) AS {profile['sql']}; EXECUTE query({values});\n"
  sql+='ROLLBACK;\n'
  output=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(output)));assert rows[:2]==[['violations'],['0' if valid else '1']],(profile,name,rows)
  if valid:assert rows[2]==['value'] and json.loads(rows[3][0])==expected
  else:assert len(rows)==2
  results.append({'required':profile['required'],'nullable':profile['nullable'],'case':name,'queryExecuted':valid,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt.write_text(json.dumps({'scope':'Scalar JSONB presence-envelope SQL across independent required/nullability matrix on synthetic string storage; original admitted location/carrier with explicit descriptor flags; not full optional/nullable original-model compilation, native-row presence, recursive values or host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('28 native scalar presence matrix cases passed.')
