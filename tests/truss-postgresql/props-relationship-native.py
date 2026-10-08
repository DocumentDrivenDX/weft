"""Original relationship reads over props homes and all emitted prerequisites."""
import csv, hashlib, io, json, subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
path=ROOT/'tests/truss-postgresql/fixtures/original-props-relationship-public.json'
raw=path.read_bytes(); captures=json.loads(raw); results=[]
for e in captures:
 for case in ['empty','exact-bag-lookahead','hidden-overflow','hidden-source-overflow','duplicate-key','missing-key','wrong-json-kind','invalid-number','fractional-key','dangling-target','wrong-endpoint-type']:
  corrupt=case not in ['empty','exact-bag-lookahead']
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb); CREATE TEMP TABLE edge(rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int);\n'
  orders=[(10,'100'),(11,'101'),(12,'102')]
  if case=='hidden-source-overflow':orders.append((13,'18446744073709551616'))
  for owner,value in orders:
   props=json.dumps({'6':value})
   sql+=f"INSERT INTO object VALUES ({owner},-2,'{props}'::jsonb);\n"
  if case!='empty':
   values=['18446744073709551615','0','9007199254740993']
   if case=='hidden-overflow':values.append('18446744073709551616')
   if case=='duplicate-key':values[0]=values[1]
   for i,value in enumerate(values):
    props={'0':value}
    if i==0:
     if case=='missing-key':props={}
     elif case=='wrong-json-kind':props={'0':100}
     elif case=='invalid-number':props={'0':'invalid'}
     elif case=='fractional-key':props={'0':'1.5'}
    sql+=f"INSERT INTO object VALUES ({i+1},-1,'{json.dumps(props)}'::jsonb);\n"
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
  results.append(dict(direction=e['direction'],kind=e['kind'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original uint64 endpoint keys over JSONB props homes, both directions; synthetic PostgreSQL fixtures, not production or embedding qualification',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-props-relationship-native.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} original props relationship native cases passed')
