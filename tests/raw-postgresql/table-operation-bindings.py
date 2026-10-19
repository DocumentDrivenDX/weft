# SPDX-License-Identifier: MIT OR Apache-2.0
"""Installed PyO3/native CLI parity over public synthetic operation source."""
import copy
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import weft

root=Path(__file__).resolve().parent
source_text=(root/'table-operation-model.json').read_text()
base=json.loads((root/'table-lifecycle-read-template.json').read_text())['binding']
base.update(modelJson=source_text,modelSha256=hashlib.sha256(source_text.encode()).hexdigest())


def encode(value):return json.dumps(value,ensure_ascii=False,separators=(',',':'))


def request(name):
    inputs={'owner':{'string':'alice'}} if name.startswith('read_') else {'id':{'string':'n1'},'owner':{'string':'alice'}}
    if name=='remove':inputs={'id':{'string':'n1'}}
    call={'kind':'batch','items':[inputs,inputs]} if name=='create_batch' else {'kind':'single','inputs':inputs}
    value={'version':'weft.table-operation/0.1','target':'postgresql18','binding':copy.deepcopy(base),'module':'demo','operation':name,'call':call}
    if not name.startswith('read_'):value['idempotencyKey']='synthetic-pinned-key'
    return value


cases=[]
for name in ['read_many','read_one','read_optional','read_page','read_count','create','rename','remove','put_ignore','put_update','create_batch']:
    cases.append((name,request(name),'compiled'))
page=request('read_page');page['call']['after']=[{'string':'n1'}];cases.append(('page-after',page,'compiled'))
quoted=request('create');quoted['call']['inputs']['owner']={'string':"é 😀 O'Reilly %_;--"};cases.append(('quoted-data',quoted,'compiled'))
lakebase=request('create');lakebase['target']='lakebase18';cases.append(('lakebase-compilation-only',lakebase,'compiled'))
for field in ['module','operation','version','target']:
    invalid=request('create');invalid[field]='forged';cases.append(('forged-'+field,invalid,'blocked'))
for name,mutate in [
    ('missing-input',lambda r:r['call']['inputs'].pop('owner')),
    ('surplus-input',lambda r:r['call']['inputs'].update(extra={'string':'x'})),
    ('missing-replay',lambda r:r.pop('idempotencyKey')),
    ('untyped-sql',lambda r:r.update(sql='SELECT arbitrary')),
    ('wrong-input-carrier',lambda r:r['call']['inputs'].update(owner={'integerToken':'1'})),
]:
    invalid=request('create');mutate(invalid);cases.append((name,invalid,'blocked'))
for path in [[],['body'],['body','values',0],['parameters',0]]:
    invalid=request('create');model=json.loads(source_text);selected=model['modules'][0]['extensions']['umf.tables']['commands'][5]
    for part in path:selected=selected[part]
    selected['future']=True;text=encode(model);invalid['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest());cases.append(('unknown-member-'+str(path),invalid,'blocked'))
# Temporal declarations share native/CLI/WASM code and preserve exact source.
temporal=request('create');model=json.loads(source_text)
for name,carrier,native in [('day','date','date'),('at','timestamp','timestamptz')]:
    ref={'module':'demo','element':name}
    model['modules'][0]['elements'].append({'id':name,'kind':'field','scalarType':carrier,'cardinality':'one','nullability':'required','extensions':{}})
    model['modules'][0]['elements'][2]['members'].append(ref)
    model['modules'][0]['extensions']['umf.tables']['tables'][0]['columns'].append({'field':ref,'name':name,'nativeType':{'kind':native}})
    temporal['binding']['tables'][0]['columns'].append({'logical':name,'native':name,'carrier':carrier,'nullable':False})
    command=next(c for c in model['modules'][0]['extensions']['umf.tables']['commands'] if c['id']=='create')
    command['parameters'].append({'id':name,'field':ref})
    command['body']['values'].append({'field':ref,'value':{'kind':'parameter','parameter':name}})
    command['body']['returning'].append(ref)
text=encode(model);temporal['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest())
temporal['call']['inputs'].update(day={'date':'2000-02-29'},at={'timestamp':'2026-01-01T00:30:00.123456000+01:00'})
cases.append(('exact-native-temporal',temporal,'compiled'))
for name,value in [('rounding',{'timestamp':'2026-01-01T00:00:00.1234567Z'}),('unknown-offset',{'timestamp':'2026-01-01T00:00:00-00:00'}),('UTC-overflow',{'timestamp':'9999-12-31T23:59:59-00:01'})]:
    bad=copy.deepcopy(temporal);bad['call']['inputs']['at']=value;cases.append(('temporal-'+name,bad,'blocked'))
for value in ['1900-02-29','infinity','0000-01-01']:
    bad=copy.deepcopy(temporal);bad['call']['inputs']['day']={'date':value};cases.append(('date-'+value,bad,'blocked'))
results=[]
browser_cases=[]
for name,value,status in cases:
    raw=encode(value)
    native=json.loads(subprocess.run([str(Path(sys.argv[1]).resolve(strict=True))],input=raw,text=True,capture_output=True,check=True,timeout=60).stdout)
    python=json.loads(weft.compile_table_operation_json(raw))
    assert native==python,name
    assert python['status']==status,(name,python)
    if status=='compiled':
        assert python['requestJson']==raw
        assert python['obligations']['nativeExecutionQualified'] is False
        assert json.loads(weft.check_table_operation_json(encode(python)))['status']=='checked'
    results.append({'name':name,'status':status,'nativePythonEqual':True})
    browser_cases.append({'name':name,'request':raw,'expected':python})
program=json.loads(weft.compile_table_operation_json(encode(request('rename'))))
controls=0
for field in ['sql','dbapiSql','module','operation','parameterSources','outputs','result','obligations','nativeInputs','nativeAfter']:
    changed=copy.deepcopy(program);changed[field]='forged';changed.pop('structuralDigest')
    changed['structuralDigest']=hashlib.sha256(json.dumps(changed,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
    assert json.loads(weft.check_table_operation_json(encode(changed)))['status']=='blocked',field
    controls+=1
if len(sys.argv)>3:Path(sys.argv[3]).write_text(json.dumps(browser_cases,ensure_ascii=False,separators=(',',':'))+'\n')
report={'cases':results,'selfRehashedProgramControls':controls,'nativeExecutionQualified':False,'browserExecutionQualified':False}
Path(sys.argv[2]).write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'parityCases':len(results),'selfRehashedControls':controls}))
