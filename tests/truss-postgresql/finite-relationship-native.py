"""Original admitted finite relationship cardinality in each direction."""
import csv,hashlib,io,json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];path=ROOT/'tests/truss-postgresql/fixtures/original-finite-relationship-checks.json';raw=path.read_bytes();captures=json.loads(raw);results=[]
for e in captures:
 for case in ['valid','forward-maximum','inverse-maximum','inverse-minimum']:
  sql='BEGIN; CREATE TEMP TABLE object(id bigint,type_id int); CREATE TEMP TABLE edge(rel_type_id int,source_id bigint,source_type int,target_id bigint,target_type int); INSERT INTO object VALUES (1,-1),(2,-1),(3,-1),(10,-2),(11,-2); INSERT INTO edge VALUES (0,1,-1,10,-2),(0,2,-1,10,-2),(0,3,-1,11,-2);\n'
  if case=='forward-maximum':sql+='INSERT INTO edge VALUES (0,1,-1,11,-2);\n'
  if case=='inverse-maximum':sql+='INSERT INTO object VALUES (4,-1); INSERT INTO edge VALUES (0,4,-1,10,-2);\n'
  if case=='inverse-minimum':sql+='INSERT INTO object VALUES (12,-2);\n'
  types=','.join('numeric' if p['logicalType']['facets']['integerWidth']['bits']==64 else 'int4' for p in e['parameters']);args=','.join("'"+p['value']+"'" for p in e['parameters'])
  for i,check in enumerate(e['checks']):sql+=f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({args});\n'
  sql+='ROLLBACK;\n'
  rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
  violation=(case in ['inverse-maximum','inverse-minimum']) if e['inverse'] else case=='forward-maximum'
  assert rows==[['violations'],['0'],['violations'],['1' if violation else '0']],(case,e['inverse'],rows)
  results.append(dict(case=case,inverse=e['inverse'],violations=[0,int(violation)],executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original admitted finite multiplicity prerequisites only; no standalone public finite query or embedding qualification',server=server,sourceMultiplicity=dict(min=1,max=2),targetMultiplicity=dict(min=0,max=1),captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-finite-relationship-native.json').write_text(json.dumps(receipt,indent=2)+'\n')
print('8 original finite relationship native cases passed')
