"""Independent Boolean original property native checks.
@covers US-003-AC1: Boolean order/filter exactness across native/props storage.
@covers US-003-AC3: invalid carriers, absent required state and duplicates refuse.
"""
import csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
transports=json.loads((F/'original-boolean-public-transport.json').read_text());inputs=json.loads((F/'original-boolean-inputs.json').read_text());results=[]
for t in transports:
 response=t['response'];home=t['home'];checks=[o['parameters']['sql'] for o in response['obligations'] if 'sql' in o['parameters']];composition=next(x['composition'] for x in inputs if x['home']==home);codec=composition['properties'][0]['leafCodecs']['root']['originalJson'].encode().hex()
 for case in ['empty','exact','required-absent','wrong-carrier','null-carrier','duplicate-key','wrong-codec']:
  corrupt=case not in ['empty','exact'];values=[] if case=='empty' else [False,False] if case=='duplicate-key' else [False,True]
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
  if home=='props' and case=='wrong-codec':continue
  for i,value in enumerate(values):
   props={} if case=='required-absent' else {'0': str(value).lower() if case=='wrong-carrier' else None if case=='null-carrier' else value}
   sql+=f"INSERT INTO object VALUES ({i+1},-1,'{json.dumps(props)}'::jsonb);\n"
   if home=='props' or case=='required-absent':continue
   code='00' if case=='wrong-codec' else codec;boolean='NULL' if case in ['wrong-carrier','null-carrier'] else str(value).lower();kind='integer' if case=='wrong-carrier' else 'boolean'
   sql+=f"INSERT INTO row_home_state VALUES ({i+10},'object',{i+1},-1,NULL,NULL,-1,0,{i+20}); INSERT INTO row_home_node VALUES ({i+10},{i+20},NULL); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,boolean_value,codec_definition_bytes,original_source_bytes) VALUES ({i+10},{i+20},'{kind}',{boolean},decode('{code}','hex'),decode('fe00','hex'));\n"
  sql+="INSERT INTO object VALUES (999,-99,'{}'::jsonb);\n"
  args=','.join("'"+p['value'].replace("'","''")+"'" for p in response['parameters']);types=','.join('text' for p in response['parameters'])
  for i,check in enumerate(checks):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {response['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n';rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  counts=[]
  for i in range(len(checks)):assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
  assert any(counts) if corrupt else not any(counts),(home,t['kind'],case,counts)
  remaining=rows[len(counts)*2:]
  if not corrupt:
   expected=[[str(v).lower()] for v in values if t['kind']=='page' or str(v).lower()==t['kind']]
   assert remaining==[['id']]+expected,(home,t['kind'],case,remaining)
  else:assert not remaining
  results.append(dict(home=home,kind=t['kind'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original Boolean required property ordering/equality on synthetic native/props fixtures; no runtime embedding or production qualification',server=server,cases=len(results),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),transportSha256=hashlib.sha256((F/'original-boolean-public-transport.json').read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-original-boolean-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} original Boolean native cases passed')
