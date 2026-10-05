"""Actual native wheel parity with all subprocess entry points disabled."""
import json,os,subprocess
from pathlib import Path
import weft
ROOT=Path(__file__).resolve().parents[2]
cases=json.loads((ROOT/'tests/compile/fixtures/cases.json').read_text())
reports=json.loads((ROOT/'target/b004/reports.json').read_text())
def forbidden(*args,**kwargs):raise AssertionError('Subprocess attempted during compile')
os.environ['PATH']=''
subprocess.Popen=subprocess.run=subprocess.check_output=forbidden
for c,r in zip(cases,reports,strict=True):
    assert weft.compile_json(json.dumps(c['request'],ensure_ascii=False))==r['raw'],c['id']
for wrong in [None,{},b'{}',1]:
    try:weft.compile_json(wrong)
    except TypeError:pass
    else:raise AssertionError('Non-string accepted')
try:weft.compile_json('\ud800')
except UnicodeError:pass
else:raise AssertionError('Lone surrogate accepted')
summary=dict(cases=len(cases),byteParity=True,subprocessDisabled=True,nativeModule=weft.__file__,version=weft.__version__)
(ROOT/'target/b004/python-summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary))
