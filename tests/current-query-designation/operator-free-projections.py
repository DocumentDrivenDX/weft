# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual retained Weft producer inputs for the original read-projection bridge.
No UMF response, read decision, native admission or result is fabricated here.
"""
import argparse,hashlib,importlib,json,pathlib,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
fixture=pathlib.Path(__file__).resolve().parent/'fixtures/mixed-original-uses.request.json';original=fixture.read_bytes();source=json.loads(original)
def sha(v):return hashlib.sha256(v).hexdigest()
native=pathlib.Path(importlib.import_module('weft.weft').__file__);shim=pathlib.Path(weft.__file__)
with zipfile.ZipFile(a.wheel)as z:
 modules=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(modules)==1
 assert native.read_bytes()==z.read(modules[0]) and shim.read_bytes()==z.read('weft/__init__.py')
checks=[]
for name,sql,field in [
 ('plain-salary','SELECT r.salary FROM Resource r','salary'),
 ('false-salary','SELECT r.salary FROM Resource r WHERE FALSE','salary'),
 ('limit-zero-salary','SELECT r.salary FROM Resource r LIMIT 0','salary'),
 ('stored-token','SELECT r.token FROM Resource r','token'),
]:
 request={**source,'sql':sql,'parameters':{}};raw=json.dumps(request,separators=(',',':'))
 parent=weft.prepare_application_lineage(raw);child=parent.prepare_current_query();assert child.source_preparation() is parent
 assert parent.source_request_json()==raw
 encoded=child.native_request_json();packet=json.loads(encoded)
 assert packet['version']=='weft.current-query.native-input/0.1-candidate'
 assert packet['sourceRequestSha256']==sha(raw.encode()) and packet['designationSha256']==sha(child.designation_json().encode())
 assert packet['authority'] is False and packet['securityReady'] is False
 assert packet['routes']==[] and len(packet['projections'])==1
 assert all(o['operator'] is None and o['operands']==[] and o['kind']=='boolean'for o in packet['operations'])
 projection=packet['projections'][0];expected={'documentId':'domain','revision':'query-value-core-r1','module':'m','element':field}
 assert projection['field']==expected and projection['path']=='outputs/0/expression' and projection['scan']=='s0'
 assert projection['descriptor'][0]['identity']==expected
 descriptor=projection['descriptor'][1];assert len(descriptor)==1 and descriptor[0]['identity']==expected
 assert descriptor[0]['type']['family']==('integer'if field=='salary'else'string')
 assert child.native_request_json()==encoded and parent.source_request_json()==raw
 (a.output/(name+'.request.json')).write_text(raw)
 (a.output/(name+'.native.json')).write_text(encoded)
 checks.append({'id':name,'sql':sql,'sameOriginalParent':True,'sourceRequestSha256':sha(raw.encode()),'packetSha256':sha(encoded.encode()),'projectionField':expected,'queryRoutes':0,'queryFieldOperands':0,'projectionOccurrences':1,'originalLogicalFamily':descriptor[0]['type']['family'],'readProjectionDecision':'not-produced'})
assert fixture.read_bytes()==original
report={'scope':'Actual original public Weft CQP producer inputs for subsequent operator-free original UMF bridge tests; no UMF execution, read disposition, output domain selection, native result or authority','weftVersion':weft.__version__,'wheel':a.wheel.name,'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native.read_bytes()),'shimSha256':sha(shim.read_bytes()),'fixtureSha256':sha(original),'checks':checks,'nativeSecurityAcceptance':False,'casesPromoted':[]}
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'version':weft.__version__,'originalProjectionPackets':len(checks),'noFabricatedQueryOperator':True}))
