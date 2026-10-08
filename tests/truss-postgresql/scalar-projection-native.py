"""Execute captured scalar projection and complete-owner physical payload check."""
import csv, hashlib, io, json, subprocess, sys
from collections import Counter
from pathlib import Path
capture, report_path = map(Path, sys.argv[1:3])
raw = capture.read_bytes(); emission = json.loads(raw)
slots = emission['parameters']
assert all(p['origin']['use'] in ('admitted-owner-scan','admitted-jsonb-member') for p in slots)
types = ','.join('int4' if p['origin']['use']=='admitted-owner-scan' else 'text' for p in slots)
quote = lambda s: "'"+s.replace("'","''")+"'"
values = ','.join(quote(p['value']) for p in slots)
results = []
valid = ['', 'é ', 'é', 'A', 'A']
cases = [('empty', [], 0), ('valid-strings', [json.dumps({'1':v},ensure_ascii=False) for v in valid], 0)]
cases += [(name,[root],1) for name,root in [('absent','{}'),('json-null','{"1":null}'),('boolean','{"1":false}'),('number','{"1":42}'),('array','{"1":[]}'),('object','{"1":{}}'),('bad-root','[]')]]
for name, roots, violations in cases:
    sql = 'BEGIN; SET standard_conforming_strings=on; CREATE TEMP TABLE object(id bigint,type_id int,props jsonb);\n'
    for i,root in enumerate(roots): sql += f'INSERT INTO object VALUES ({i+1},-1,{quote(root)});\n'
    sql += "INSERT INTO object VALUES (100,-2,'{\"1\":false}');\n"
    sql += f"PREPARE preflight({types}) AS {emission['check']}; EXECUTE preflight({values});\n"
    if violations==0: sql += f"PREPARE query({types}) AS {emission['sql']}; EXECUTE query({values});\n"
    sql += 'ROLLBACK;\n'
    output = subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-v','ON_ERROR_STOP=1'],input=sql.encode()).decode()
    rows = list(csv.reader(io.StringIO(output)))
    assert rows[:2] == [['violations'],[str(violations)]], (name,rows)
    if violations==0:
        assert rows[2] == [emission['column']['outputName']]
        actual = [row[0] if row else '' for row in rows[3:]]
        assert Counter(actual)==Counter(valid if name=='valid-strings' else []),(name,actual)
    else: assert len(rows)==2
    results.append({'id':name,'violations':violations,'queryExecuted':violations==0,'executedSqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
version = subprocess.check_output(['docker','exec','weft-b005-pg17','psql','-U','postgres','-X','-Atc','SELECT version()']).decode().strip()
report_path.write_text(json.dumps({'scope':'Rust scalar string projection and owner-wide physical JSONB payload check over synthetic rows; no source-domain, general decoder, host or adopted Truss qualification','server':version,'captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('9 scalar projection/native cases passed, including seven physical payload refusals.')
