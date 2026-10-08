"""Whole entities and keyset pages with compounds and native typed ID homes.
@covers US-004-AC1 @covers US-004-AC2 @covers US-004-AC3 (candidate component).
Read-only existing pinned compound fixture; independent authored results.
"""
import copy
from pathlib import Path
# Reuse request/model builders; stop before fixture creation or execution.
_builder=Path(__file__).with_name('compound-native.py')
exec(compile(_builder.read_text().split('# Validate all authored input models/mappings')[0],str(_builder),'exec'))
from host_obligation_fixture import strict_json
def p(name,value):return dict(name=name,type="STRING",value=value)
c=Client(OUT);state=json.loads((ROOT/'docs/helix/04-build/evidence/B-006-compound-native/fixture-state.json').read_text());outcomes=[];artifacts=[]
for case in [x for x in cases if (x['spec']['id'] in ['nested-list','structured','cyclic','map-structured']) and x['label'] in ['exact','absent'] and not x['corrupt']]:
 for home in ['props','column']:
  req=request(case,state);doc=json.loads(req['modules'][0]['documentJson']);record=doc['modules'][0]['elements'][0];record['keys']=[dict(id='primary',name='primary',primary=True,fields=[dict(module='main',element='id')])]
  raw=json.dumps(doc,ensure_ascii=False);req['modules'][0]['documentJson']=raw;req['modules'][0]['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
  b=json.loads(req['target']['bindingJson']);b['modelPins']=[req['modules'][0]['pin']]
  if home=='column':b['records'][0]['properties'][0]['home']=dict(kind='column',value='id',nativeType='BIGINT')
  raw=json.dumps(b);req['target']['bindingJson']=raw;req['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
  req['readProfile']=dict(version='weft-application-read/0.2.0',subset='entity-page')
  for page in [0,1]:
   id=case['id']+'-'+home+'-page-'+str(page);req['sql']='SELECT t.* FROM Thing t'+(' WHERE t.id > :after' if page else '')+' ORDER BY t.id ASC LIMIT 1'
   if page:req['parameters']=dict(after=dict(family='integer',value='1'))
   artifact=compile(req);assert artifact['status']=='compiled',(id,artifact)
   artifacts.append(dict(id=id,request=copy.deepcopy(req),response=artifact));params=[p('p'+str(s['position']),s['value']) for s in artifact['parameters']]
   checks=[check for o in artifact['obligations'] if 'checks' in o['parameters'] for check in o['parameters']['checks']]
   for i,check in enumerate(checks):assert c.sql(id+'-guard-'+str(i),check['sql'],params)==[['0']]
   actual=c.sql(id+'-query',artifact['sql'],params);expected=[] if page else [['1',case['expected']]]
   decoded=[[row[0],strict_json(row[1])] for row in actual];assert decoded==expected,(id,decoded,expected)
   assert artifact['columns'][1]['representation']['kind']=='value'
   outcomes.append(dict(id=id,rows=actual,expected=expected))
assert hashlib.sha256(binary.read_bytes()).hexdigest()==binary_sha
(OUT/'compile-artifacts.jsonl').write_text(''.join(json.dumps(x,ensure_ascii=False)+'\n' for x in artifacts))
summary=dict(state='passed',cases=len(outcomes),outcomes=outcomes,compilerBinarySha256=binary_sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Whole entity/ordered keyset combinations of compound values and props/native BIGINT ID homes; same independently pinned synthetic fixture publication, no production policy qualification.')
(OUT/'summary.json').write_text(json.dumps(summary,ensure_ascii=False,indent=2)+'\n');print(json.dumps({k:v for k,v in summary.items() if k!='outcomes'}))
