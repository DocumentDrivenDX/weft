"""Exhaustive scalar numeric domains through the exact Truss NativeReview registration.
@covers US-003-AC1 @covers US-003-AC3 @covers US-006-AC1
Temporary tables, one PostgreSQL repeatable-read transaction, rollback only.
"""
import base64,copy,csv,gzip,hashlib,io,json,os,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];OUT=Path(os.environ.get('WEFT_TRUSS_NUMERIC_OUT','/private/tmp/weft-b007-truss-review-numerics'));OUT.mkdir(parents=True,exist_ok=True)
BINARY=Path(os.environ.get('WEFT_TRUSS_NUMERIC_BINARY','/private/tmp/weft-b007-parser-target/debug/examples/compile_native_review'));BINARY_SHA=hashlib.sha256(BINARY.read_bytes()).hexdigest()
sha=lambda b:hashlib.sha256(b).hexdigest()
def snapshot(value):
 raw=json.dumps(value,separators=(',',':')).encode();digest=sha(raw)
 return dict(identity='candidate-fixture-'+digest,bytesBase64=base64.b64encode(raw).decode(),sha256=digest)
def fixed(coefficient,scale):
 sign='-' if coefficient<0 else '';digits=str(abs(coefficient)).zfill(scale+1)
 return sign+(digits[:-scale]+'.'+digits[-scale:] if scale else digits)
base=json.loads((ROOT/'tests/truss-postgresql/fixtures/compiler-cases.json').read_text());base={c['home']:c for c in base if c['id'].startswith('sales-join-')}
domains=[dict(family='decimal',precision=p,scale=s) for p in range(1,29) for s in range(p+1)]+[dict(family='integer',bits=b,signed=s) for s in [False,True] for b in range(1,65)]
cases=[]
for home in ['props','row']:
 for domain in domains:
  request=copy.deepcopy(base[home]['request']);doc=json.loads(request['modules'][0]['documentJson']);element=next(e for e in doc['modules'][0]['elements'] if e['id']=='order-total');element['scalarType']=domain['family'];element['facets']={k:domain[k] for k in ['precision','scale']} if domain['family']=='decimal' else {'integerWidth':{k:domain[k] for k in ['bits','signed']}}
  raw=json.dumps(doc);request['modules'][0]['documentJson']=raw;request['modules'][0]['pin']['sha256']=sha(raw.encode())
  binding=json.loads(request['target']['bindingJson']);binding['basis']['modelBundle']=snapshot(request['modules'])
  for section in ['entities','properties']:
   for entry in binding[section]:
    entry['source']=snapshot(doc);entry['acceptedDefinition']=snapshot(next(e for e in doc['modules'][0]['elements'] if e['id']==entry['logical']['element']))
  raw=json.dumps(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=sha(raw.encode());request['target']['backendVersion']='0.1.0-native-review';request['target']['targetProfile']='pg17.9-native-review';request['sql']='SELECT SUM(o.total) AS total FROM Orders o'
  if domain['family']=='decimal':
   p,s=domain['precision'],domain['scale'];maximum=fixed(10**p-1,s);unit=fixed(1,s);overflow=fixed(10**p,s);overScale=fixed(1,s+1)
   valid=['-'+maximum,maximum,'-'+maximum,maximum,unit];invalid=[overflow,'-'+overflow,overScale,None,'ABSENT'];expected=unit;id=f'{home}-decimal-{p}-{s}'
  else:
   b,s=domain['bits'],domain['signed'];minimum=-(1<<(b-1)) if s else 0;maximum=(1<<(b-1))-1 if s else (1<<b)-1
   valid=[str(minimum),str(maximum),str(minimum),str(maximum)];invalid=[str(minimum-1),str(maximum+1),'0.5',None,'ABSENT'];expected=str(2*(minimum+maximum));id=f'{home}-integer-{b}-{s}'
  cases.append(dict(id=id,home=home,domain=domain,request=request,valid=valid,invalid=invalid,expectedSum=expected))
assert len(cases)==1124
requests=''.join(json.dumps(c['request'])+'\n' for c in cases)
run=subprocess.run([str(BINARY)],input=requests,text=True,capture_output=True);assert run.returncode==0,run.stderr
responses=[json.loads(l) for l in run.stdout.splitlines()];assert len(responses)==1124
for c,r in zip(cases,responses):
 assert r['status']=='compiled',(c['id'],r);assert r['qualification']['status']=='candidate';c['response']=r
with gzip.open(OUT/'compile-artifacts.jsonl.gz','wb') as f:f.write(''.join(json.dumps(c)+'\n' for c in cases).encode())
if '--prepare-only' in sys.argv:print(json.dumps(dict(status='prepared',cases=1124,compilerSha256=BINARY_SHA)));sys.exit(0)
quote=lambda s:"'"+s.replace("'","''")+"'"
setup='''BEGIN ISOLATION LEVEL REPEATABLE READ;
SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint PRIMARY KEY,type_id int NOT NULL,props jsonb NOT NULL);
CREATE TEMP TABLE row_home_state(state_id bigint PRIMARY KEY,owner_kind text,object_id bigint,object_type_id int,property_owner_type_id int,property_id int,root_node_id bigint);
CREATE TEMP TABLE row_home_node(state_id bigint,node_id bigint,parent_node_id bigint,value_kind text,slot_kind text,sequence_ordinal bigint,map_key text COLLATE "C",record_field_identity_bytes bytea,PRIMARY KEY(state_id,node_id));
CREATE TEMP TABLE row_home_scalar(state_id bigint,node_id bigint,scalar_kind text,text_value text,boolean_value boolean,numeric_value numeric,PRIMARY KEY(state_id,node_id));
SELECT 'engine',json_build_object('version',current_setting('server_version'),'server_encoding',current_setting('server_encoding'),'client_encoding',current_setting('client_encoding'),'standard_conforming_strings',current_setting('standard_conforming_strings'),'transaction_isolation',current_setting('transaction_isolation'),'lc_collate',(SELECT datcollate FROM pg_database WHERE datname=current_database()),'lc_ctype',(SELECT datctype FROM pg_database WHERE datname=current_database()))::text;
'''
def seed(c,values):
 sql='TRUNCATE object,row_home_state,row_home_node,row_home_scalar;\n'
 for i,v in enumerate(values,1):
  props='{}' if v=='ABSENT' else '{"4":'+('null' if v is None else v)+'}'
  sql+=f"INSERT INTO object VALUES ({i},-2,{quote(props)});\n"
  if v=='ABSENT':continue
  numeric='NULL' if v is None else quote(v)
  sql+=f"INSERT INTO row_home_state VALUES ({i},'object',{i},-2,-2,4,{i});\nINSERT INTO row_home_node VALUES ({i},{i},NULL,'scalar','root',NULL,NULL,NULL);\nINSERT INTO row_home_scalar VALUES ({i},{i},{quote(c['domain']['family'])},NULL,NULL,{numeric});\n"
 return sql
parts=[setup]
for c in cases:
 r=c['response'];checks=[g for o in r['obligations'] if o['id']=='truss.candidate.scalarIntegrity' for g in o['parameters']['checks']];assert len(checks)==1
 types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in r['parameters']);values=','.join(quote(p['value']) for p in r['parameters'])
 sql=f"PREPARE guard({types}) AS SELECT {quote(c['id'])},'guard',g.* FROM ({checks[0]['sql']}) g;\nPREPARE query({types}) AS SELECT {quote(c['id'])},'sum',q.* FROM ({r['sql']}) q;\n"
 sql+=seed(c,c['valid'])+f'EXECUTE guard({values});\nEXECUTE query({values});\n'
 sql+=seed(c,c['invalid'])+f'EXECUTE guard({values});\n'
 sql+=seed(c,[])+f'EXECUTE query({values});\nDEALLOCATE guard;\nDEALLOCATE query;\n'
 parts.append(sql)
