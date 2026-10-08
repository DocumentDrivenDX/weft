"""Original inverse relationship subqueries plus every emitted prerequisite."""
import csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];path=ROOT/'tests/truss-postgresql/fixtures/original-relationship-lowering.json';raw=path.read_bytes();captures=json.loads(raw);results=[]
for e in captures:
 for case in ['empty','exact-bag-lookahead','hidden-overflow','wrong-codec','duplicate-key','dangling-target','wrong-endpoint-type']:
  corrupt=case not in ['empty','exact-bag-lookahead']
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int);
CREATE TEMP TABLE edge(rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
INSERT INTO object VALUES (10,-2),(11,-2),(12,-2);
'''
  if case!='empty':
   sql+="INSERT INTO edge VALUES (0,1,-1,10,-2),(0,2,-1,10,-2),(0,3,-1,10,-2),(0,3,-1,11,-2),(0,3,-1,11,-2),(1,1,-1,11,-2);\n"
   values=['18446744073709551615','0','9007199254740993']
   if case=='hidden-overflow':values.append('18446744073709551616')
   if case=='duplicate-key':values[0]=values[1]
   for i,value in enumerate(values):
    owner=i+1;state=i+20;node=i+30;codec='00' if case=='wrong-codec' and i==0 else e['codecHex']
    sql+=f"INSERT INTO object VALUES ({owner},-1); INSERT INTO row_home_state VALUES ({state},'object',{owner},-1,NULL,NULL,-1,0,{node}); INSERT INTO row_home_node VALUES ({state},{node},NULL); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'integer',{value},'{value}',decode('{codec}','hex'),decode('fe00','hex'));\n"
   if case=='dangling-target':sql+="INSERT INTO edge VALUES (0,999,-1,10,-2);\n"
   if case=='wrong-endpoint-type':sql+="INSERT INTO edge VALUES (0,1,-999,10,-2);\n"
  args=','.join("'"+p['value']+"'" for p in e['parameters'])
  types=','.join('numeric' if p['logicalType']['facets'].get('integerWidth',{}).get('bits')==64 else 'int4' for p in e['parameters'])
  for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {e['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n'
  rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  counts=[]
  for i in range(len(e['checks'])):
   assert rows[i*2]==['violations'];counts.append(int(rows[i*2+1][0]))
  assert any(counts) if corrupt else not any(counts),(e['kind'],case,counts)
  remaining=rows[len(counts)*2:]
  if not corrupt:
   assert remaining[0]==['owner','related']
   if e['kind']=='has':assert remaining[1:]==[['10','f' if case=='empty' else 't'],['11','f' if case=='empty' else 't'],['12','f']],remaining
   else:
    parsed=[(r[0],json.loads(r[1])) for r in remaining[1:]]
    empty=dict(items=[],truncated=False)
    expected=[('10',empty),('11',empty),('12',empty)] if case=='empty' else [('10',dict(items=[['0'],['9007199254740993']],truncated=True)),('11',dict(items=[['9007199254740993'],['9007199254740993']],truncated=False)),('12',empty)]
    assert parsed==expected,parsed
  else:assert not remaining
  results.append(dict(kind=e['kind'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original inverse relationship subquery lowering over native uint64 target keys; not public compiler/embedding qualification or production compatibility',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-relationship-lowering-native.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} original relationship lowering native cases passed')
