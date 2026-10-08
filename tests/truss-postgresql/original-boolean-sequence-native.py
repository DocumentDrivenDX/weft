"""Native typed recursive Boolean sequence proof with independent exact values.
@covers US-003-AC1: false/true/false sequence retains Boolean values/order/bag.
@covers US-003-AC3: complete-tree corrupt payload or presence blocks query.
"""
import copy,csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
path=F/'original-boolean-sequence-public.json';response=json.loads(path.read_text())['response'];tree=json.loads((F/'original-boolean-sequence-tree.json').read_text());binding=tree['binding'];prop=next(p for p in binding['properties'] if p['logical']['element']=='tags');results=[]
def cell(v,i):
 if v is None:return 'NULL'
 if i in [7,8,9,19,20,21,22]:return "decode('"+v+"','hex')"
 return "'"+v.replace("'","''")+"'"
checks=[o['parameters']['sql'] for o in response['obligations'] if 'sql' in o['parameters']]
for case in ['exact','empty','null-boolean','extra-text','wrong-kind','wrong-codec','wrong-source','absent-root','duplicate-ordinal']:
 corrupt=case not in ['exact','empty'];rows=copy.deepcopy(tree['cells'])
 if case=='empty':rows=rows[:1]
 if case=='null-boolean':rows[-1][14]=None
 if case=='extra-text':rows[-1][13]='false'
 if case=='wrong-kind':rows[-1][12]='text'
 if case=='wrong-codec':rows[-1][21]='00'
 if case=='wrong-source':rows[-1][9]='00'
 if case=='duplicate-ordinal':rows[-1][5]='0'
 sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);
INSERT INTO object VALUES (1,-1,'{}'),(2,-999,'{}');
'''
 if case!='absent-root':
  sql+=f"INSERT INTO row_home_state VALUES (1,'object',1,-1,NULL,NULL,-1,{prop['propertyId']},1);\n"
  for row in rows:
   sql+='INSERT INTO row_home_node VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[:10]))+');\n'
   if row[10] is not None:sql+='INSERT INTO row_home_scalar VALUES ('+','.join(cell(v,i) for i,v in enumerate(row[10:],10))+');\n'
 types=','.join('text' for p in response['parameters']);args=','.join("'"+p['value'].replace("'","''")+"'" for p in response['parameters'])
 for i,check in enumerate(checks):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
 if not corrupt:sql+=f"PREPARE q({types}) AS {response['sql']}; EXECUTE q({args});\n"
 sql+='ROLLBACK;\n';out=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
 counts=[]
 for i in range(len(checks)):assert out[i*2]==['violations'];counts.append(int(out[i*2+1][0]))
 assert any(counts) if corrupt else not any(counts),(case,counts)
 remaining=out[len(counts)*2:]
 if corrupt:assert not remaining
 else:
  assert remaining[0]==['tags'] and len(remaining)==2
  actual=json.loads(remaining[1][0]);expected=dict(state='value',value=[] if case=='empty' else [False,True,False]);assert actual==expected,(case,actual)
  assert all(type(v) is bool for v in actual['value'])
 results.append(dict(case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original native Boolean sequence over synthetic pinned fixture codec/source procedures; no runtime embedding or production claim',server=server,cases=len(results),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),captureSha256=hashlib.sha256(path.read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-boolean-sequence-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} original Boolean sequence native cases passed')
