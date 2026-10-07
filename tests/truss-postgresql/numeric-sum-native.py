import csv, hashlib, io, json, subprocess, sys
from pathlib import Path
capture, receipt = map(Path, sys.argv[1:3])
raw = capture.read_bytes(); results = []
for emission in json.loads(raw):
    samples = [('empty', [], '__null__')]
    samples += [('exact', ['18446744073709551615','18446744073709551615'], '36893488147419103230')] if emission['name']=='uint64' else [('exact',['9999999999999999999.123456789','-0.000000001'], '9999999999999999999.123456788')]
    for name, values, expected in samples:
        sql='BEGIN; CREATE TEMP TABLE values_table(value text);\n'
        for value in values: sql+=f"INSERT INTO values_table VALUES ('{value}');\n"
        sql+=emission['sql']+'; ROLLBACK;\n'
        output=subprocess.check_output(['docker','exec','-i','weft-b005-pg17','psql','-U','postgres','-X','-q','--csv','-P','null=__null__','-v','ON_ERROR_STOP=1'], input=sql.encode()).decode()
        assert list(csv.reader(io.StringIO(output)))==[['total'],[expected]], output
        results.append({'family':emission['name'],'case':name,'result':expected,'sqlSha256':hashlib.sha256(sql.encode()).hexdigest()})
receipt.write_text(json.dumps({'scope':'Selected comparator SUM SQL over synthetic exact text inputs; not full admitted property/SELECT or production support','captureSha256':hashlib.sha256(raw).hexdigest(),'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'results':results},indent=2)+'\n')
print('4 exact numeric SUM native cases passed.')
