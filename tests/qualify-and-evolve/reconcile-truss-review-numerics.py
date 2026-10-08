"""Independent coefficient arithmetic and exact native-registration receipt audit.
@covers US-003-AC1 @covers US-003-AC3 @covers US-006-AC1
"""
import base64,csv,gzip,hashlib,io,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=Path(os.environ.get('WEFT_TRUSS_NUMERIC_EVIDENCE',str(ROOT/'docs/helix/04-build/evidence/B-007-truss-review-numerics')))
ENGINE={'version':'17.9 (Debian 17.9-1.pgdg13+1)','server_encoding':'UTF8','client_encoding':'UTF8','standard_conforming_strings':'on','transaction_isolation':'repeatable read','lc_collate':'C','lc_ctype':'C'}
def decimal(n,scale):
 sign='-' if n<0 else '';digits=str(abs(n))
 if scale:
  digits=digits.rjust(scale+1,'0');digits=digits[:-scale]+'.'+digits[-scale:]
 return sign+digits
quote=lambda value:"'"+value.replace("'","''")+"'"
def fixture(case,values):
 statements=['TRUNCATE object,row_home_state,row_home_node,row_home_scalar;\n']
 for index,value in enumerate(values,1):
  document='{}' if value=='ABSENT' else '{"4":'+('null' if value is None else value)+'}'
  statements.append(f"INSERT INTO object VALUES ({index},-2,{quote(document)});\n")
  if value!='ABSENT':
   cell='NULL' if value is None else quote(value)
   statements.extend([f"INSERT INTO row_home_state VALUES ({index},'object',{index},-2,-2,4,{index});\n",f"INSERT INTO row_home_node VALUES ({index},{index},NULL,'scalar','root',NULL,NULL,NULL);\n",f"INSERT INTO row_home_scalar VALUES ({index},{index},{quote(case['domain']['family'])},NULL,NULL,{cell});\n"])
 return ''.join(statements)
