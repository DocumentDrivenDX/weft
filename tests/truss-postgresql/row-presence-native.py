"""Native row presence matrix with independent structural/payload expectations."""
import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();emission=json.loads(raw)
quote=lambda s:"'"+str(s).replace("'","''")+"'"
slots=emission['parameters'];assert [p['value'] for p in slots]==[emission['ownerTypeId'],emission['ownerTypeId'],emission['propertyId']]
values=','.join(quote(p['value']) for p in slots);types=','.join('int4' for p in slots)
results=[]
for profile in emission['cases']:
 for name in ['absent','null','empty','unicode','null-with-payload','missing-root','wrong-kind','missing-source','wrong-codec']:
  valid=(name=='absent' and not profile['required']) or (name=='null' and profile['nullable']) or name in ('empty','unicode')
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
  oid=emission['ownerTypeId'];pid=emission['propertyId']
  sql+=f'INSERT INTO object VALUES (987654321,{oid}),(123,-2);\n'
  if name!='absent':
   sql+=f"INSERT INTO row_home_state VALUES (10,'object',987654321,{oid},NULL,NULL,{oid},{pid},20);\n"
   if name!='missing-root':
    kind='null' if name in ('null','null-with-payload','missing-source') else 'sequence' if name=='wrong-kind' else 'scalar'
    source='NULL' if name=='missing-source' else "pg_catalog.decode('fe00','hex')"
    sql+=f"INSERT INTO row_home_node VALUES (10,20,NULL,'{kind}',pg_catalog.decode('00','hex'),{source});\n"
   if name in ('empty','unicode','null-with-payload','wrong-kind','wrong-codec'):
    text='' if name=='empty' else 'é  '
    codec='00' if name=='wrong-codec' else emission['codecHex']
    sql+=f"INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,codec_definition_bytes,original_source_bytes) VALUES (10,20,'string',{quote(text)},pg_catalog.decode('{codec}','hex'),pg_catalog.decode('fe00','hex'));\n"
  checks=profile['structuralChecks']+[profile['check']]
  for index,check in enumerate(checks):sql+=f'PREPARE check_{index}({types}) AS {check}; EXECUTE check_{index}({values});\n'
  if valid:sql+=f"PREPARE query({types}) AS {profile['sql']}; EXECUTE query({values});\n"
  sql+='ROLLBACK;\n'
  output=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(output)));counts=[]
  for index in range(len(checks)):
   assert rows[index*2]==['violations'];counts.append(int(rows[index*2+1][0]))
  assert (not any(counts))==valid,(profile,name,counts)
  rest=rows[len(checks)*2:]
  if valid:
   expected={'state':'absent'} if name=='absent' else {'state':'null'} if name=='null' else {'state':'value','value':'' if name=='empty' else 'é  '}
   assert rest[0]==['value'] and json.loads(rest[1][0])==expected
  else:assert not rest
  results.append({'required':profile['required'],'nullable':profile['nullable'],'case':name,'violations':counts,'queryExecuted':valid,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt.write_text(json.dumps({'scope':'Native row scalar presence SQL matrix using captured original home/location with explicit descriptor flags; synthetic object state/node/scalar storage; not full optional/nullable original-model compilation, recursive values or installed host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('36 native row presence matrix cases passed.')
