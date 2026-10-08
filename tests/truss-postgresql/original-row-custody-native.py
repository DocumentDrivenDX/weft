"""Captured original-property/native-home custody query on synthetic storage."""
import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
capture,receipt=map(Path,sys.argv[1:3]);raw=capture.read_bytes();emission=json.loads(raw)
quote=lambda s:"'"+str(s).replace("'","''")+"'"
slots=emission['parameters'];assert [p['origin'] for p in slots]==[{'typeId':emission['ownerTypeId'],'use':'admitted-owner-scan'},{'typeId':emission['ownerTypeId']},{'propertyId':emission['propertyId']}]
values=','.join(quote(p['value']) for p in slots);types=','.join('int4' for p in slots)
headers=['scalar_present','scalar_kind','text_value','boolean_value','native_numeric_text','original_numeric_token','codec_bytes_hex','source_bytes_hex']
results=[]
for name,present in [('no-scalar',False),('string-custody',True)]:
 sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea);
'''
 oid=emission['ownerTypeId'];pid=emission['propertyId']
 sql+=f'INSERT INTO object VALUES (987654321,{oid}),(123,-2);\n'
 if present:
  sql+=f"INSERT INTO row_home_state VALUES (10,'object',987654321,{oid},NULL,NULL,{oid},{pid},20);\n"
  sql+='INSERT INTO row_home_node VALUES (10,20,NULL);\n'
  sql+=f"INSERT INTO row_home_scalar VALUES (10,20,'string','é  ',NULL,NULL,NULL,pg_catalog.decode('{emission['codecHex']}','hex'),pg_catalog.decode('fe00','hex'));\n"
 sql+=f"PREPARE query({types}) AS {emission['sql']}; EXECUTE query({values}); ROLLBACK;\n"
 output=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__native_null__','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
 rows=list(csv.reader(io.StringIO(output)));assert rows[0]==headers
 actual=[[None if v=='__native_null__' else v for v in row] for row in rows[1:]]
 expected=[['true','string','é  ',None,None,None,emission['codecHex'],'fe00']] if present else [['false']+[None]*7]
 assert actual==expected,(name,actual,expected)
 results.append({'id':name,'values':actual,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt.write_text(json.dumps({'scope':'Original UMF property/binding/native-home lowered custody query on synthetic unconstrained object row storage; no source semantic, tree, logical decoding, native layout or host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},ensure_ascii=False,indent=2)+'\n')
print('2 original-property/native-home custody queries passed.')
