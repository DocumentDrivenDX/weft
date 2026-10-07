import csv,hashlib,io,json,subprocess
from pathlib import Path
emissions=json.loads(Path('tests/truss-postgresql/fixtures/row-tree-custody-emission.json').read_text())
results=[]
def value(cell,index):
 if cell is None:return 'NULL'
 if index in [7,8,9,19,20,21,22]:return "decode('"+cell+"','hex')"
 return "'"+cell.replace("'","''")+"'"
for fixture in sorted(Path('tests/truss-postgresql/fixtures').glob('original-*-native-tree.json')):
 data=json.loads(fixture.read_text())
 for e in emissions:
  kind=e['kind']; discriminator='type_id' if kind=='object' else 'rel_type_id'
  sql='BEGIN; CREATE SCHEMA "schema.with.dot"; SET LOCAL search_path="schema.with.dot",pg_catalog;\n'
  sql+=f'CREATE TABLE {kind}(id bigint,{discriminator} int); INSERT INTO {kind} VALUES (9007199254740993,-1);\n'
  sql+='CREATE TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);\n'
  sql+='CREATE TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);\n'
  sql+='CREATE TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);\n'
  owner_cells='9007199254740993,-1,NULL,NULL' if kind=='object' else 'NULL,NULL,9007199254740993,-1'
  sql+=f"INSERT INTO row_home_state VALUES (1,'{kind}',{owner_cells},-1,42,1);\n"
  for cells in data['cells']:
   sql+='INSERT INTO row_home_node VALUES ('+','.join(value(v,i) for i,v in enumerate(cells[:10]))+');\n'
   if cells[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(value(v,i) for i,v in enumerate(cells[10:],10))+');\n'
  sql+=f"PREPARE custody(int4,int4) AS SELECT {e['sql']} AS custody FROM {kind} owner {' '.join(e['joins'])}; EXECUTE custody(-1,42); ROLLBACK;\n"
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(out)));assert rows[0]==['custody'] and len(rows)==2
  bag=json.loads(rows[1][0]);assert bag==data['cells'],(fixture.name,kind,bag,data['cells'])
  results.append({'fixture':data['fixture'],'kind':kind,'cells':bag,'fixtureSha256':hashlib.sha256(fixture.read_bytes()).hexdigest(),'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-original-recursive-row-native.json').write_text(json.dumps({'scope':'14 PostgreSQL custody round trips of original UMF-admitted compound fixture trees; fixture-selected JSON field identity procedure, not adopted Truss encoding; public recursive row emission remains unqualified','harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print(f'{len(results)} original compound native custody round trips passed.')
