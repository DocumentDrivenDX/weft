import argparse,copy,hashlib,importlib,json,pathlib,zipfile
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
import weft
sha=lambda b:hashlib.sha256(b).hexdigest()
native=pathlib.Path(importlib.import_module('weft.weft').__file__);shim=pathlib.Path(weft.__file__)
with zipfile.ZipFile(a.wheel)as z:
 names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1
 assert native.read_bytes()==z.read(names[0]);assert shim.read_bytes()==z.read('weft/__init__.py')
pins={'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native.read_bytes()),'shimSha256':sha(shim.read_bytes())}
assert weft.__version__=='0.4.36'
f=pathlib.Path('/private/tmp/truss-northwind-genuine-query-capture-20261015/northwind-compile-request.json');raw=f.read_bytes();original=json.loads(raw);checks=[]
for version in ('0.4.38','0.4.39'):
 request=copy.deepcopy(original);request['target']['backendVersion']=version
 result=json.loads(weft.compile_truss_runtime_json(json.dumps(request)))
 if version=='0.4.38':
  assert result['status']=='blocked';assert result['diagnostics'][0]['code']=='WFT-NAME-MISSING';assert 'sql'not in result
 else:
  assert result['status']=='compiled',result.get('diagnostics');assert result['backend']['backendVersion']==version
  assert any(p['value']==original['target']['bindingJson']for p in result['parameters'])
  assert any(p['value']=='1.23'for p in result['parameters'])
 assert request['modules']==original['modules'];assert request['target']['bindingJson']==original['target']['bindingJson']
 checks.append({'backendVersion':version,'status':result['status'],'originalModuleAndBindingPreserved':True})
assert pins=={'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native.read_bytes()),'shimSha256':sha(shim.read_bytes())}
report={'scope':'Actual installed candidate wheel and genuine owner-captured modules/binding; compiler-only, no native SQL execution','version':weft.__version__,'pinsBeforeAfter':pins,'originalRequestSha256':sha(raw),'checks':checks,'nativeAdmission':False,'securityReady':False}
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'controls':len(checks),'nativeAdmission':False}))
