"""Original relationship reads with independently selected endpoint homes."""
import csv, hashlib, io, json, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
path=ROOT/'tests/truss-postgresql/fixtures/original-mixed-relationship-public.json'
raw=path.read_bytes(); captures=json.loads(raw); results=[]
def insert(e,owner,value,customer,case):
 type_id=-1 if customer else -2
 member='0' if customer else '6'
 home=e['customerHome' if customer else 'orderHome']
 if home=='props':
  props={} if customer and owner==1 and case=='missing-key' else {member:value}
  return f"INSERT INTO object VALUES ({owner},{type_id},'{json.dumps(props)}'::jsonb);\n"
 sql=f"INSERT INTO object VALUES ({owner},{type_id},NULL);\n"
 if customer and owner==1 and case=='missing-key':return sql
 state=owner+40;node=owner+50;codec=e['customerCodecHex' if customer else 'orderCodecHex']
 return sql+f"INSERT INTO row_home_state VALUES ({state},'object',{owner},{type_id},NULL,NULL,{type_id},{member},{node}); INSERT INTO row_home_node VALUES ({state},{node},NULL); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'integer',{value},'{value}',decode('{codec}','hex'),decode('fe00','hex'));\n"
for e in captures:
 for case in ['empty','exact-bag-lookahead','hidden-overflow','hidden-source-overflow','duplicate-key','missing-key','fractional-key','dangling-target','wrong-endpoint-type']:
  corrupt=case not in ['empty','exact-bag-lookahead']
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb); CREATE TEMP TABLE edge(rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int);\n'
  sql+='CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint); CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint); CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);\n'
  orders=[(10,'100'),(11,'101'),(12,'102')]
  if case=='hidden-source-overflow':orders.append((13,'18446744073709551616'))
  for owner,value in orders:
   sql+=insert(e,owner,value,False,case)
  if case!='empty':
   values=['18446744073709551615','0','9007199254740993']
   if case=='hidden-overflow':values.append('18446744073709551616')
   if case=='duplicate-key':values[0]=values[1]
   for i,value in enumerate(values):
    if i==0 and case=='fractional-key':value='1.5'
    sql+=insert(e,i+1,value,True,case)
   sql+='INSERT INTO edge VALUES (0,1,-1,10,-2),(0,2,-1,10,-2),(0,3,-1,10,-2),(0,3,-1,11,-2),(0,3,-1,11,-2),(1,1,-1,11,-2);\n'
   if case=='dangling-target':sql+='INSERT INTO edge VALUES (0,999,-1,10,-2);\n'
   if case=='wrong-endpoint-type':sql+='INSERT INTO edge VALUES (0,1,-999,10,-2);\n'
  args=','.join("'"+p['value']+"'" for p in e['parameters'])
  types=','.join('numeric' if p['logicalType']['facets'].get('integerWidth',{}).get('bits')==64 else 'int4' for p in e['parameters'])
  for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {e['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n'
  rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  counts=[]
  for i in range(len(e['checks'])):
   assert rows[i*2]==['violations'];counts.append(int(rows[i*2+1][0]))
  assert any(counts) if corrupt else not any(counts),(e['direction'],e['kind'],case,counts)
  remaining=rows[len(counts)*2:]
  if not corrupt:
   if e['kind']=='has':assert remaining==[['n'],['0' if case=='empty' else ('3' if e['direction']=='forward' else '2')]],remaining
   else:
    parsed=[(r[0],json.loads(r[1])) for r in remaining[1:]]
    empty=dict(items=[],truncated=False)
    if e['direction']=='forward':expected=[] if case=='empty' else [('0',dict(items=[['100']],truncated=False)),('9007199254740993',dict(items=[['100'],['101']],truncated=True)),('18446744073709551615',dict(items=[['100']],truncated=False))]
    else:expected=[('100',empty),('101',empty),('102',empty)] if case=='empty' else [('100',dict(items=[['0'],['9007199254740993']],truncated=True)),('101',dict(items=[['9007199254740993'],['9007199254740993']],truncated=False)),('102',empty)]
    assert parsed==expected,parsed
  else:assert not remaining
  results.append(dict(customerHome=e['customerHome'],orderHome=e['orderHome'],direction=e['direction'],kind=e['kind'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original uint64 endpoint keys over mixed native row and JSONB props homes, both directions; synthetic PostgreSQL fixtures, not production or embedding qualification',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-mixed-relationship-native.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} original mixed-home relationship native cases passed')
