"""Native PREPARE witness for generated primitives, with independent expectations.

This harness uses trusted fixture values and SQL-literal escaping solely to invoke
SQL EXECUTE through psql. Production host protocol parameter binding is a separate
Truss execution gate, not qualified by SQL-level PREPARE.
"""
import json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
artifact=json.loads(subprocess.check_output([ROOT/'target/debug/examples/parameter_probe']))
assert artifact['sql'].count('$')==4
assert all(p['position']==i+1 for i,p in enumerate(artifact['parameters']))
values=[p['value'] for p in artifact['parameters']]
expected=['-2147483648','-32768','18446744073709551615',"a'; DROP SCHEMA public; --\\ 😀 "]
assert values==expected
literal=lambda value:"'"+value.replace("'","''")+"'"
sql="SET standard_conforming_strings=on;\nPREPARE weft_primitives(int,smallint,numeric,text) AS "+artifact['sql']+";\nEXECUTE weft_primitives("+','.join(literal(v) for v in values)+");\nDEALLOCATE weft_primitives;\n"
raw=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
import csv,io
rows=list(csv.reader(io.StringIO(raw)))
assert rows==[['odd."alias;--','key_num','exact_value','text_value'],expected],rows
OUT=ROOT/'target/b005';OUT.mkdir(exist_ok=True)
(OUT/'prepared-summary.json').write_text(json.dumps(dict(rows=rows,slots=len(values),sqlPrepare=True,protocolBindingQualified=False,qualifiedAdapter=False),ensure_ascii=False,indent=2)+'\n')
print('Native generated quoting and PREPARE checks passed; adapter/protocol qualification remains pending.')
