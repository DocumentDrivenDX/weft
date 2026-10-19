# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual installed-wheel versus retained original Truss consumer gate.
No monkeypatch/imported consumer registry, policy decision or native admission.
"""
import argparse,hashlib,importlib,json,pathlib,sys,zipfile
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--expect',choices=['accepted','refused'],required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
sys.path.insert(0,str(a.truss/'packages/python/src'))
import weft
from truss import _current_query_originals as original
native=importlib.import_module('weft.weft');module=pathlib.Path(native.__file__);shim=pathlib.Path(weft.__file__)
def sha(data):return hashlib.sha256(data).hexdigest()
with zipfile.ZipFile(a.wheel)as z:
 names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1
 assert module.read_bytes()==z.read(names[0]) and shim.read_bytes()==z.read('weft/__init__.py')
source=pathlib.Path(original.__file__);before=source.read_bytes();checks=[]
assert sha(before)=="ae1998b6729cc4971cd7d375326b6e8a668864807706866e08cc57ca08e0fa43"
assert len(original._CONSUMER_COHORTS)==3
assert {c['weftRevision']for c in original._CONSUMER_COHORTS}=={'718a9649413baa4e3ede5c0c3b7ed24f4ae48d46','cc943a2f0ad87e3742cccdc171184bad7d4dbf60'}
retained=next(c for c in original._CONSUMER_COHORTS if c['weftWheelSha256']=='8ea8e06b7e8cf3623df69dd1e27c3bde3a1189f3adbd01c43f4a1519431b2b35')
for name,call in [('actual-module-cohort',original._actual_consumer),('retained-public33-selection',lambda:original._match_consumer(dict(retained))),('original-preparation-entry',lambda:original.prepare_current_query_originals(b'{}'))]:
 try:result=call()
 except ValueError as e:
  assert a.expect=='refused',(name,str(e));assert str(e)=='Actual selected original Weft ABI2 module required';checks.append({'id':name,'refused':True,'error':str(e)})
 else:
  assert a.expect=='accepted',name
  if name=='actual-module-cohort':assert result==retained
  checks.append({'id':name,'refused':False})
assert source.read_bytes()==before
report={'scope':'Actual installed public wheel against unchanged retained Truss original consumer selector before native execution; no source semantic or native security acceptance','weftVersion':weft.__version__,'wheel':a.wheel.name,'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(module.read_bytes()),'shimSha256':sha(shim.read_bytes()),'retainedGateSourceSha256':sha(before),'retainedCohorts':list(original._CONSUMER_COHORTS),'checks':checks,'nativeSecurityAcceptance':False,'casesPromoted':[]}
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'version':weft.__version__,'expected':a.expect,'controls':len(checks),'gateSourceSha256':report['retainedGateSourceSha256']}))
