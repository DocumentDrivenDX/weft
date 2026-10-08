"""Native exact integer operand boundary proof; not full storage/domain admission.
Uses captured renderer SQL and independently calculated mathematical bounds.
"""
import csv,hashlib,io,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
path=Path(os.environ['WEFT_INTEGER_OPERAND_CAPTURE']);raw=path.read_bytes();captures=json.loads(raw);results=[]
assert len(captures)==256
for index,case in enumerate(captures):
 bits=case['bits'];signed=case['signed'];offset=index%2
 expected=str((-(2**(bits-1)) if signed else 0) if offset==0 else (2**(bits-1)-1 if signed else 2**bits-1))
 assert case['value']==expected
 sql=f"PREPARE q(text) AS SELECT ({case['sql']})::pg_catalog.text AS exact_value; EXECUTE q('{case['value']}');"
 rows=list(csv.reader(io.StringIO(subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode())))
 assert rows==[['exact_value'],[expected]],(case,rows)
 results.append(dict(bits=bits,signed=signed,boundary='minimum' if offset==0 else 'maximum',expected=expected,actual=rows[1][0],executedSqlSha256=hashlib.sha256(sql.encode()).hexdigest()))
server=subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
receipt=dict(scope='Conformance original scalar renderer integer literals; signed/unsigned widths 1..64 exact native numeric transport, not new storage codec or production/domain admission qualification',server=server,cases=len(results),captureSha256=hashlib.sha256(raw).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-integer-operand-native.json').write_text(json.dumps(receipt,indent=2)+'\n');print(f'{len(results)} exact integer operand native cases passed')
