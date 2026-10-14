"""Original Python registration/lowering controls; source-only, no native authority."""
import argparse,json,hashlib,copy
from pathlib import Path
import weft
p=argparse.ArgumentParser();p.add_argument('--model',type=Path,required=True);p.add_argument('--binding',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
packet=json.loads(a.model.read_text());raw=a.binding.read_text();binding={'mediaType':'application/json','utf8':raw,'sha256':hashlib.sha256(raw.encode()).hexdigest()}
compiler=weft.MutationCompiler(json.dumps(packet['configuration']));configuration={'model':packet['request']['model'],'modelPin':packet['request']['modelPin'],'binding':binding,'evidence':['Retained original same-store native export; component registration controls']};handle=compiler.register_truss_selection(json.dumps(configuration));report=[]
def lowered(sql,maximum=1):
 req=dict(packet['request'],sql=sql);c=json.loads(compiler.compile_json(json.dumps(req)));assert c['status']=='compiled',c;plan=json.loads(c['plan']['utf8']);r={'selection':plan['selection']['artifact'],'model':req['model'],'binding':binding,'maxItems':maximum};return r,json.loads(compiler.lower_selection_json(handle,json.dumps(r)))
r,program=lowered(packet['request']['sql']);assert program['status']=='lowered';assert program['bounds']=={'maxItems':1,'fetchLimit':2};assert [c['kind'] for c in program['columns']]==['identity','version','key'];report.append('original descriptors, preflight and overflow bounds')
_,hostile=lowered("UPDATE products SET name='Foo' WHERE id='bar'' OR TRUE--'");assert hostile['status']=='lowered';assert "bar' OR TRUE--" not in hostile['sql'];assert hostile['parameters'][0]['value']=={'string':"bar' OR TRUE--"};report.append('hostile string retained only as typed parameter')
_,nul=lowered("UPDATE products SET name='Foo' WHERE id='x\x00y'");assert nul['status']=='blocked';report.append('unsupported PostgreSQL NUL parameter refused')
for key,value in [('sql','SELECT secret'),('principal','admin'),('backend','caller-code')]:
 bad=dict(r);bad[key]=value
 try:compiler.lower_selection_json(handle,json.dumps(bad))
 except ValueError:pass
 else:raise AssertionError('Unknown lowering member accepted')
report.append('caller SQL/principal/backend fields rejected')
try:compiler.lower_selection_json(handle+1,json.dumps(r))
except ValueError:pass
else:raise AssertionError('Uninstalled backend handle accepted')
for change in ['version','binding','member']:
 bad=copy.deepcopy(configuration)
 if change=='version':bad['modelPin']['revision']='demo1'
 elif change=='binding':bad['binding']['sha256']='0'*64
 else:bad['plugin']='file:///caller.py'
 try:compiler.register_truss_selection(json.dumps(bad))
 except ValueError:pass
 else:raise AssertionError('Original registration mismatch accepted')
report.append('uninstalled handle, stale pin, changed binding and plugin rejected')
assert json.loads(compiler.compile_json(json.dumps({'interfaceVersion':'weft-mutation-compile/9'})))['status']=='blocked'
a.output.write_text(json.dumps({'passed':True,'scope':'Native Python original source/registration/lowering controls only','checks':report},indent=2)+'\n');print(a.output.read_text())
