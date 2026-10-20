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
domain=request('create');model=json.loads(source_text)
model['modules'][0]['elements'][1]['allowedValues']=[{'string':'alice'},{'string':'bob'}]
model['modules'][0]['elements'][1]['facets']={'length':{'unit':'unicode-scalar','min':1,'max':8}}
text=encode(model);domain['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest())
cases.append(('source-domain-enumeration',domain,'compiled'))
bad=copy.deepcopy(domain);bad['call']['inputs']['owner']={'string':'other'};cases.append(('source-domain-rejects',bad,'blocked'))
clock=copy.deepcopy(temporal);model=json.loads(clock['binding']['modelJson'])
command=next(c for c in model['modules'][0]['extensions']['umf.tables']['commands'] if c['id']=='create')
command['parameters']=[p for p in command['parameters'] if p['id']!='at']
next(v for v in command['body']['values'] if v['field']['element']=='at')['value']={'kind':'currentTimestamp'}
text=encode(model);clock['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest());clock['call']['inputs'].pop('at')
cases.append(('source-transaction-clock',clock,'compiled'))
for label,fixture in [('direct','table-related-model.json'),('nested','table-related-nested-model.json')]:
    model=json.loads((root/fixture).read_text());module=model['modules'][0]
    fields={e['id']:e for e in module['elements'] if e['kind']=='field'}
    records={e['id']:e for e in module['elements'] if e['kind']=='record'}
    tables=[]
    for table in module['extensions']['umf.tables']['tables']:
        record=records[table['record']['element']];key=next(k for k in record['keys'] if k.get('primary'))
        tables.append({'logical':['demo',record['id']],'native':table['name'],'key':[f['element'] for f in key['fields']],
                       'columns':[{'logical':c['field']['element'],'native':c['name'],'carrier':fields[c['field']['element']]['scalarType'],
                                   'nullable':fields[c['field']['element']]['nullability']=='absent-allowed'} for c in table['columns']]})
    value=request('read_many');text=encode(model)
    value['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest(),tables=tables)
    value.update(operation='all_notes',call={'kind':'single','inputs':{}})
    cases.append(('related-'+label,value,'compiled'))
    bad=copy.deepcopy(value);model=json.loads(bad['binding']['modelJson'])
    model['modules'][0]['extensions']['umf.tables']['tables'][0]['policy']['person']['select']['matches'][0]['future']={'retained':True}
    text=encode(model);bad['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest())
    cases.append(('related-'+label+'-unknown-pair-refuses',bad,'blocked'))
# Atomic source snapshots use original same-module source fields and native identity.
model=json.loads((root/'table-snapshot-model.json').read_text());module=model['modules'][0]
fields={e['id']:e for e in module['elements'] if e['kind']=='field'}
records={e['id']:e for e in module['elements'] if e['kind']=='record'}
tables=[]
for table in module['extensions']['umf.tables']['tables']:
    record=records[table['record']['element']];key=next(k for k in record['keys'] if k.get('primary'))
    tables.append({'logical':['demo',record['id']],'native':table['name'],'key':[f['element'] for f in key['fields']],
                   'columns':[{'logical':c['field']['element'],'native':c['name'],'carrier':fields[c['field']['element']]['scalarType'],
                               'nullable':fields[c['field']['element']]['nullability']=='absent-allowed'} for c in table['columns']]})
snapshot=request('create');text=encode(model)
snapshot['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest(),tables=tables)
snapshot.update(operation='capture',call={'kind':'single','inputs':{'snapshot_id':{'string':'s1'},'source_id':{'string':'n1'},'comment':None}})
cases.append(('atomic-source-snapshot',snapshot,'compiled'))
all_rows=copy.deepcopy(snapshot);all_rows.update(operation='capture_all',call={'kind':'single','inputs':{'comment':None}})
cases.append(('atomic-source-overflow-declaration',all_rows,'compiled'))
for label,mutate in [
    ('source-field-outside-source',lambda c:c['body']['values'][1]['value'].update(field={'module':'demo','element':'actor'})),
    ('source-predicate-cannot-read-copy',lambda c:c['body']['source']['where'].update(value={'kind':'sourceField','field':{'module':'demo','element':'id'}})),
    ('identity-cannot-be-time',lambda c:c['body']['values'][4].update(value={'kind':'currentPrincipal'})),
    ('unknown-source-meaning',lambda c:c['body']['source'].update(future=True)),
]:
    bad=copy.deepcopy(snapshot);model=json.loads(bad['binding']['modelJson'])
    command=next(c for c in model['modules'][0]['extensions']['umf.tables']['commands'] if c['id']=='capture')
    mutate(command);text=encode(model);bad['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest())
    cases.append((label,bad,'blocked'))
principal=request('create_batch');model=json.loads(principal['binding']['modelJson'])
command=next(c for c in model['modules'][0]['extensions']['umf.tables']['commands'] if c['id']=='create_batch')
command['parameters']=[p for p in command['parameters'] if p['id']=='id']
command['body']['values'][1]['value']={'kind':'currentPrincipal'}
text=encode(model);principal['binding'].update(modelJson=text,modelSha256=hashlib.sha256(text.encode()).hexdigest())
principal['call']['items']=[{'id':{'string':'a'}},{'id':{'string':'b'}}]
cases.append(('batch-native-principal',principal,'compiled'))
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
batch_program=json.loads(weft.compile_table_operation_json(encode(principal)))
assert 'CURRENT_USER::pg_catalog.text' in batch_program['nativeBatch']['sql']
batch_program['nativeBatch']['sql']='forged'
batch_program.pop('structuralDigest')
batch_program['structuralDigest']=hashlib.sha256(json.dumps(batch_program,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()).hexdigest()
assert json.loads(weft.check_table_operation_json(encode(batch_program)))['status']=='blocked'
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
