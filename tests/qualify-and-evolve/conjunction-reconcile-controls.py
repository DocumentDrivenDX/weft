"""Semantic corruption controls for retained native conjunction evidence.
@covers US-006-AC1 @covers US-006-AC3
"""
import copy,hashlib,importlib.util,json,os,shutil,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('conjunction_reconcile',HERE/'reconcile-conjunction-native.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
BASE=Path(os.environ.get('WEFT_CONJUNCTION_EVIDENCE',str(audit.BASE)))
assert audit.verify(BASE)['cases']==32
controls=[]
def edit_json(base,name,fn):
 p=base/name;data=json.loads(p.read_text());fn(data);p.write_text(json.dumps(data))
def edit_lines(base,name,fn):
 p=base/name;data=[json.loads(l) for l in p.read_text().splitlines()];fn(data);p.write_text(''.join(json.dumps(x)+'\n' for x in data))
def test(name,mutate):
 with tempfile.TemporaryDirectory(prefix='weft-conjunction-control-') as tmp:
  base=Path(tmp)/'receipts';shutil.copytree(BASE,base);mutate(base)
  custody=json.loads((base/'custody.json').read_text());custody['files']={name:hashlib.sha256((base/name).read_bytes()).hexdigest() for name in custody['files']};(base/'custody.json').write_text(json.dumps(custody))
  try:audit.verify(base)
  except (AssertionError,KeyError,IndexError,TypeError):controls.append(name)
  else:raise AssertionError('Accepted corruption '+name)
def native_query(rows):return next(r for r in rows if r['label'].endswith('True-hit-query'))
test('duplicate bag collapsed',lambda b:edit_lines(b,'ashlar/statements.jsonl',lambda rows:native_query(rows)['response']['result'].__setitem__('data_array',native_query(rows)['response']['result']['data_array'][:1])))
test('AND became OR',lambda b:edit_lines(b,'ashlar/statements.jsonl',lambda rows:native_query(rows).__setitem__('sql',native_query(rows)['sql'].replace(' AND ',' OR '))))
test('query parameter changed',lambda b:edit_lines(b,'ashlar/statements.jsonl',lambda rows:native_query(rows)['parameters'][-1].__setitem__('value','miss')))
test('integrity count nonzero',lambda b:edit_lines(b,'ashlar/statements.jsonl',lambda rows:next(r for r in rows if '-guard-' in r['label'])['response']['result'].__setitem__('data_array',[['1']])))
test('warehouse identity drift',lambda b:edit_lines(b,'ashlar/statements.jsonl',lambda rows:native_query(rows)['response']['result']['data_array'][0].__setitem__(0,json.dumps(dict(audit.ENGINE,dbsql_version='newer')))))
test('PostgreSQL transaction settings drift',lambda b:edit_json(b,'postgresql-receipts.json',lambda rows:rows[0]['profile'].__setitem__('transaction_isolation','read committed')))
test('expected result weakened',lambda b:edit_lines(b,'compile-artifacts.jsonl',lambda rows:rows[0].__setitem__('expectedRows',[])))
test('duplicate terminal receipt',lambda b:edit_lines(b,'ashlar/statements.jsonl',lambda rows:rows[0]['response'].__setitem__('statement_id',rows[1]['response']['statement_id'])))
print(json.dumps(dict(status='passed',corruptionsRejected=len(controls),controls=controls,scope='Eight semantic receipt corruptions rejected by exact native conjunction reconciliation; no new SQL executions.')))
