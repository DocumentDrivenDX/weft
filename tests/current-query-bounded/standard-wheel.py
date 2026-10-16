"""Actual feature-disabled wheel closure, not native database qualification."""
import argparse,hashlib,importlib,json,pathlib,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--packet',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
native=pathlib.Path(importlib.import_module('weft.weft').__file__);shim=pathlib.Path(weft.__file__)
with zipfile.ZipFile(a.wheel)as z:
 assert native.read_bytes()==z.read(next(n for n in z.namelist()if n.endswith(('.so','.pyd'))))
 assert shim.read_bytes()==z.read('weft/__init__.py')
checks=[]
for v in json.loads(a.packet.read_text())['vectors']:
 child=weft.prepare_application_lineage(v['request']).prepare_current_query();suffix=v['backend']+'_bounded_current_'+v['kind']+'_candidate_json'
 for prefix in ['lower_','check_']:
  assert not hasattr(child,prefix+suffix);checks.append(v['id']+'/'+prefix+'absent')
 try:getattr(child,'lower_'+v['backend']+'_current_'+v['kind']+'_candidate_json')(v['binding'])
 except ValueError:checks.append(v['id']+'/old-lowerer-refused')
 else:raise AssertionError('old lowerer widened')
assert len(checks)==len(set(checks))
r=dict(scope='Actual standard-feature wheel: bounded methods absent, old lowerers refuse new contexts; no native admission.',controls=len(checks),checks=checks,wheelSha256=hashlib.sha256(a.wheel.read_bytes()).hexdigest(),nativeModuleSha256=hashlib.sha256(native.read_bytes()).hexdigest(),shimSha256=hashlib.sha256(shim.read_bytes()).hexdigest())
a.output.write_text(json.dumps(r,indent=2)+'\n');print(len(checks))
