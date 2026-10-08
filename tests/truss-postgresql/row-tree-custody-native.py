import csv,hashlib,io,json,subprocess
from pathlib import Path
capture=Path('tests/truss-postgresql/fixtures/row-tree-custody-emission.json');raw=capture.read_bytes();emissions=json.loads(raw);results=[]
for e in emissions:
 kind=e['kind'];owner_table=kind;discriminator='type_id' if kind=='object' else 'rel_type_id'
 for case,expected in [('absent',0),('container',1),('descendant',2),('orphan-payload',3),('duplicate-payload',3)]:
  sql='BEGIN; CREATE SCHEMA "schema.with.dot"; SET LOCAL search_path="schema.with.dot",pg_catalog;\n'
  sql+=f'CREATE TABLE {owner_table}(id bigint,{discriminator} int); INSERT INTO {owner_table} VALUES (9007199254740993,-1);\n'
  sql+='CREATE TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);\n'
  sql+='CREATE TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);\n'
  sql+='CREATE TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);\n'
  if case!='absent':
   owner_cells='9007199254740993,-1,NULL,NULL' if kind=='object' else 'NULL,NULL,9007199254740993,-1'
   sql+=f"INSERT INTO row_home_state VALUES (1,'{kind}',{owner_cells},-1,42,10);\nINSERT INTO row_home_node VALUES (1,10,NULL,'sequence','root',NULL,NULL,NULL,decode('00ff','hex'),decode('','hex'));\n"
  if case in ['descendant','orphan-payload','duplicate-payload']:
   sql+="INSERT INTO row_home_node VALUES (1,11,10,'scalar','sequence',0,NULL,NULL,decode('61','hex'),decode('62','hex'));\n"
   sql+="INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES (1,11,'integer',18446744073709551615,'18446744073709551615',decode('0063','hex'),decode('64','hex'));\n"
  if case=='orphan-payload':sql+="INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value) VALUES (1,99,'string','');\n"
  if case=='duplicate-payload':sql+="INSERT INTO row_home_scalar SELECT * FROM row_home_scalar WHERE node_id=11;\n"
  sql+="INSERT INTO row_home_node VALUES (2,200,NULL,'null','root',NULL,NULL,NULL,decode('','hex'),decode('','hex'));\n"
  sql+=f"PREPARE custody(int4,int4) AS SELECT {e['sql']} AS custody FROM {owner_table} owner {' '.join(e['joins'])}; EXECUTE custody(-1,42); ROLLBACK;\n"
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));assert rows[0]==['custody'] and len(rows)==2;bag=json.loads(rows[1][0]);assert len(bag)==expected,(kind,case,bag)
  for row in bag:assert len(row)==23 and all(v is None or isinstance(v,str) for v in row) and row[1]!='200'
  for row in bag:
   if row[1]=='10':assert row[8]=='00ff' and row[9]==''
   if row[1]=='11':assert row[15:17]==['18446744073709551615']*2 and row[21:23]==['0063','64']
  if case=='orphan-payload':assert any(row[1] is None and row[11]=='99' and row[13]=='' for row in bag)
  if case=='duplicate-payload':assert sum(row[1]=='11' for row in bag)==2
  results.append({'kind':kind,'case':case,'observedRows':len(bag),'values':bag,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-row-tree-custody-native.json').write_text(json.dumps({'scope':'Private complete-state native node/payload custody bag only; retains malformed duplicates/orphans without semantic admission or public result qualification','captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n');print('10 complete-state row custody native cases passed.')
