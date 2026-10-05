"""Assert authored public outcomes and independently execute fixture SQL."""
import json,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
cases=json.loads((ROOT/'tests/compile/fixtures/cases.json').read_text())
values=json.loads((ROOT/'tests/register-backend/fixtures/rows.json').read_text())
db=sqlite3.connect(':memory:')
for table in ['fixture_customers','fixture_customers_v2']:
    db.execute(f'CREATE TABLE {table} (display_name TEXT NOT NULL,renamed_name TEXT NOT NULL)')
    db.executemany(f'INSERT INTO {table} VALUES (?,?)',[(v,v) for v in values])
reports=[];executions=0
for c in cases:
    raw=subprocess.check_output([ROOT/'target/debug/weft-runtime'],input=json.dumps(c['request'],ensure_ascii=False).encode()).decode().strip()
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
OUT=ROOT/'target/b004';OUT.mkdir(exist_ok=True)
(OUT/'reports.json').write_text(json.dumps(reports,ensure_ascii=False))
print(json.dumps(dict(cases=len(cases),sqlExecutions=executions,sqlite=sqlite3.sqlite_version)))