parts.append('ROLLBACK;\n');sql=''.join(parts)
with gzip.open(OUT/'executed.sql.gz','wb') as f:f.write(sql.encode())
run=subprocess.run(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-t','-P','null=__WEFT_NULL__','-v','ON_ERROR_STOP=1'],input=sql,text=True,capture_output=True)
(OUT/'stdout.csv').write_text(run.stdout);(OUT/'stderr.log').write_text(run.stderr);assert run.returncode==0,run.stderr
rows=list(csv.reader(io.StringIO(run.stdout)));assert len(rows)==4497 and rows[0][0]=='engine',len(rows)
engine=json.loads(rows[0][1]);assert engine=={'version':'17.9 (Debian 17.9-1.pgdg13+1)','server_encoding':'UTF8','client_encoding':'UTF8','standard_conforming_strings':'on','transaction_isolation':'repeatable read','lc_collate':'C','lc_ctype':'C'},engine
for i,c in enumerate(cases):
 actual=rows[1+4*i:5+4*i];expected=[[c['id'],'guard','0'],[c['id'],'sum',c['expectedSum']],[c['id'],'guard','5'],[c['id'],'sum','__WEFT_NULL__']];assert actual==expected,(c['id'],actual,expected)
assert sha(BINARY.read_bytes())==BINARY_SHA
summary=dict(status='passed',cases=1124,decimalDomainsPerHome=434,integerDomainsPerHome=128,homes=['props','row'],nativeAssertions=4496,postgresqlTransactions=1,engine=engine,compilerSha256=BINARY_SHA,harnessSha256=sha(Path(__file__).read_bytes()),scope='Exact Truss NativeReview scalar SUM, valid/invalid numeric-domain guards and empty SUM for all admitted precision/scale and signed/unsigned width domains in JSONB and typed numeric rows. One same-view PostgreSQL rollback transaction; no promotion or stored-table custody.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
