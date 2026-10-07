import csv,hashlib,io,json,subprocess
from pathlib import Path
results=[]
def value(cell,index):
 if cell is None:return 'NULL'
 if index in [7,8,9,19,20,21,22]:return "decode('"+cell+"','hex')"
 return "'"+cell.replace("'","''")+"'"
for emission in sorted(Path('tests/truss-postgresql/fixtures').glob('original-*-native-public.json')):
 e=json.loads(emission.read_text());tree=json.loads(emission.with_name('original-'+e['fixture']+'-native-tree.json').read_text())
 cases=['valid','hidden-corruption','foreign-owner']
 for case in cases:
  cells=[list(row) for row in tree['cells']]
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int); INSERT INTO object VALUES (9007199254740993,-1);\n'
  sql+='CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);\n'
  sql+='CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);\n'
  sql+='CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);\n'
  if case!='absent':sql+=f"INSERT INTO row_home_state VALUES (1,'object',9007199254740993,-1,NULL,NULL,-1,{e['propertyId']},1);\n"
  for row in (cells if case!='absent' else []):
   sql+='INSERT INTO row_home_node VALUES ('+','.join(value(v,i) for i,v in enumerate(row[:10]))+');\n'
   if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(value(v,i) for i,v in enumerate(row[10:],10))+');\n'
  if case in ['hidden-corruption','foreign-owner']:
   owner_type=-1 if case=='hidden-corruption' else -2
   sql+=f"INSERT INTO object VALUES (9007199254740994,{owner_type}); INSERT INTO row_home_state VALUES (2,'object',9007199254740994,{owner_type},NULL,NULL,-1,{e['propertyId']},101);\n"
   hidden=[list(row) for row in cells]
   for row in hidden:
    row[0]='2';row[1]=str(int(row[1])+100)
    if row[2] is not None:row[2]=str(int(row[2])+100)
    if row[10] is not None:row[10]='2';row[11]=str(int(row[11])+100)
   hidden[-1][9]='ff'
   for row in hidden:
    sql+='INSERT INTO row_home_node VALUES ('+','.join(value(v,i) for i,v in enumerate(row[:10]))+');\n'
    if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(value(v,i) for i,v in enumerate(row[10:],10))+');\n'
  parameters=','.join("'"+p['value'].replace("'","''")+"'" for p in e['parameters'])
  types=','.join(['text']*len(e['parameters']))
  sql+='PREPARE projection('+types+') AS '+e['sql']+'; EXECUTE projection('+parameters+');\n'
  for i,check in enumerate(e['checks']):sql+=f'PREPARE prerequisite_{i}('+types+') AS '+check+f'; EXECUTE prerequisite_{i}('+parameters+');\n'
  sql+='ROLLBACK;\n'
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(out)));first_check=next(i for i,row in enumerate(rows) if row==['violations'])
  carriers=[json.loads(row[0]) for row in rows[1:first_check]]
  expected={'state':'value','value':tree['logical']}
  assert all(carrier==expected for carrier in carriers)
  assert len(carriers)==(2 if case=='hidden-corruption' else 1),(e['fixture'],case,carriers)
  counts=[]
  for i in range(first_check,len(rows),2):assert rows[i]==['violations'];counts.append(int(rows[i+1][0]))
  assert len(counts)==len(e['checks']) and (any(counts))==(case=='hidden-corruption'),(e['fixture'],case,counts)
  results.append({'fixture':e['fixture'],'case':case,'carriers':carriers,'violations':counts,'captureSha256':hashlib.sha256(emission.read_bytes()).hexdigest(),'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-native-tree-public-native.json').write_text(json.dumps({'scope':'Public original backend registry native compound SELECT and prerequisites, with explicitly selected fixture procedures; Python/WASM unqualified','harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print(f'{len(results)} public native compiler SQL cases passed.')
