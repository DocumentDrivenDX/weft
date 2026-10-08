"""Public original composite-key SQL against independent exact tuple expectations.

@covers US-003-AC1: exact native rows/types/multiplicity for the selected subset.
@covers US-003-AC2: storage IDs and typed complete logical keys remain distinct.
@covers US-003-AC3: emitted guards observe corrupt selected stored meanings.
These subset claims do not close the full story or qualify host AC4.
"""
import csv,hashlib,io,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
mixed='--mixed' in sys.argv
heterogeneous='--heterogeneous' in sys.argv
strings='--string' in sys.argv
prefix='original-string' if strings else ('original-heterogeneous' if heterogeneous else 'original-composite')
path=F/(prefix+('-mixed-public.json' if mixed else '-relationship-public.json'));raw=path.read_bytes();captures=json.loads(raw)
inputs=json.loads((F/(prefix+'-relationship-inputs.json')).read_text());binding=json.loads(inputs['requests'][0]['request']['target']['bindingJson']);composition=inputs['composition'];results=[]
codec={int(binding['properties'][p['index']]['propertyId']):p['leafCodecs']['root']['originalJson'].encode().hex() for p in composition['properties']}
for e in captures:
 for case in (['decimal-excess-scale','decimal-precision-overflow'] if heterogeneous else [])+['empty','ordered-partial-duplicates','duplicate-customer-tuple','duplicate-order-tuple','hidden-third-overflow','missing-part','wrong-third-codec','dangling-endpoint']:
  corrupt=case not in ['empty','ordered-partial-duplicates']
  sql='''BEGIN;
CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);
CREATE TEMP TABLE edge(rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int);
CREATE TEMP TABLE row_home_state(state_id bigint,owner_kind text,object_id bigint,object_type_id int,edge_id bigint,relationship_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint);
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value bool,numeric_value numeric,numeric_token text,codec_definition_bytes bytea,original_source_bytes bytea,binary_value bytea,temporal_text text,temporal_instant timestamptz,opaque_bytes bytea);
'''
  if strings:
   sql=sql.replace('CREATE TEMP TABLE row_home_scalar',"CREATE COLLATION pg_temp.weft_adversarial (provider=icu,locale='und-u-ks-level1',deterministic=false); CREATE TEMP TABLE row_home_scalar").replace('text_value text,','text_value text COLLATE pg_temp.weft_adversarial,')
  customers=[(1,['7','18446744073709551615']),(2,['7','0']),(3,['7','9007199254740993']),(4,['6','18446744073709551615']),(5,['8','0'])]
  orders=[(10,['8','100','9']),(11,['8','101','8']),(12,['7','999','0']),(13,['8','100','10'])]
  if case=='duplicate-customer-tuple':customers[-1]=(5,['7','0'])
  if case=='duplicate-order-tuple':orders[-1]=(13,['8','100','9'])
  if case=='hidden-third-overflow':orders[-1]=(13,['8','100','18446744073709551616'])
  if heterogeneous:
   customers=[(owner,[{'7':'7.25','8':'8.50','6':'6.25'}[v[0]],v[1]]) for owner,v in customers]
   orders=[(owner,[{'7':'7.25','8':'8.50'}[v[0]],*v[1:]]) for owner,v in orders]
  if strings:
   customers=[(owner,[{'7':'é','8':'e\u0301','6':'Z'}[v[0]],v[1]]) for owner,v in customers]
   orders=[(owner,[{'7':'é','8':'e\u0301'}[v[0]],*v[1:]]) for owner,v in orders]
  if case=='decimal-excess-scale':customers[-1]=(5,['8.501','0'])
  if case=='decimal-precision-overflow':customers[-1]=(5,['100000000000000000000000000.00','0'])
  if case=='empty':customers=[]
  props_pids=e.get('propsPropertyIds',[])
  for type_id,owners,pids in [(-1,customers,[20,0]),(-2,orders,[21,6,22])]:
   for owner,values in owners:
    props={str(pid):(True if case=='wrong-third-codec' and owner==13 and pid==22 else value) for pid,value in zip(pids,values) if pid in props_pids and not (case=='missing-part' and owner==5 and pid==20)}
    sql+=f"INSERT INTO object VALUES ({owner},{type_id},'{json.dumps(props)}'::jsonb);\n"
    for pid,value in zip(pids,values):
     if pid in props_pids or (case=='missing-part' and owner==5 and pid==20):continue
     state=owner*100+pid;node=state+10000;code='00' if case=='wrong-third-codec' and owner==13 and pid==22 else codec[pid]
     family='string' if strings and pid in [20,21] else ('decimal' if heterogeneous and pid in [20,21] else 'integer')
     text="'"+value.replace("'","''")+"'" if family=='string' else 'NULL'
     numeric='NULL' if family=='string' else value
     token='NULL' if family=='string' else "'"+value+"'"
     sql+=f"INSERT INTO row_home_state VALUES ({state},'object',{owner},{type_id},NULL,NULL,{type_id},{pid},{node}); INSERT INTO row_home_node VALUES ({state},{node},NULL); INSERT INTO row_home_scalar(state_id,node_id,scalar_kind,text_value,numeric_value,numeric_token,codec_definition_bytes,original_source_bytes) VALUES ({state},{node},'{family}',{text},{numeric},{token},decode('{code}','hex'),decode('fe00','hex'));\n"
  if case!='empty':
   sql+='INSERT INTO edge VALUES (0,1,-1,10,-2),(0,2,-1,10,-2),(0,3,-1,10,-2),(0,3,-1,11,-2),(0,3,-1,11,-2),(0,4,-1,10,-2),(1,1,-1,11,-2);\n'
   if strings:sql+='INSERT INTO edge VALUES (0,5,-1,11,-2);\n'
   if case=='dangling-endpoint':sql+='INSERT INTO edge VALUES (0,999,-1,10,-2);\n'
  if strings:sql+="SELECT ('é'::text COLLATE pg_temp.weft_adversarial)=('e\u0301'::text COLLATE pg_temp.weft_adversarial) AS adversarial_equal;\n"
  args=','.join("'"+p['value']+"'" for p in e['parameters']);types=','.join('numeric' if p['logicalType']['family']=='decimal' or p['logicalType']['facets'].get('integerWidth',{}).get('bits')==64 else ('text' if p['logicalType']['family']=='string' else 'int4') for p in e['parameters'])
  for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  if not corrupt:sql+=f"PREPARE q({types}) AS {e['sql']}; EXECUTE q({args});\n"
  sql+='ROLLBACK;\n'
  rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  if strings:
   assert rows[:2]==[['adversarial_equal'],['t']],rows[:2]
   rows=rows[2:]
  counts=[]
  for i in range(len(e['checks'])):
   assert rows[2*i]==['violations'];counts.append(int(rows[2*i+1][0]))
  assert any(counts) if corrupt else not any(counts),(e['direction'],e['kind'],case,counts)
  remaining=rows[len(counts)*2:]
  if not corrupt:
   if e['kind']=='has':assert remaining==[['n'],['0' if case=='empty' else ('4' if e['direction']=='forward' else '2')]],remaining
   else:
    parsed=[(r[:-1],json.loads(r[-1])) for r in remaining[1:]]
    empty=dict(items=[],truncated=False);single=dict(items=[['8','100','9']],truncated=False)
    if e['direction']=='forward':expected=[] if case=='empty' else [(['6','18446744073709551615'],single),(['7','0'],single),(['7','9007199254740993'],dict(items=[['8','100','9'],['8','101','8']],truncated=True)),(['7','18446744073709551615'],single),(['8','0'],empty)]
    else:expected=[(['7','999','0'],empty),(['8','100','9'],empty if case=='empty' else dict(items=[['6','18446744073709551615'],['7','0']],truncated=True)),(['8','100','10'],empty),(['8','101','8'],empty if case=='empty' else dict(items=[['7','9007199254740993'],['7','9007199254740993']],truncated=False))]
    if heterogeneous:
     expected=[([{'7':'7.25','8':'8.50','6':'6.25'}[key[0]],*key[1:]],dict(items=[[{'7':'7.25','8':'8.50','6':'6.25'}[item[0]],*item[1:]] for item in related['items']],truncated=related['truncated'])) for key,related in expected]
    if strings:
     expected=[([{'7':'é','8':'e\u0301','6':'Z'}[key[0]],*key[1:]],dict(items=[[{'7':'é','8':'e\u0301','6':'Z'}[item[0]],*item[1:]] for item in related['items']],truncated=related['truncated'])) for key,related in expected]
     if case!='empty':
      if e['direction']=='forward':expected=[(key,dict(items=[['e\u0301','101','8']],truncated=False) if key==['e\u0301','0'] else related) for key,related in expected]
      else:expected=[(key,dict(items=[['e\u0301','0'],['é','9007199254740993']],truncated=True) if key==['e\u0301','101','8'] else related) for key,related in expected]
     expected.sort(key=lambda pair:(pair[0][0].encode('utf-8'),*[int(v) for v in pair[0][1:]]))
    if e['kind']=='cursor':
     boundary=['7','0'] if e['direction']=='forward' else ['8','100','9']
     if heterogeneous:boundary[0]='7.25' if e['direction']=='forward' else '8.50'
     if strings:boundary[0]='é' if e['direction']=='forward' else 'e\u0301'
     from decimal import Decimal
     def ordered(key):return (key[0].encode('utf-8') if strings else Decimal(key[0]),*[int(v) for v in key[1:]])
     expected=[pair for pair in expected if ordered(pair[0])>ordered(boundary)]
    assert parsed==expected,parsed
  else:assert not remaining
  results.append(dict(propsPropertyIds=props_pids,direction=e['direction'],kind=e['kind'],case=case,violations=counts,queryExecuted=not corrupt,executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(nativeTextColumnCollation='ICU und-u-ks-level1 nondeterministic' if strings else 'database default',scope=('Public original owned composite UTF8 C-collation string/uint64 relationship keys' if strings else ('Public original owned composite decimal(28,2)/uint64 relationship keys' if heterogeneous else 'Public original owned composite uint64 relationship keys'))+', source arity 2 and target arity 3, independently selected native-row/JSONB synthetic PostgreSQL fixtures; not embedding or production qualification',server=server,captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence'/(('B-005-string' if strings else ('B-005-heterogeneous' if heterogeneous else 'B-005-composite'))+('-mixed-native.json' if mixed else '-relationship-native.json'))).write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} original composite relationship native cases passed')
