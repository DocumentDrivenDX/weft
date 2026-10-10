"""Actual qualified native Python ABI parity, no subprocess or JS sidecar.
@covers US-005-AC1 @covers US-005-AC3
"""
import hashlib,json,os,subprocess,sys
import sys
if sys.flags.optimize:raise RuntimeError('Qualification requires nonoptimized Python')
from pathlib import Path
import weft
import weft.weft as extension
OUT=Path(os.environ.get('WEFT_QUALIFIED_HOST_OUT','/private/tmp/weft-b007-qualified-hosts'))
cases=[json.loads(l) for l in (OUT/'cases.jsonl').read_text().splitlines()];reports=json.loads((OUT/'cli-reports.json').read_text());assert len(cases)==len(reports)==2181
os.environ['PATH']=''
def forbidden(*a,**k):raise AssertionError('Subprocess during native compilation')
subprocess.Popen=subprocess.run=subprocess.check_output=forbidden
receipts=[]
for c,r in zip(cases,reports,strict=True):
 raw=json.dumps(c['request'],ensure_ascii=False);actual=weft.compile_json(raw);assert actual==r['raw'],c['id']
 receipts.append(dict(id=c['id'],requestSha256=hashlib.sha256(raw.encode()).hexdigest(),actualSha256=hashlib.sha256(actual.encode()).hexdigest(),expectedSha256=hashlib.sha256(r['raw'].encode()).hexdigest()))
for bad in [None,{},1,b'{}']:
 try:weft.compile_json(bad)
 except TypeError:pass
 else:raise AssertionError('Invalid transport type accepted')
try:weft.compile_json('\ud800')
except UnicodeError:pass
else:raise AssertionError('Invalid surrogate accepted')
summary=dict(status='passed',cases=2181,byteParity=True,subprocessDisabled=True,extensionSha256=hashlib.sha256(Path(extension.__file__).read_bytes()).hexdigest(),nativeModule=extension.__file__,version=weft.__version__,python=sys.version,scope='Actual loaded Python ABI with both qualified backends. Byte parity to fresh public Rust runtime on all native-qualified inputs; no Python database executions or broader platform claims.')
(OUT/'python-summary.json').write_text(json.dumps(summary,indent=2)+'\n');(OUT/'python-receipts.json').write_text(json.dumps(dict(summary=summary,cases=receipts),indent=2)+'\n');print(json.dumps(summary))
