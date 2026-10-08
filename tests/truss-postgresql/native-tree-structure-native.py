import csv,hashlib,io,json,subprocess
from pathlib import Path
results=[]
def value(cell,index):
 if cell is None:return 'NULL'
 if index in [7,8,9,19,20,21,22]:return "decode('"+cell+"','hex')"
 return "'"+cell.replace("'","''")+"'"
for emission in sorted(Path('tests/truss-postgresql/fixtures').glob('original-*-native-structure-body.json')):
 e=json.loads(emission.read_text());tree=json.loads(emission.with_name('original-'+e['fixture']+'-native-tree.json').read_text())
 cases=['valid','orphan','duplicate','wrong-slot','root-slot','container-payload','missing-source']
 if any(row[4]=='sequence' for row in tree['cells']):cases.append('ordinal-gap')
 if any(row[4]=='map' for row in tree['cells']):cases.append('duplicate-map-key')
 if any(row[4]=='record' for row in tree['cells']):cases.append('unknown-field')
 for case in cases:
  cells=[list(row) for row in tree['cells']]
  if case=='orphan':cells[-1][2]='999'
  if case=='duplicate':cells.append(list(cells[-1]))
  if case=='wrong-slot':cells[-1][4]='root'
  if case=='root-slot':cells[0][5]='0'
  if case=='container-payload':
   cells[0][10:]=list(cells[-1][10:]);cells[0][10]='1';cells[0][11]='1'
  if case=='missing-source':cells[-1][9]=None
  if case=='ordinal-gap':next(row for row in cells if row[4]=='sequence')[5]='99'
  if case=='duplicate-map-key':
   row=next(row for row in cells if row[4]=='map');second=list(row);second[1]='999';second[11]='999';cells.append(second)
  if case=='unknown-field':next(row for row in cells if row[4]=='record')[7]='ff'
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
  rows=list(csv.reader(io.StringIO(out)));assert rows[0]==['stored','logical','integrity','structure'] and len(rows)==2
  structure=rows[1][3]=='t'
  assert structure==(case=='valid'),(e['fixture'],case,rows[1])
  results.append({'fixture':e['fixture'],'case':case,'structure':structure,'captureSha256':hashlib.sha256(emission.read_bytes()).hexdigest(),'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-native-tree-structure-native.json').write_text(json.dumps({'scope':'Native structural topology/slot/payload gate, separate from source/codec interpretation and logical presence','harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print(f'{len(results)} native structural SQL cases passed.')
