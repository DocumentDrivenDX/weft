"""Assert authored public outcomes and independently execute fixture SQL."""
import json,sqlite3,subprocess,os,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BINARY=Path(os.environ.get('WEFT_COMPILE_BINARY',str(ROOT/'target/debug/weft-runtime')))
BINARY_SHA=hashlib.sha256(BINARY.read_bytes()).hexdigest()
cases=json.loads((ROOT/'tests/compile/fixtures/cases.json').read_text())
values=json.loads((ROOT/'tests/register-backend/fixtures/rows.json').read_text())
db=sqlite3.connect(':memory:')
for table in ['fixture_customers','fixture_customers_v2']:
    db.execute(f'CREATE TABLE {table} (display_name TEXT NOT NULL,renamed_name TEXT NOT NULL)')
    db.executemany(f'INSERT INTO {table} VALUES (?,?)',[(v,v) for v in values])
reports=[];executions=0
for c in cases:
    raw=subprocess.check_output([BINARY],input=json.dumps(c['request'],ensure_ascii=False).encode()).decode().strip()
    response=json.loads(raw)
    assert response['status']==c['expected']['status'],(c['id'],response)
    if response['status']=='blocked':
        assert response['diagnostics'][0]['code']==c['expected']['code'],(c['id'],response)
        assert 'sql' not in response and 'parameters' not in response
    else:
        assert response['parameters']==[]
        rows=[list(r) for r in db.execute(response['sql']).fetchall()]
        assert rows==c['expected']['rows'],(c['id'],rows)
        executions+=1
    reports.append(dict(id=c['id'],raw=raw,response=response))
OUT=Path(os.environ.get('WEFT_COMPILE_OUTPUT',str(ROOT/'target/b004')));OUT.mkdir(parents=True,exist_ok=True)
(OUT/'reports.json').write_text(json.dumps(reports,ensure_ascii=False))
assert hashlib.sha256(BINARY.read_bytes()).hexdigest()==BINARY_SHA
summary=dict(cases=len(cases),sqlExecutions=executions,sqlite=sqlite3.sqlite_version,binarySha256=BINARY_SHA,corpusSha256=hashlib.sha256((ROOT/'tests/compile/fixtures/cases.json').read_bytes()).hexdigest(),harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),scope='Public fixture backend corpus; SQLite executes only fixture SQL, not Truss or Ashlar SQL.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary))
