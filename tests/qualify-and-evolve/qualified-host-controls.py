"""Semantic controls for final actual qualified host receipt joins.
@covers US-006-AC1 @covers US-006-AC3
"""
import gzip,hashlib,importlib.util,json,os,shutil,tempfile
from pathlib import Path
HERE=Path(__file__).resolve().parent;spec=importlib.util.spec_from_file_location('qualified_host_audit',HERE/'audit-qualified-hosts.py');audit=importlib.util.module_from_spec(spec);spec.loader.exec_module(audit)
BASE=Path(os.environ.get('WEFT_QUALIFIED_HOST_EVIDENCE',str(audit.BASE)));assert audit.verify(BASE)['hostArtifactJoins']==4362
controls=[]
def edit(base,name,fn,lines=False):
 p=base/name;compressed=False
 if not p.exists():p=base/(name+'.gz');compressed=True
 raw=gzip.decompress(p.read_bytes()).decode() if compressed else p.read_text()
 data=[json.loads(l) for l in raw.splitlines()] if lines else json.loads(raw);fn(data)
 new=''.join(json.dumps(r)+'\n' for r in data) if lines else json.dumps(data)+'\n'
 if compressed:
  with gzip.open(p,'wb') as f:f.write(new.encode())
 else:p.write_text(new)
def test(name,fn):
 with tempfile.TemporaryDirectory(prefix='weft-qualified-host-control-') as tmp:
  base=Path(tmp)/'receipts';shutil.copytree(BASE,base);fn(base)
  if (base/'custody.json').exists():
   p=base/'custody.json';c=json.loads(p.read_text());c['files']={n:hashlib.sha256((base/n).read_bytes()).hexdigest() for n in c['files']};p.write_text(json.dumps(c))
  try:audit.verify(base)
  except (AssertionError,KeyError,IndexError,TypeError):controls.append(name)
  else:raise AssertionError('Accepted host corruption '+name)
test('Python actual hash changed',lambda b:edit(b,'python-receipts.json',lambda r:r['cases'][0].__setitem__('actualSha256','0'*64)))
test('browser request identity changed',lambda b:edit(b,'browser-receipts.json',lambda r:r['cases'][0].__setitem__('requestSha256','0'*64)))
test('duplicate Python case identity',lambda b:edit(b,'python-receipts.json',lambda r:r['cases'][0].__setitem__('id',r['cases'][1]['id'])))
test('qualified request reverted to candidate',lambda b:edit(b,'cases.jsonl',lambda r:r[0]['request']['target'].__setitem__('backendVersion','0.1.0-candidate'),True))
def stale_output(base):
 def mutate(rows):
  r=json.loads(rows[0]['raw']);r['sql']+=' ';rows[0]['raw']=json.dumps(r)
 edit(base,'cli-reports.json',mutate)
test('emitted SQL changed',stale_output)
test('package native payload changed',lambda b:edit(b,'package-summary.json',lambda r:r.__setitem__('nativeExtensionSha256','0'*64)))
test('resource refusal input identity changed',lambda b:edit(b,'resource-cases.json',lambda r:r[0].__setitem__('raw','{}')))
test('browser runtime drift',lambda b:edit(b,'browser-summary.json',lambda r:r.__setitem__('browser','newer')))
print(json.dumps(dict(status='passed',corruptionsRejected=len(controls),controls=controls,scope='Eight semantic host receipt corruptions rejected after custody rehashing; no rebuilding or database execution.')))
