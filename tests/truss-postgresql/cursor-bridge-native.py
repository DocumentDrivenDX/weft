"""Execute captured V02 named cursor predicate and all physical preflight checks."""
import csv, hashlib, io, json, subprocess, sys
from collections import Counter
from pathlib import Path
capture, report_path = map(Path, sys.argv[1:3])
raw = capture.read_bytes(); emission = json.loads(raw)
slots = emission['parameters']
assert slots[2]['origin']['parameter']=='selected_name'
assert slots[2]['value']=='A'
types = ','.join('int4' if p['origin'].get('use')=='admitted-owner-scan' else 'text' for p in slots)
quote = lambda value: "'" + value.replace("'", "''") + "'"
values = ','.join(quote(p['value']) for p in slots)
results = []
for case, names, corrupt in [('empty', [], False), ('duplicate-unicode-empty', ['A','A','','é ','é'], False), ('predicate-read-corruption', ['A'], True)]:
    sql = 'BEGIN; SET standard_conforming_strings=on; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
    for index,name in enumerate(names):
        sql += f'INSERT INTO object VALUES ({index+1},-1,{quote(json.dumps({"1":name},ensure_ascii=False))});\n'
    sql += "INSERT INTO object VALUES (100,-2,'[]');\n"
    if corrupt: sql += "INSERT INTO object VALUES (200,-1,'{\"1\":false}');\n"
    for index,check in enumerate(emission['checks']):
        sql += f'PREPARE check_{index}({types}) AS {check}; EXECUTE check_{index}({values});\n'
    if not corrupt: sql += f"PREPARE query({types}) AS {emission['sql']}; EXECUTE query({values});\n"
    sql += 'ROLLBACK;\n'
    output = subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows = list(csv.reader(io.StringIO(output)))
    counts = []
    for index in range(len(emission['checks'])):
        assert rows[index*2] == ['violations']; counts.append(int(rows[index*2+1][0]))
    assert (any(counts) if corrupt else not any(counts)), (case,counts)
    remaining = rows[len(counts)*2:]
    if not corrupt:
        assert remaining[0] == [column['outputName'] for column in emission['columns']]
        assert Counter(tuple(row if row else [""]) for row in remaining[1:]) == Counter((a,) for a in names if a>"A")
    else: assert not remaining
    results.append({'id':case,'violations':counts,'queryExecuted':not corrupt,'executedSqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version = subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report_path.write_text(json.dumps({'scope':'Captured original-profile V02 named cursor predicate and physical preflight over synthetic string JSONB rows; callback fixture only, no adopted codecs, native rows, recursive results or host qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('3 V02 named cursor predicate/native cases passed.')
