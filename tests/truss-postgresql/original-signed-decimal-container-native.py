"""Native typed recursive numeric map/structured proof with independent exact values.
@covers US-003-AC1: signed/decimal map keys and structured member presence retains exact numeric strings/order/bag.
@covers US-003-AC3: complete-tree corrupt payload or presence blocks query.
"""
import copy,csv,hashlib,io,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
results=[]
for name in ['signed-map','decimal-map','signed-structured','decimal-structured']:
 label,shape=name.split('-',1)
 path=F/f'original-{name}-public.json';response=json.loads(path.read_text())['response'];tree=json.loads((F/f'original-{name}-tree.json').read_text());binding=tree['binding'];prop=next(p for p in binding['properties'] if p['logical']['element']==('tags' if shape=='map' else 'address'))
 def cell(v,i):
  if v is None:return 'NULL'
  if i in [7,8,9,19,20,21,22]:return "decode('"+v+"','hex')"
  return "'"+v.replace("'","''")+"'"
 checks=[o['parameters']['sql'] for o in response['obligations'] if 'sql' in o['parameters']]
 for case in ['exact','empty','zero','missing-token','malformed-token','null-number','extra-text','wrong-kind','wrong-codec','wrong-source','absent-root','duplicate-slot','payload-disagrees','above-domain','below-domain','excess-scale','nonfinite']:
  corrupt=case not in ['exact','empty','zero'] and not (case=='absent-root' and shape=='structured');rows=copy.deepcopy(tree['cells'])
  if case=='empty':
   rows=rows[:1] if shape=='map' else rows[:2]
   if shape=='structured':rows[1][13]=''
  if case=='zero':
   for row in rows[1:]:
    if row[12] in ['integer','decimal']:row[15]=row[16]='0' if label=='signed' else '0.00'
  if case=='missing-token':rows[-1][16]=None
  if case=='malformed-token':rows[-1][16]='not-a-number'
  if case=='null-number':rows[-1][15]=None
  if case=='extra-text':rows[-1][13]='extra'
  if case=='wrong-kind':rows[-1][12]='text'
  if case=='wrong-codec':rows[-1][21]='00'
  if case=='wrong-source':rows[-1][9]='00'
  if case=='duplicate-slot':
   duplicate=copy.deepcopy(rows[1] if shape=='map' else rows[-1]);duplicate[1]=duplicate[11]='99';rows.append(duplicate)
  if case=='payload-disagrees':rows[-1][15]='0'
  if case in ['above-domain','below-domain','excess-scale','nonfinite']:
   token={'above-domain':'9223372036854775808','below-domain':'-9223372036854775809','excess-scale':'0.5','nonfinite':'NaN'}[case] if label=='signed' else {'above-domain':'100000000000000000000000000.00','below-domain':'-100000000000000000000000000.00','excess-scale':'0.001','nonfinite':'NaN'}[case]
   rows[-1][15]=rows[-1][16]=token
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
   assert remaining[0]==['tags' if shape=='map' else 'address'] and len(remaining)==2
   actual=json.loads(remaining[1][0]);logical=copy.deepcopy(tree['logical'])
   if case=='empty':logical={} if shape=='map' else dict(street='',zip=dict(state='absent'))
   if case=='zero':
    if shape=='map':logical={k:('0' if label=='signed' else '0.00') for k in logical}
    else:logical['zip']['value']='0' if label=='signed' else '0.00'
   expected=dict(state='absent') if case=='absent-root' else dict(state='value',value=logical)
   assert actual==expected,(name,case,actual,expected)
   if 'value' in actual:
    numbers=list(actual['value'].values()) if shape=='map' else ([actual['value']['zip']['value']] if actual['value']['zip']['state']=='value' else [])
    assert all(type(v) is str for v in numbers)
  results.append(dict(fixture=name,case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original native numeric map/structured over synthetic pinned fixture codec/source procedures; no runtime embedding or production claim',server=server,cases=len(results),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),captureSha256={name:hashlib.sha256((F/f'original-{name}-public.json').read_bytes()).hexdigest() for name in sorted({r['fixture'] for r in results})},results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-numeric-container-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} original numeric map/structured native cases passed')
