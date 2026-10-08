"""Independent native complete entity proof for multiple recursive roots.
@covers US-003-AC1: declared order and exact Boolean/structured/scalar values.
@covers US-003-AC3: each root's corruption blocks even a one-row page.
"""
import copy,csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
cut=json.loads((F/'original-multi-recursive-entity-inputs.json').read_text());binding=json.loads(cut['requests'][0]['request']['target']['bindingJson']);transports=json.loads((F/'original-multi-recursive-entity-public.json').read_text());trees={4:json.loads((F/'original-boolean-sequence-tree.json').read_text()),5:json.loads((F/'original-numeric-address-native-tree.json').read_text())};codecs={int(binding['properties'][p['index']]['propertyId']):p['leafCodecs']['root']['originalJson'].encode().hex() for p in cut['composition']['properties'] if not p.get('nativeTree',True)};results=[]
def cell(v,i):
 if v is None:return 'NULL'
 if i in [7,8,9,19,20,21,22]:return "decode('"+v+"','hex')"
 return "'"+v.replace("'","''")+"'"
for t in transports:
 r=t['response'];checks=[o['parameters']['sql'] for o in r['obligations'] if 'sql' in o['parameters']]
 for case in ['exact','empty-sequence','absent-address','missing-sequence','hidden-sequence-codec','hidden-address-codec','hidden-key-codec','absent-note']:
  corrupt=case in ['missing-sequence','hidden-sequence-codec','hidden-address-codec','hidden-key-codec']
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);
'''
  owners=[(1,'9007199254740993','A',None),(2,'18446744073709551615','B','')]
  for owner,id,part,note in owners:
   sql+=f"INSERT INTO object VALUES ({owner},-1,'{{}}'::jsonb);\n"
   scalars=[(0,'integer',id),(20,'string',part)]+([(23,'string',note)] if note is not None and case!='absent-note' else [])
   for pid,family,value in scalars:
    state=owner*100+pid;node=state+10000;codec='00' if case=='hidden-key-codec' and owner==2 and pid==0 else codecs[pid];text="'"+value+"'" if family=='string' else 'NULL';numeric=value if family=='integer' else 'NULL';token="'"+value+"'" if family=='integer' else 'NULL'
    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},-1,NULL,NULL,-1,{pid},{node}); INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind) VALUES ({state},{node},NULL,'scalar'); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'{family}',{text},{numeric},{token},decode('{codec}','hex'),decode('fe00','hex'));\n"
   for pid,tree in trees.items():
    if owner==2 and ((case=='absent-address' and pid==5) or (case=='missing-sequence' and pid==4)):continue
    state=owner*1000+pid;rows=copy.deepcopy(tree['cells']);sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},-1,NULL,NULL,-1,{pid},1);\n"
    if case=='empty-sequence' and pid==4:rows=rows[:1]
    if owner==2 and ((case=='hidden-sequence-codec' and pid==4) or (case=='hidden-address-codec' and pid==5)):rows[-1][21]='00'
    for row in rows:
     row[0]=str(state)
     if row[10] is not None:row[10]=str(state)
     sql+='INSERT INTO row_home_node VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[:10]))+');\n'
     if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[10:],10))+');\n'
  sql+="INSERT INTO object VALUES (999,-999,'{}'::jsonb);\n"
  types=','.join('text' for p in r['parameters']);args=','.join("'"+p['value'].replace("'","''")+"'" for p in r['parameters'])
  for i,check in enumerate(checks):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {r['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n';rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  counts=[]
  for i in range(len(checks)):assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
  assert any(counts) if corrupt else not any(counts),(t['bound'],case,counts)
  remaining=rows[len(counts)*2:]
  if corrupt:assert not remaining
  else:
   assert remaining[0]==['tags','address','note','id','part'];expected=[]
   for owner,id,part,note in owners[:t['bound']]:
    tags=dict(state='value',value=[] if case=='empty-sequence' else [False,True,False]);address=dict(state='absent') if owner==2 and case=='absent-address' else dict(state='value',value=trees[5]['logical']);note_value=dict(state='absent') if note is None or case=='absent-note' else dict(state='value',value=note)
    expected.append((tags,address,note_value,id,part))
   actual=[(json.loads(row[0]),json.loads(row[1]),json.loads(row[2]),row[3],row[4]) for row in remaining[1:]];assert actual==expected,(case,actual,expected)
  results.append(dict(bound=t['bound'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(server=server,transportSha256=hashlib.sha256((F/'original-multi-recursive-entity-public.json').read_bytes()).hexdigest(),scope='Synthetic original complete entity combining independently admitted native Boolean sequence and numeric structured roots plus scalars; no fresh embedding or production claim',cases=len(results),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-multi-recursive-entity-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} multi-recursive entity native cases passed')
