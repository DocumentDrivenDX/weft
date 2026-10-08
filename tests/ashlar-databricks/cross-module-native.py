"""Preserve original field identity across supplied modules in one UMF document.
@covers US-004-AC1 @covers US-004-AC2 (candidate component)
"""
import hashlib,json,os,subprocess
from pathlib import Path
from native_transport import Client
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);FIXTURE=Path(os.environ['WEFT_ASHLAR_COLUMNS_FIXTURE'])
c=Client(OUT)
request=json.loads((FIXTURE/'64-exact-column-column-compile.json').read_text())['request']
document=json.loads(request['modules'][0]['documentJson'])
field=document['modules'][0]['elements'].pop(1)
assert field['id']=='name'
document['modules'][0]['elements'][0]['members'][0]['module']='types'
document['modules'].append(dict(id='types',namespace='types',elements=[field]))
raw=json.dumps(document);digest=hashlib.sha256(raw.encode()).hexdigest()
request['modules'][0]['documentJson']=raw;request['modules'][0]['pin']['sha256']=digest
binding=json.loads(request['target']['bindingJson']);binding['modelPins'][0]['sha256']=digest
binding['records'][0]['properties'][0]['logical']['module']='types'
raw=json.dumps(binding);request['target']['bindingJson']=raw;request['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
process=subprocess.run([os.environ['WEFT_ASHLAR_COMPILER']],input=json.dumps(request),text=True,capture_output=True,check=True)
artifact=json.loads(process.stdout);assert artifact['status']=='compiled',artifact
assert artifact['columns'][0]['sourceIdentities'][0]['module']=='types'
parameters=[dict(name='p'+str(p['position']),type='STRING',value=p['value']) for p in artifact['parameters']]
checks=next(o for o in artifact['obligations'] if o['id']=='ashlar.candidate.scalarIntegrity')['parameters']['checks']
for index,check in enumerate(checks):assert c.sql('cross-module-integrity-'+str(index),check['sql'],parameters)==[['0']]
actual=c.sql('cross-module-query',artifact['sql'],parameters)
assert sorted(actual)==sorted([['same','0'],['é','3'],['e\u0301','4'],['x','5'],['x ','6']])
(OUT/'compile.json').write_text(json.dumps(dict(request=request,response=artifact),indent=2,ensure_ascii=False)+'\n')
summary=dict(state='passed',rows=actual,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),compilerBinarySha256=hashlib.sha256(Path(os.environ['WEFT_ASHLAR_COMPILER']).read_bytes()).hexdigest(),qualification='Actual compiler/native typed projection with a declared field member in another supplied module of the same original UMF document. Exact field identity and namespace retained. Synthetic fixtures; no cross-document implicit fetching or production schema claim.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2,ensure_ascii=False)+'\n');print(json.dumps(summary,indent=2,ensure_ascii=False))
