import csv,hashlib,io,json,subprocess
from pathlib import Path
results=[]
def value(cell,index):
 if cell is None:return 'NULL'
 if index in [7,8,9,19,20,21,22]:return "decode('"+cell+"','hex')"
 return "'"+cell.replace("'","''")+"'"
for emission in sorted(Path('tests/truss-postgresql/fixtures').glob('original-*-native-guarded-body.json')):
 e=json.loads(emission.read_text());tree=json.loads(emission.with_name('original-'+e['fixture']+'-native-tree.json').read_text())
 cases=['valid','node-definition','node-source','leaf-codec','leaf-source','leaf-kind','extra-slot','missing-leaf-source']
 if any(row[16] is not None for row in tree['cells']):cases+=['native-token-mismatch','noncanonical-token','invalid-token']
 for case in cases:
  cells=[list(row) for row in tree['cells']];leaf=next(row for row in cells if row[12] is not None)
  if case=='node-definition':cells[0][8]='ff'
  if case=='node-source':cells[0][9]='ff'
  if case=='leaf-codec':leaf[21]='ff'
  if case=='leaf-source':leaf[22]='ff'
  if case=='leaf-kind':leaf[12]='unknown'
  if case=='extra-slot':leaf[14]='true'
  if case=='missing-leaf-source':leaf[22]=None
  if case in ['native-token-mismatch','noncanonical-token','invalid-token']:
   number=next(row for row in cells if row[16] is not None)
   if case=='native-token-mismatch':number[15]='1'
   elif case=='noncanonical-token':number[15]='0';number[16]='00'
   else:number[16]='x'
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int); INSERT INTO object VALUES (9007199254740993,-1);\n'
  sql+='CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);\n'
  sql+='CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);\n'
  sql+='CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);\n'
  sql+=f"INSERT INTO row_home_state VALUES (1,'object',9007199254740993,-1,NULL,NULL,-1,{e['propertyId']},1);\n"
  for row in cells:
   sql+='INSERT INTO row_home_node VALUES ('+','.join(value(v,i) for i,v in enumerate(row[:10]))+');\n'
   if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(value(v,i) for i,v in enumerate(row[10:],10))+');\n'
  parameters=','.join("'"+p['value'].replace("'","''")+"'" for p in e['parameters'])
  sql+='PREPARE walk(text,text,text,text,text,text) AS '+e['sql']+'; EXECUTE walk('+parameters+'); ROLLBACK;\n'
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(out)));assert rows[0]==['stored','logical','integrity','structure','native'] and len(rows)==2
  stored=json.loads(rows[1][0]);logical=json.loads(rows[1][1]);integrity=rows[1][2]=='t';structure=rows[1][3]=='t';native=rows[1][4]=='t'
  assert native==(case=='valid'),(e['fixture'],case,rows[1])
  assert structure==(case!='missing-leaf-source'),(e['fixture'],case,'independent physical custody')
  if case=='valid':assert integrity and stored==tree['stored'] and logical==tree['logical']
  if case in ['native-token-mismatch','noncanonical-token']:assert integrity,'Logical values alone must not bypass source/token interpretation'
  results.append({'fixture':e['fixture'],'case':case,'integrity':integrity,'structure':structure,'native':native,'captureSha256':hashlib.sha256(emission.read_bytes()).hexdigest(),'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-native-tree-guarded-native.json').write_text(json.dumps({'scope':'Fixture-selected native source/typed scalar SQL predicates, separate from logical integrity; not adopted Truss codecs or public projection','harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print(f'{len(results)} native selected-procedure SQL cases passed.')
