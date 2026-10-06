"""Execute captured Rust admitted-owner source assembly; component evidence only."""
import csv, hashlib, io, json, subprocess, sys
from collections import Counter
from pathlib import Path
capture = Path(sys.argv[1])
report_path = Path(sys.argv[2])
raw = capture.read_bytes()
emission = json.loads(raw)
assert emission['namespace'] == 'pg_temp'
slots = emission['parameters']
assert [p['position'] for p in slots] == [1, 2, 3, 4]
assert [p['origin']['use'] for p in slots] == ['admitted-owner-scan', 'admitted-jsonb-member'] * 2
assert slots[0]['value'] == slots[2]['value'] == '-1'
assert slots[1]['value'] == slots[3]['value'] == '1'
quote = lambda value: "'" + value.replace("'", "''") + "'"
values = ','.join(quote(p['value']) for p in slots)
results = []
for case, names in [('empty', []), ('duplicate-unicode-empty', ['A', 'A', '', 'é ', 'é'])]:
    sql = '''BEGIN;
SET standard_conforming_strings=on;
CREATE TEMP TABLE object(id bigint, type_id int, props jsonb);
'''
    for index, name in enumerate(names):
        props = json.dumps({'1': name}, ensure_ascii=False)
        sql += f'INSERT INTO object VALUES ({index+1}, -1, {quote(props)});\n'
    # Wrong owner types must not participate, including malformed unrelated roots.
    sql += "INSERT INTO object VALUES (100,-2,'{\"1\":\"A\"}'),(101,-2,'[]');\n"
    sql += f"PREPARE query(int4,text,int4,text) AS {emission['sql']};\nEXECUTE query({values});\nROLLBACK;\n"
    output = subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'], input=sql.encode()).decode()
    rows = list(csv.reader(io.StringIO(output)))
    assert rows[0] == ['left_name','right_name'], rows
    expected = Counter((left,right) for left in names for right in names if left == right)
    assert Counter(map(tuple, rows[1:])) == expected, (case, rows, expected)
    results.append({'id':case,'rows':rows[1:],'rowCount':len(rows)-1,'executedSqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version = subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report = {'scope':'Captured Rust original-profile source assembly over minimal synthetic owner rows; not native codec, integrity enforcement, result ABI, Truss adoption or deployment qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results}
report_path.write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
print('2 captured Rust owner-scan/native cases passed (empty and seven-row duplicate/Unicode bag).')
