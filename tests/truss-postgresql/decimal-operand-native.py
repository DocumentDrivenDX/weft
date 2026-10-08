"""Native exact decimal operand transport, independent mathematical bounds.
This does not qualify new property storage codecs or production profiles.
"""
import csv,hashlib,io,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];path=Path(os.environ['WEFT_DECIMAL_OPERAND_CAPTURE']);raw=path.read_bytes();cases=json.loads(raw);assert len(cases)==868
statements=[];expected=[]
for index,case in enumerate(cases):
 p=case['precision'];s=case['scale'];whole=10**p-1;integer,remainder=divmod(whole,10**s)
 value=str(integer)+(('.'+str(remainder).rjust(s,'0')) if s else '')
 if index%2:value='-'+value
 assert case['value']==value
 statements.append(f"PREPARE q_{index}(text) AS SELECT ({case['sql']})::pg_catalog.text AS exact_value; EXECUTE q_{index}('{value}');")
 expected.append(value)
sql='\n'.join(statements);out=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode();rows=list(csv.reader(io.StringIO(out)));assert len(rows)==len(cases)*2
results=[]
for i,case in enumerate(cases):
 assert rows[i*2]==['exact_value'] and rows[i*2+1]==[expected[i]],(case,rows[i*2+1])
 results.append(dict(precision=case['precision'],scale=case['scale'],boundary='minimum' if i%2 else 'maximum',expected=expected[i],actual=rows[i*2+1][0]))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Original conformance decimal literal renderer for precisions 1..28 and scales 0..precision; exact native numeric text, not new storage/domain/production qualification',server=server,cases=len(results),captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-decimal-operand-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} exact decimal operand native cases passed')
