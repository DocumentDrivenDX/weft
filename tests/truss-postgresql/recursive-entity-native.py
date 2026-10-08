"""Complete original scalar/recursive entities with independent native expectations.
@covers US-003-AC1: declared member order and exact recursive/scalar values.
@covers US-003-AC2: original property homes and discriminated owner scans.
@covers US-003-AC3: owner-wide prerequisite refusal independent of page bound.
"""
import csv,hashlib,io,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures';path=F/'original-recursive-entity-public.json';raw=path.read_bytes();captures=json.loads(raw);inputs=json.loads((F/'original-recursive-entity-inputs.json').read_text());results=[];driver_setups=[]
def cell(value,index):
 if value is None:return 'NULL'
 if index in [7,8,9,19,20,21,22]:return "decode('"+value+"','hex')"
 return "'"+value.replace("'","''")+"'"
for e in captures:
 input=next(x for x in inputs if x['fixture']==e['fixture']);binding=json.loads(input['requests'][0]['request']['target']['bindingJson']);codecs={int(binding['properties'][p['index']]['propertyId']):p['leafCodecs']['root']['originalJson'].encode().hex() for p in input['composition']['properties'] if not p.get('nativeTree',True)}
 tree=json.loads((F/f"original-{e['fixture']}-native-tree.json").read_text())
 for case in ['valid','hidden-compound-corruption','hidden-key-corruption','hidden-note-corruption','foreign-owner','absent-compound']:
  corrupt=case.startswith('hidden-') or (case=='absent-compound' and e['fixture'] not in ['address','cyclic','numeric-address'])
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);
'''
  owners=[(1,'9007199254740993','A',None),(2,'18446744073709551615','B','')]
  for owner,id,part,note in owners:
   props={} if note is None else {'23':note}
   if case=='hidden-note-corruption' and owner==2 and e['noteHome']=='props':props={'23':17}
   sql+=f"INSERT INTO object VALUES ({owner},-1,'{json.dumps(props)}'::jsonb);\n"
   for pid,family,value in [(0,'integer',id),(20,'string',part)]+([(23,'string',note)] if note is not None and e['noteHome']=='row' else []):
    state=owner*100+pid;node=state+10000;code=codecs[pid]
    if owner==2 and ((case=='hidden-key-corruption' and pid==0) or (case=='hidden-note-corruption' and pid==23)):code='00'
    text="'"+value.replace("'","''")+"'" if family=='string' else 'NULL';numeric=value if family=='integer' else 'NULL';token="'"+value+"'" if family=='integer' else 'NULL'
    sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},-1,NULL,NULL,-1,{pid},{node}); INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind) VALUES ({state},{node},NULL,'scalar'); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'{family}',{text},{numeric},{token},decode('{code}','hex'),decode('fe00','hex'));\n"
   if case=='absent-compound' and owner==2:continue
   sql+=f"INSERT INTO row_home_state VALUES ({owner},'object',{owner},-1,NULL,NULL,-1,{e['propertyId']},{1+(owner-1)*100});\n"
   rows=[list(r) for r in tree['cells']]
   for row in rows:
    row[0]=str(owner);row[1]=str(int(row[1])+(owner-1)*100)
    if row[2] is not None:row[2]=str(int(row[2])+(owner-1)*100)
    if row[10] is not None:row[10]=str(owner);row[11]=str(int(row[11])+(owner-1)*100)
   if case=='hidden-compound-corruption' and owner==2:rows[-1][9]='ff'
   for row in rows:
    sql+='INSERT INTO row_home_node VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[:10]))+');\n'
    if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[10:],10))+');\n'
  if case=='foreign-owner':
   sql+=f"INSERT INTO object VALUES (1,-2,'{{}}'::jsonb); INSERT INTO row_home_state VALUES (3,'object',1,-2,NULL,NULL,-1,{e['propertyId']},201);\n"
   foreign=[list(r) for r in tree['cells']]
   for row in foreign:
    row[0]='3';row[1]=str(int(row[1])+200)
    if row[2] is not None:row[2]=str(int(row[2])+200)
    if row[10] is not None:row[10]='3';row[11]=str(int(row[11])+200)
   foreign[-1][9]='ff'
   for row in foreign:
    sql+='INSERT INTO row_home_node VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[:10]))+');\n'
    if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[10:],10))+');\n'
  driver_setups.append(dict(fixture=e['fixture'],noteHome=e['noteHome'],bound=e['bound'],case=case,corrupt=corrupt,setupSql=sql.removeprefix('BEGIN;\n'),capture=e))
  args=','.join("'"+p['value'].replace("'","''")+"'" for p in e['parameters']);types=','.join('text' for p in e['parameters'])
  for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {e['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n'
  rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  counts=[]
  for i in range(len(e['checks'])):
   assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
  assert any(counts) if corrupt else not any(counts),(e['fixture'],e['noteHome'],e['bound'],case,counts)
  remaining=rows[len(counts)*2:]
  if not corrupt:
   assert remaining[0]==['address' if input['root']=='address' else 'tags','note','id','part'],remaining[0]
   expected=[]
   for owner,id,part,note in owners[:e['bound']]:
    compound=dict(state='absent') if case=='absent-compound' and owner==2 else dict(state='value',value=tree['logical'])
    expected.append((compound,dict(state='absent') if note is None else dict(state='value',value=note),id,part))
   parsed=[(json.loads(r[0]),json.loads(r[1]),r[2],r[3]) for r in remaining[1:]];assert parsed==expected,(e['fixture'],case,parsed)
  else:assert not remaining
  results.append(dict(fixture=e['fixture'],noteHome=e['noteHome'],bound=e['bound'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Complete original scalar/recursive entities over seven selected graph shapes with mixed optional scalar homes; synthetic PostgreSQL fixtures, not new embedding or production qualification',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-recursive-entity-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} original recursive entity native cases passed')

if os.environ.get("WEFT_ENTITY_DRIVER_SETUPS"):
 Path(os.environ["WEFT_ENTITY_DRIVER_SETUPS"]).write_text(json.dumps(driver_setups)+"\n")
