"""Captured property-independent owner count SQL over minimal native tables."""
import csv, hashlib, io, json, subprocess, sys
from pathlib import Path
capture, report_path = map(Path, sys.argv[1:3])
raw = capture.read_bytes(); emission = json.loads(raw)
assert emission['checks'] == []
assert emission['columns'][0]['representation']['decoder'] == 'exact-integer'
assert emission['columns'][0]['nullable'] is False
assert len(emission['parameters']) == 1
slot = emission['parameters'][0]
assert slot['position'] == 1 and slot['value'] == '-1'
assert slot['origin']['use'] == 'admitted-owner-scan'
results = []
for name, owned in [('empty',0),('two-owners',2),('duplicate-owner-bag',5)]:
    sql = 'BEGIN; CREATE TEMP TABLE object(id bigint,type_id int);\n'
    for i in range(owned): sql += f'INSERT INTO object VALUES ({987654321},-1);\n'
    sql += 'INSERT INTO object VALUES (1,-2);\n'
    sql += f"PREPARE query(int4) AS {emission['sql']}; EXECUTE query(-1); ROLLBACK;\n"
    output = subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows = list(csv.reader(io.StringIO(output)))
    assert rows == [['count'],[str(owned)]],(name,rows)
    results.append({'id':name,'count':owned,'executedSqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version = subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report_path.write_text(json.dumps({'scope':'Captured Rust original-definition V02 COUNT assembly over synthetic table without property columns; not complete public backend, host or adopted Truss qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('3 original-definition V02 COUNT/native cases passed.')
