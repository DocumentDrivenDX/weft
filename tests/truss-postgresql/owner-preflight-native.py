"""Captured owner-wide structural checks, independent of result join membership."""
import csv, hashlib, io, json, subprocess, sys
from collections import Counter
from pathlib import Path
capture, report_path = map(Path, sys.argv[1:3])
raw = capture.read_bytes(); emission = json.loads(raw)
assert len(emission['structuralChecks']) == 2
assert emission['namespace'] == 'pg_temp'
quote = lambda s: "'" + s.replace("'", "''") + "'"
values = ','.join(quote(p['value']) for p in emission['parameters'])
for p in emission['parameters']:
    assert p['origin']['use'] in ('admitted-owner-scan','admitted-jsonb-member')
    assert p['logicalType']['family'] == ('integer' if p['origin']['use'] == 'admitted-owner-scan' else 'string')
types = ','.join('int4' if p['origin']['use'] == 'admitted-owner-scan' else 'text' for p in emission['parameters'])
results = []
cases = [('empty',[],None), ('valid-bag',['A','A','','é ','é'],None)]
cases += [(name,[],root) for name,root in [('array','\'[]\''),('json-null','\'null\''),('number','\'42\''),('sql-null','NULL')]]
for name, names, invalid in cases:
    sql = 'BEGIN; SET standard_conforming_strings=on; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
    for i,text in enumerate(names):
        sql += f'INSERT INTO object VALUES ({i+1},-1,{quote(json.dumps({"1":text},ensure_ascii=False))});\n'
    sql += "INSERT INTO object VALUES (100,-2,'[]');\n"
    if invalid is not None: sql += f'INSERT INTO object VALUES (200,-1,{invalid});\n'
    for i,check in enumerate(emission['structuralChecks']):
        sql += f'PREPARE check_{i}({types}) AS {check}; EXECUTE check_{i}({values});\n'
    if invalid is None:
        sql += f"PREPARE query({types}) AS {emission['sql']}; EXECUTE query({values});\n"
    sql += 'ROLLBACK;\n'
    output = subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows = list(csv.reader(io.StringIO(output)))
    count = '0' if invalid is None else '1'
    assert rows[:4] == [['violations'],[count],['violations'],[count]], (name,rows)
    if invalid is None:
        assert rows[4] == ['left_name','right_name']
        if 'columns' in emission:
            assert [c['position'] for c in emission['columns']] == [1,2]
            assert rows[4] == [c['outputName'] for c in emission['columns']]
            assert all(c['representation']['kind'] == 'scalar' and c['representation']['carrier'] == 'text' and c['representation']['decoder'] == 'text' for c in emission['columns'])
        expected = Counter((left,right) for left in names for right in names if left == right)
        assert Counter(map(tuple,rows[5:])) == expected,(name,rows)
    else: assert len(rows) == 4
    results.append({'id':name,'violationsPerOccurrence':int(count),'queryExecuted':invalid is None,'queryRows':rows[5:] if invalid is None else [],'columnMetadataChecked':invalid is None and 'columns' in emission,'executedSqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version = subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report_path.write_text(json.dumps({'scope':'Captured Rust owner-wide structural check SQL over synthetic props roots; fixture withholding on corruption only, not general host, codec, stored-domain or adopted Truss qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},ensure_ascii=False,indent=2)+'\n')
print('6 owner-wide native cases passed; all four malformed owned roots detected before result query.')
