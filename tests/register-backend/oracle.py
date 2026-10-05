"""Host-only SQLite execution of emitted fixture SQL, with independently authored rows.

SQLite is an independent execution witness for this simple text/projection subset;
this does not qualify a production SQLite, Truss or Ashlar backend.
"""
import json,os,sqlite3,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
cases=json.loads((ROOT/'tests/register-backend/fixtures/cases.json').read_text())
values=json.loads((ROOT/'tests/register-backend/fixtures/rows.json').read_text())
connection=sqlite3.connect(':memory:')
for table in ['fixture_customers','fixture_customers_v2']:
    connection.execute(f'CREATE TABLE {table} (display_name TEXT NOT NULL,renamed_name TEXT NOT NULL)')
    connection.executemany(f'INSERT INTO {table} VALUES (?,?)',[(v,v) for v in values])
reports=[];executions=0
for c in cases:
    raw=subprocess.check_output([ROOT/'target/debug/weft-backend-probe'],input=json.dumps(c['request'],ensure_ascii=False).encode()).decode().strip()
    response=json.loads(raw);assert response['status']==c['expected']['status'],(c['id'],response)
    if response['status']=='blocked':
        assert response['diagnostics'][0]['code']==c['expected']['code'],(c['id'],response)
        assert 'compilation' not in response
    else:
        emission=response['compilation']['emission'];assert emission['parameters']==[]
        cursor=connection.execute(emission['sql']);rows=[list(r) for r in cursor.fetchall()]
        assert rows==c['expected']['rows'],(c['id'],rows,c['expected']['rows'])
        assert [d[0] for d in cursor.description]==[c['outputName'] for c in emission['columns']]
        assert response['retainedModules']==c['request']['modules']
        executions+=1
    reports.append(dict(id=c['id'],raw=raw,response=response))
# Injection labels and stored strings have not executed another statement.
assert connection.execute('SELECT COUNT(*) FROM fixture_customers').fetchone()[0]==len(values)
OUT=ROOT/'target/b003';OUT.mkdir(exist_ok=True)
(OUT/'reports.json').write_text(json.dumps(reports,ensure_ascii=False))
summary=dict(cases=len(cases),sqlExecutions=executions,sqlite=sqlite3.sqlite_version,rowsPerExecution=len(values),bagDuplicates=True,exactUnicode=True,physicalHomeVariants=True,productionQualification=False)
(OUT/'oracle-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print('Independent fixture SQL execution:',json.dumps(summary))