def verify(base):
 custody=json.loads((base/'custody.json').read_text())
 for name,digest in custody['files'].items():assert hashlib.sha256((base/name).read_bytes()).hexdigest()==digest,name
 summary=json.loads((base/'summary.json').read_text());assert summary['status']=='passed' and summary['cases']==1124 and summary['engine']==ENGINE
 assert hashlib.sha256((base/'harness.py').read_bytes()).hexdigest()==summary['harnessSha256']
 cases=[json.loads(l) for l in gzip.decompress((base/'compile-artifacts.jsonl.gz').read_bytes()).decode().splitlines()];assert len(cases)==len({c['id'] for c in cases})==1124
 sql=gzip.decompress((base/'executed.sql.gz').read_bytes()).decode();rows=list(csv.reader(io.StringIO((base/'stdout.csv').read_text())))
 assert len(rows)==4497 and rows[0][0]=='engine' and json.loads(rows[0][1])==ENGINE
 assert sql.startswith('BEGIN ISOLATION LEVEL REPEATABLE READ;\nSET standard_conforming_strings=on;\n') and sql.endswith('ROLLBACK;\n')
 assert sql.count('BEGIN ISOLATION LEVEL REPEATABLE READ;')==sql.count('ROLLBACK;')==1
 seen=set()
 for i,c in enumerate(cases):
  d=c['domain'];home=c['home'];assert home in ['props','row']
  if d['family']=='decimal':
   p,s=d['precision'],d['scale'];assert 1<=p<=28 and 0<=s<=p;seen.add((home,'decimal',p,s))
   limit=10**p;valid=[decimal(n,s) for n in [-limit+1,limit-1,-limit+1,limit-1,1]];invalid=[decimal(limit,s),decimal(-limit,s),decimal(1,s+1),None,'ABSENT'];expected=decimal(1,s);facets={'precision':p,'scale':s}
  else:
   bits,signed=d['bits'],d['signed'];assert d['family']=='integer' and 1<=bits<=64 and isinstance(signed,bool);seen.add((home,'integer',bits,signed))
   low=-(2**(bits-1)) if signed else 0;high=2**(bits-1)-1 if signed else 2**bits-1
   valid=list(map(str,[low,high,low,high]));invalid=[str(low-1),str(high+1),'0.5',None,'ABSENT'];expected=str(sum([low,high,low,high]));facets={'integerWidth':{'bits':bits,'signed':signed}}
  assert c['valid']==valid and c['invalid']==invalid and c['expectedSum']==expected
  request=c['request'];response=c['response'];assert request['sql']=='SELECT SUM(o.total) AS total FROM Orders o'
  assert request['target']['backendVersion']=='0.1.0-native-review' and request['target']['targetProfile']=='pg17.9-native-review'
  for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
  module=request['modules'][0];doc=json.loads(module['documentJson']);element=next(e for e in doc['modules'][0]['elements'] if e['id']=='order-total');assert element['scalarType']==d['family'] and element['facets']==facets
  raw=request['target']['bindingJson'];assert hashlib.sha256(raw.encode()).hexdigest()==request['target']['bindingSha256'];binding=json.loads(raw)
  snapshot=binding['basis']['modelBundle'];assert json.loads(base64.b64decode(snapshot['bytesBase64']))==request['modules']
  selected=next(p for p in binding['properties'] if p['logical']['element']=='order-total')
  assert selected['home']==home
  for name,expectedSource in [('source',doc),('acceptedDefinition',element)]:
   snap=selected[name];content=base64.b64decode(snap['bytesBase64']);assert hashlib.sha256(content).hexdigest()==snap['sha256'] and json.loads(content)==expectedSource
  assert response['columns']==[{'aggregate':'sum','carrier':'text','decoder':'exact-decimal' if d['family']=='decimal' else 'exact-integer','logicalType':{'facets':{'scale':d['scale']} if d['family']=='decimal' else {},'family':d['family'],'nullable':True},'nullable':True,'outputName':'total','position':1,'sourceIdentities':[selected['logical']]}]
  assert response['status']=='compiled' and response['qualification']['status']=='candidate'
  assert response['modelPins']==[m['pin'] for m in request['modules']] and response['bindingSha256']==request['target']['bindingSha256']
  checks=[g for o in response['obligations'] if o['id']=='truss.candidate.scalarIntegrity' for g in o['parameters']['checks']];assert len(checks)==1
  assert any(o['id']=='truss.nativeProfile' for o in response['obligations'])
  types=','.join({'string':'text','boolean':'bool','integer':'numeric','decimal':'numeric'}[p['logicalType']['family']] for p in response['parameters']);values=','.join(quote(p['value']) for p in response['parameters'])
  block=f"PREPARE guard({types}) AS SELECT {quote(c['id'])},'guard',g.* FROM ({checks[0]['sql']}) g;\nPREPARE query({types}) AS SELECT {quote(c['id'])},'sum',q.* FROM ({response['sql']}) q;\n"
  block+=fixture(c,valid)+f'EXECUTE guard({values});\nEXECUTE query({values});\n'+fixture(c,invalid)+f'EXECUTE guard({values});\n'+fixture(c,[])+f'EXECUTE query({values});\nDEALLOCATE guard;\nDEALLOCATE query;\n'
  assert sql.count(block)==1,c['id']
  expectedRows=[[c['id'],'guard','0'],[c['id'],'sum',expected],[c['id'],'guard','5'],[c['id'],'sum','__WEFT_NULL__']]
  assert rows[1+4*i:5+4*i]==expectedRows,c['id']
 expectedSet={(h,'decimal',p,s) for h in ['props','row'] for p in range(1,29) for s in range(p+1)}|{(h,'integer',b,s) for h in ['props','row'] for b in range(1,65) for s in [False,True]}
 assert seen==expectedSet and len(seen)==1124
 return dict(status='passed',cases=1124,nativeAssertions=4496,postgresqlTransactions=1,scope='Exact Truss registration, all 434 decimal and 128 integer domains per props/typed home, independent integer coefficient expectations, complete executed SQL/parameter/fixture/guard/pin correspondence. No support promotion.')
if __name__=='__main__':print(json.dumps(verify(BASE)))
