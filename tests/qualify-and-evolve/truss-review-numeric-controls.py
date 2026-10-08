"""Semantic controls for exact registered Truss numeric-domain receipts.
@covers US-006-AC1 @covers US-006-AC3
"""
import csv,gzip,hashlib,importlib.util,io,json,os,shutil,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('truss_numeric_audit',HERE/'reconcile-truss-review-numerics.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
BASE=Path(os.environ.get('WEFT_TRUSS_NUMERIC_EVIDENCE',str(audit.BASE)))
assert audit.verify(BASE)['cases']==1124
controls=[]
def csv_edit(base,fn):
 p=base/'stdout.csv';rows=list(csv.reader(io.StringIO(p.read_text())));fn(rows);stream=io.StringIO();csv.writer(stream,lineterminator='\n').writerows(rows);p.write_text(stream.getvalue())
def artifacts_edit(base,fn):
 p=base/'compile-artifacts.jsonl.gz';rows=[json.loads(l) for l in gzip.decompress(p.read_bytes()).decode().splitlines()];fn(rows)
 with gzip.open(p,'wb') as f:f.write(''.join(json.dumps(r)+'\n' for r in rows).encode())
def test(name,mutate):
 with tempfile.TemporaryDirectory(prefix='weft-truss-numeric-control-') as tmp:
  base=Path(tmp)/'receipts';shutil.copytree(BASE,base);mutate(base)
  p=base/'custody.json';custody=json.loads(p.read_text());custody['files']={n:hashlib.sha256((base/n).read_bytes()).hexdigest() for n in custody['files']};p.write_text(json.dumps(custody))
  try:audit.verify(base)
  except (AssertionError,KeyError,IndexError,TypeError):controls.append(name)
  else:raise AssertionError('Accepted corrupted numeric receipt '+name)
test('exact sum changed',lambda b:csv_edit(b,lambda r:r[2].__setitem__(2,'0')))
test('valid guard rejects value',lambda b:csv_edit(b,lambda r:r[1].__setitem__(2,'1')))
test('invalid domain silently lost',lambda b:csv_edit(b,lambda r:r[3].__setitem__(2,'4')))
test('empty sum coerced to zero',lambda b:csv_edit(b,lambda r:r[4].__setitem__(2,'0')))
test('PostgreSQL release drift',lambda b:csv_edit(b,lambda r:r[0].__setitem__(1,json.dumps(dict(audit.ENGINE,version='17.10')))))
test('independent expectation weakened',lambda b:artifacts_edit(b,lambda r:r[0].__setitem__('expectedSum','0')))
test('decimal result scale changed',lambda b:artifacts_edit(b,lambda r:r[0]['response']['columns'][0]['logicalType']['facets'].__setitem__('scale',1)))
def sql_edit(base):
 p=base/'executed.sql.gz';text=gzip.decompress(p.read_bytes()).decode();assert ' AND ' in text
 with gzip.open(p,'wb') as f:f.write(text.replace(' AND ',' OR ',1).encode())
test('integrity AND became OR',sql_edit)
print(json.dumps(dict(status='passed',corruptionsRejected=len(controls),controls=controls,scope='Eight semantic corruptions rejected after recomputing archive custody hashes; no new native SQL.')))
