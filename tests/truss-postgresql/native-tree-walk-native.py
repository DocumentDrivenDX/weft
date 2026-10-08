import csv,hashlib,io,json,subprocess
from pathlib import Path
results=[]
def value(cell,index):
 if cell is None:return 'NULL'
 if index in [7,8,9,19,20,21,22]:return "decode('"+cell+"','hex')"
 return "'"+cell.replace("'","''")+"'"
for emission in sorted(Path('tests/truss-postgresql/fixtures').glob('original-*-native-walk.json')):
 e=json.loads(emission.read_text());tree=json.loads(emission.with_name('original-'+e['fixture']+'-native-tree.json').read_text())
 for case in ['valid','orphan','duplicate','wrong-shape']:
  cells=[list(row) for row in tree['cells']]
  if case=='orphan':cells[-1][2]='999'
  if case=='duplicate':cells.append(list(cells[-1]))
  if case=='wrong-shape':cells[0][3]='map' if cells[0][3]!='map' else 'sequence'
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int); INSERT INTO object VALUES (9007199254740993,-1);\n'
  sql+='CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);\n'
  sql+='CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);\n'
  sql+='CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);\n'
  sql+=f"INSERT INTO row_home_state VALUES (1,'object',9007199254740993,-1,NULL,NULL,-1,{e['propertyId']},1);\n"
  for row in cells:
   sql+='INSERT INTO row_home_node VALUES ('+','.join(value(v,i) for i,v in enumerate(row[:10]))+');\n'
   if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(value(v,i) for i,v in enumerate(row[10:],10))+');\n'
  parameters=','.join("'"+p['value'].replace("'","''")+"'" for p in e['parameters'])
  sql+='PREPARE walk(text,text,text,text) AS '+e['sql']+'; EXECUTE walk('+parameters+'); ROLLBACK;\n'
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
  rows=list(csv.reader(io.StringIO(out)));assert rows[0]==['mapping'] and len(rows)==2
  mapping=json.loads(rows[1][0]);valid=mapping['unmatchedRows']==0 and mapping['uniqueNodes'] and mapping['matchingShapes']
  assert valid==(case=='valid'),(e['fixture'],case,mapping)
  if case=='valid':
   assert len(mapping['rows'])==len(cells)
   assert sorted(row['cells'] for row in mapping['rows'])==sorted(cells)
   expected={}
   def paths(v,path=()):
    expected[path]=v
    if isinstance(v,list):
     for i,child in enumerate(v):paths(child,path+(str(i),))
    elif isinstance(v,dict):
     for key,child in v.items():paths(child,path+(key,))
   paths(tree['stored'])
   assert {tuple(row['path']) for row in mapping['rows']}==set(expected)
   for row in mapping['rows']:
    path=tuple(row['path']);assert row['depth']==len(path)
    if isinstance(expected[path],str):assert (row['cells'][13] if row['cells'][13] is not None else row['cells'][16])==expected[path]
  results.append({'fixture':e['fixture'],'case':case,'mapping':mapping,'captureSha256':hashlib.sha256(emission.read_bytes()).hexdigest(),'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
Path('docs/helix/04-build/evidence/B-005-native-tree-walk-native.json').write_text(json.dumps({'scope':'Original topology pairing SQL, partial integrity only; no public carrier or complete native structure/source qualification','harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print(f'{len(results)} native topology walk SQL cases passed.')
