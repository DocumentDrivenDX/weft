"""Original optional scalar and complete entity native evidence.
@covers US-003-AC1: independent exact scalar/presence outputs for this subset.
@covers US-003-AC2: original member order and logical/native identifiers.
@covers US-003-AC3: null and corrupt stored meanings report violations.
"""
import csv,hashlib,io,json,subprocess
import os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
path=F/'original-optional-scalar-public.json';raw=path.read_bytes();captures=json.loads(raw)
inputs=json.loads((F/'original-optional-scalar-inputs.json').read_text());binding=json.loads(inputs['requests'][0]['request']['target']['bindingJson'])
codecs={int(binding['properties'][p['index']]['propertyId']):p['leafCodecs']['root']['originalJson'].encode().hex() for p in inputs['composition']['properties']}
results=[]
for e in captures:
 cases=['empty-owners','absence-empty-unicode','unexpected-null','wrong-value-kind','sql-null-carrier']+(['missing-root','duplicate-state','orphan-scalar','wrong-codec','null-node-with-payload'] if e['home']=='row' else ['root-array'])
 for case in cases:
  corrupt=case not in ['empty-owners','absence-empty-unicode']
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
  owners=[] if case=='empty-owners' else [(1,'1','A',None),(2,'9007199254740993','A',''),(3,'18446744073709551615','B','é  '),(4,'0','C','last')]
  for owner,id,part,note in owners:
   props={} if note is None else {'23':note}
   if e['home']=='props' and owner==4:
    if case=='unexpected-null':props={'23':None}
    elif case=='wrong-value-kind':props={'23':17}
    elif case=='root-array':props=[]
   root='NULL' if e['home']=='props' and owner==4 and case=='sql-null-carrier' else "'"+json.dumps(props).replace("'","''")+"'::jsonb"
   sql+=f'INSERT INTO object VALUES ({owner},-1,{root});\n'
   fields=[(0,'integer',id),(20,'string',part)]+([(23,'string',note)] if e['home']=='row' and note is not None else [])
   for pid,family,value in fields:
    state=owner*100+pid;node=state+10000
    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},-1,NULL,NULL,-1,{pid},{node});\n"
    if pid==23 and owner==4 and case=='duplicate-state':sql+=f"INSERT INTO row_home_state VALUES ({state+5000},'object',{owner},-1,NULL,NULL,-1,{pid},{node});\n"
    if pid==23 and owner==4 and case=='missing-root':continue
    null_node=pid==23 and owner==4 and case in ['unexpected-null','null-node-with-payload']
    sql+=f"INSERT INTO row_home_node VALUES ({state},{node},NULL,'{'null' if null_node else 'scalar'}',decode('00','hex'),decode('6e756c6c','hex'));\n"
    if null_node and case!='null-node-with-payload':continue
    code='00' if pid==23 and owner==4 and case=='wrong-codec' else codecs[pid]
    text="'"+value.replace("'","''")+"'" if family=='string' else 'NULL';numeric=value if family=='integer' else 'NULL';token="'"+value+"'" if family=='integer' else 'NULL'
    if pid==23 and owner==4 and case=='sql-null-carrier':text='NULL'
    if pid==23 and owner==4 and case=='wrong-value-kind':family='integer'
    if pid==23 and owner==4 and case=='orphan-scalar':node+=1
    sql+=f"INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'{family}',{text},{numeric},{token},decode('{code}','hex'),decode('fe00','hex'));\n"
  args=','.join("'"+p['value'].replace("'","''")+"'" for p in e['parameters']);types=','.join('int4' for p in e['parameters'])
  for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {e['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n'
  rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  counts=[]
  for i in range(len(e['checks'])):
   assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
  assert any(counts) if corrupt else not any(counts),(e['home'],e['kind'],case,counts)
  remaining=rows[len(counts)*2:]
  if not corrupt:
   assert remaining[0]==(['note','id','part'] if e['kind'].startswith('entity') else ['id','note']),remaining
   expected=[]
   for owner,id,part,note in owners:
    presence=dict(state='absent') if note is None else dict(state='value',value=note)
    expected.append((presence,id,part) if e['kind'].startswith('entity') else (id,presence))
   if e['kind']=='entity-bounded':expected=expected[:2]
   parsed=[(json.loads(r[0]),r[1],r[2]) if e['kind'].startswith('entity') else (r[0],json.loads(r[1])) for r in remaining[1:]]
   assert parsed==expected,parsed
  else:assert not remaining
  results.append(dict(home=e['home'],kind=e['kind'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original optional Unicode scalar and complete three-scalar entity over props/native row homes; native null explicitly unsupported by pinned presence profile; synthetic PostgreSQL fixtures only',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
Path(os.environ.get("WEFT_EVIDENCE_OUTPUT", ROOT/"docs/helix/04-build/evidence/B-005-optional-scalar-native.json")).write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} original optional scalar/entity native cases passed')
