# SPDX-License-Identifier: MIT OR Apache-2.0
"""Seed original-source parity through CLI and installed PyO3, no database IO."""
import copy
import hashlib
import json
import subprocess
import sys
from pathlib import Path

import weft

root=Path(__file__).resolve().parent
model=json.loads((root/'table-operation-model.json').read_text())
ref=lambda field:{'module':'demo','element':field}
row=lambda key:{'table':'notes','values':[{'field':ref('id'),'value':{'string':key}},
                                        {'field':ref('owner'),'value':{'string':"é %_ O'Reilly"}}]}
model['modules'][0]['extensions']['umf.tables']['seedSets']=[{'id':'first','conflict':'error','rows':[row('a'),row('b')]},{'id':'last','conflict':'ignore','rows':[row('c')]}]
base=json.loads((root/'table-lifecycle-read-template.json').read_text())['binding']
encode=lambda value:json.dumps(value,ensure_ascii=False,separators=(',',':'))
def repin(request,source):
    text=encode(source);request['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest())
request={'version':'weft.table-seed/0.1','target':'postgresql18','module':'demo','seedSets':['first','last'],'binding':base}
repin(request,model)
cases=[('source-seeds',request,'compiled')]
for label,selection in [('last-only',['last']),('missing',['none']),('reordered',['last','first']),('repeated',['first','first']),('empty',[])]:
    value=copy.deepcopy(request);value['seedSets']=selection;cases.append((label,value,'compiled' if label=='last-only' else 'blocked'))
for label,mutate in [('unknown-row',lambda s:s['modules'][0]['extensions']['umf.tables']['seedSets'][0]['rows'][0].update(future=True)),
                     ('wrong-carrier',lambda s:s['modules'][0]['extensions']['umf.tables']['seedSets'][0]['rows'][0]['values'][0].update(value={'integerToken':'3'})),
                     ('missing-required',lambda s:s['modules'][0]['extensions']['umf.tables']['seedSets'][0]['rows'][0]['values'].pop())]:
    value=copy.deepcopy(request);source=copy.deepcopy(model);mutate(source);repin(value,source);cases.append((label,value,'blocked'))
value=copy.deepcopy(request);value['sql']='INSERT untyped';cases.append(('request-sql-refuses',value,'blocked'))
results=[];corpus=[]
for name,value,status in cases:
    raw=encode(value)
    cli=json.loads(subprocess.run([str(Path(sys.argv[1]).resolve(strict=True))],input=raw,text=True,capture_output=True,check=True,timeout=60).stdout)
    native=json.loads(weft.compile_table_seed_json(raw));assert cli==native,name
    assert native['status']==status,(name,native)
    if status=='compiled':assert json.loads(weft.check_table_seed_json(encode(native)))['status']=='checked'
    results.append({'name':name,'status':status,'nativePythonEqual':True})
    corpus.append({'name':name,'request':raw,'expected':native})
Path(sys.argv[2]).write_text(json.dumps({'cases':results,'nativeExecutionQualified':False},indent=2)+'\n')
Path(sys.argv[3]).write_text(encode(corpus)+'\n')
print(json.dumps({'parityCases':len(results)}))
