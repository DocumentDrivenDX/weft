"""Native proof of the original sales join across all selected property homes.
@covers US-003-AC1: independent decimal sums and grouped join multiplicity.
@covers US-003-AC2: exact four-field mapping/type filters and storage IDs.
@covers US-003-AC3: hidden selected-field corruption prevents execution.
Synthetic pinned candidate layout, not installed Truss or production policy.
"""
import base64,copy,csv,decimal,hashlib,io,json,os,subprocess
from pathlib import Path
import weft
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures';source=F/'original-sales-join-inputs.json';cuts=json.loads(source.read_text());results=[];transports=[];driver_setups=[]
decimal.getcontext().prec=100
quote=lambda v:"'"+str(v).replace("'","''")+"'"
maximum='99999999999999999999999999.99';uintmax='18446744073709551615'
for cut in cuts:
 request=cut['request'];configuration=json.dumps(cut['configuration']);raw=weft.compile_json_with_conformance_configuration(json.dumps(request),configuration);assert raw==weft.compile_json_with_conformance_configuration(json.dumps(request),configuration)
 r=json.loads(raw);assert r['status']=='compiled',r
 transports.append(dict(homes=cut['homes'],response=r));checks=[o['parameters']['sql'] for o in r['obligations'] if 'sql' in o['parameters']]
 binding=json.loads(request['target']['bindingJson']);codecs={p['index']:p['leafCodecs']['root']['originalJson'].encode().hex() for p in cut['configuration']['properties']}
 for case in ['exact','empty','missing-hidden-name','wrong-name-carrier','wrong-total-carrier','hidden-total-domain','hidden-foreign-key-domain','hidden-total-scale']:
  corrupt=case not in ['exact','empty'];customers=[] if case=='empty' else [('9007199254740993','A'),('9007199254740994','A'),(uintmax,'A '),('17','é'),('19','e\u0301'),('21','unmatched-left')]
  orders=[] if case=='empty' else [('9007199254740993',maximum),('9007199254740993',maximum),('9007199254740994','-0.01'),(uintmax,'0.00'),('17','1.20'),('19','-1.20'),('23',maximum)]
  sql='''BEGIN; SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text,record_field_identity_bytes bytea,definition_bytes bytea,source_bytes bytea);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,temporal_text text,temporal_instant timestamptz,binary_value bytea,opaque_bytes bytea,codec_definition_bytes bytea,original_source_bytes bytea);
'''
  objects=[(100+i,-1,[(0,'integer',key),(1,'string',name)]) for i,(key,name) in enumerate(customers)]+[(200+i,-2,[(7,'integer',key),(8,'decimal',total)]) for i,(key,total) in enumerate(orders)]
  for owner,type_id,values in objects:
   props={};row_sql=''
   for pid,family,value in values:
    missing=case=='missing-hidden-name' and owner==105 and pid==1
    wrong=(case=='wrong-name-carrier' and owner==105 and pid==1) or (case=='wrong-total-carrier' and owner==206 and pid==8)
    if missing:continue
    if case=='hidden-total-domain' and owner==206 and pid==8:value='100000000000000000000000000.00'
    if case=='hidden-total-scale' and owner==206 and pid==8:value='0.001'
    if case=='hidden-foreign-key-domain' and owner==206 and pid==7:value='18446744073709551616'
    if binding['properties'][pid]['home']=='props':
     props[str(pid)]=False if wrong else value
     continue
    state=owner*100+pid;node=state+10000;codec=codecs[pid];text=quote(value) if family=='string' else 'NULL';numeric=quote(value)+'::numeric' if family!='string' else 'NULL';token=quote(value) if family!='string' else 'NULL';kind='boolean' if wrong else family
    row_sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},{type_id},NULL,NULL,{type_id},{pid},{node}); INSERT INTO row_home_node(state_id,node_id,parent_node_id,value_kind) VALUES ({state},{node},NULL,'scalar'); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'{kind}',{text},{numeric},{token},decode('{codec}','hex'),decode('fe00','hex'));\n"
   sql+=f"INSERT INTO object VALUES ({owner},{type_id},{quote(json.dumps(props,ensure_ascii=False))}::jsonb);\n"+row_sql
  # Unrelated type with matching property identifiers and invalid carrier values.
  sql+="INSERT INTO object VALUES (999,-999,'{\"0\":false,\"1\":\"A\",\"7\":false,\"8\":false}'::jsonb);\n"
  setup_sql=sql.removeprefix('BEGIN; ')
  types=','.join('text' for _ in r['parameters']);args=','.join(quote(p['value']) for p in r['parameters'])
  for i,check in enumerate(checks):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {r['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n'
  out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));counts=[]
  for i in range(len(checks)):assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
  assert any(counts) if corrupt else not any(counts),(cut['homes'],case,counts)
  remaining=rows[len(counts)*2:]
  if corrupt:assert not remaining
  else:
   assert remaining[0]==['name','total'];expected={}
   for key,name in customers:
    for foreign,total in orders:
     if int(key)==int(foreign):expected[name]=expected.get(name,decimal.Decimal(0))+decimal.Decimal(total)
   actual={name:decimal.Decimal(total) for name,total in remaining[1:]};assert len(actual)==len(remaining)-1;assert actual==expected,(cut['homes'],case,actual,expected)
  if os.environ.get('WEFT_SALES_DRIVER_SETUPS'):
   driver_setups.append(dict(homes=cut['homes'],case=case,corrupt=corrupt,setupSql=setup_sql,artifact=r,expectedRows=[] if corrupt else [[name,format(total.quantize(decimal.Decimal('0.01')),'f')] for name,total in expected.items()]))
  results.append(dict(homes=cut['homes'],case=case,violations=counts,queryExecuted=not corrupt,sqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
(F/'original-sales-join-public.json').write_text(json.dumps(transports,indent=2)+'\n')
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original Customer name / Orders total join, UInt64 equality, Unicode C groups, decimal(28,2), all 16 row/props home cuts; synthetic candidate only',cases=len(results),configurations=len(cuts),server=server,binaryProvenance='Fresh B-005-sales-join Python extension; exact binary hash pinned',extensionSha256=hashlib.sha256(next(Path(weft.__file__).parent.glob('*.so')).read_bytes()).hexdigest(),sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),transportSha256=hashlib.sha256((F/'original-sales-join-public.json').read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-original-sales-join-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} original sales join native cases passed')

if os.environ.get('WEFT_SALES_DRIVER_SETUPS'):Path(os.environ['WEFT_SALES_DRIVER_SETUPS']).write_text(json.dumps(driver_setups)+'\n')
