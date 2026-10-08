"""Actual native wheel parity with all subprocess entry points disabled."""
import json,os,subprocess,hashlib,sys
from pathlib import Path
import weft
import weft.weft as native_extension
ROOT=Path(__file__).resolve().parents[2]
cases=json.loads((ROOT/os.environ.get('WEFT_CORPUS','tests/truss-postgresql/fixtures/application-cases.json')).read_text())
reports=json.loads((ROOT/os.environ.get('WEFT_REPORTS','target/b005/application-native-reports.json')).read_text())
def forbidden(*args,**kwargs):raise AssertionError('Subprocess attempted during compile')
os.environ['PATH']=''
subprocess.Popen=subprocess.run=subprocess.check_output=forbidden
receipts=[]
for c,r in zip(cases,reports,strict=True):
    request=json.dumps(c['request'],ensure_ascii=False)
    actual=weft.compile_json(request)
    assert actual==r['raw'],c['id']
    receipts.append(dict(id=c['id'],requestSha256=hashlib.sha256(request.encode()).hexdigest(),actualSha256=hashlib.sha256(actual.encode()).hexdigest(),expectedSha256=hashlib.sha256(r['raw'].encode()).hexdigest()))
for wrong in [None,{},b'{}',1]:
    try:weft.compile_json(wrong)
    except TypeError:pass
    else:raise AssertionError('Non-string accepted')
try:weft.compile_json('\ud800')
except UnicodeError:pass
else:raise AssertionError('Lone surrogate accepted')
summary=dict(cases=len(cases),byteParity=True,subprocessDisabled=True,nativeModule=native_extension.__file__,version=weft.__version__,python=sys.version,extensionSha256=hashlib.sha256(Path(native_extension.__file__).read_bytes()).hexdigest())
(ROOT/os.environ.get('WEFT_SUMMARY','target/b005/python-summary.json')).write_text(json.dumps(summary,indent=2)+'\n')
if os.environ.get('WEFT_RECEIPTS'):
    (ROOT/os.environ['WEFT_RECEIPTS']).write_text(json.dumps(dict(summary=summary,cases=receipts),indent=2)+'\n')
print(json.dumps(summary))
