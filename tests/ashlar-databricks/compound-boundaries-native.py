"""Read-only generated codec controls; synthetic owner relation, no publication claim."""
import copy,hashlib,json,os,re,subprocess
from pathlib import Path
from native_transport import Client,NativeFailure
ROOT=Path(__file__).resolve().parents[2];OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
binary=Path(os.environ['WEFT_ASHLAR_COMPILER']);sha=hashlib.sha256(binary.read_bytes()).hexdigest()
source=ROOT/'docs/helix/04-build/evidence/B-006-compound-native-initial/compile-artifacts.jsonl'
original={x['id']:x for x in map(json.loads,source.read_text().splitlines())}
c=Client(OUT);outcomes=[];artifacts=[]
def run(name,base,value,expected,mutate=None,refuse=False):
 req=copy.deepcopy(original[base]['request'])
 if mutate:
  doc=json.loads(req['modules'][0]['documentJson']);mutate(doc);raw=json.dumps(doc,ensure_ascii=False)
  req['modules'][0]['documentJson']=raw;req['modules'][0]['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
  binding=json.loads(req['target']['bindingJson']);binding['modelPins']=[req['modules'][0]['pin']];req['target']['bindingJson']=json.dumps(binding);req['target']['bindingSha256']=hashlib.sha256(req['target']['bindingJson'].encode()).hexdigest()
 artifact=json.loads(subprocess.run([str(binary)],input=json.dumps(req),text=True,capture_output=True,check=True).stdout);assert artifact['status']=='compiled',artifact
 artifacts.append(dict(id=name,request=req,response=artifact))
 params=[dict(name='p'+str(s['position']),type='STRING',value=s['value']) for s in artifact['parameters']]
 params.append(dict(name='fixture_raw',type='STRING',value=json.dumps({'23':1,'28':value},ensure_ascii=False,separators=(',',':'))))
 # Substitute a single exact owner relation, keeping the emitted codec and guards intact.
 relation='(SELECT CAST(1 AS BIGINT) AS id,:fixture_raw AS props_json,:p1 AS source_system,CAST(:p2 AS BIGINT) AS type_id)'
 def sql(s):
  result,n=re.subn(r'`client_dev`\.`weft_b006_20261008_compounds`\.`node_type_a` VERSION AS OF 1',relation,s);assert n>0;return result
 checks=[x for o in artifact['obligations'] if o['id'] in ['ashlar.candidate.scalarIntegrity','ashlar.candidate.compoundIntegrity'] for x in o['parameters']['checks']]
 try:counts=[c.sql(name+'-guard-'+str(i),sql(x['sql']),params)[0][0] for i,x in enumerate(checks)]
 except NativeFailure:
  assert refuse,name;outcomes.append(dict(id=name,outcome='native-refusal'));return
 if refuse:assert any(int(v)>0 for v in counts),(name,counts);outcomes.append(dict(id=name,outcome='guard-refusal',counts=counts));return
 assert all(v=='0' for v in counts),(name,counts)
 actual=c.sql(name+'-query',sql(artifact['sql']),params);assert [json.loads(r[0]) for r in actual]==[expected],(name,actual,expected)
 outcomes.append(dict(id=name,outcome='exact-result',rows=actual))
names=['a.b"\\','é','e\u0301','A','a','x ']
def exotic(doc):
 elements=doc['modules'][0]['elements'];record=next(e for e in elements if e['id']=='record');record['members']=[dict(module='main',element='name'+str(i)) for i in range(len(names))]
 for i,name in enumerate(names):elements.append(dict(id='name'+str(i),name=name,kind='field',cardinality='one',nullability='required',scalarType='integer',facets={'integerWidth':{'bits':8,'signed':True}},extensions={}))
run('exact-member-names','structured-False-exact',{k:i for i,k in enumerate(names)},{'state':'value','value':{k:str(i) for i,k in enumerate(names)}},exotic)
run('unknown-member-name','structured-False-exact',{**{k:i for i,k in enumerate(names)},'x':1},None,exotic,True)
# Root counts as a node. Boundary exactly 100000 succeeds; 100001 refuses.
run('nodes-at-limit','boolean-list-False-exact',[False]*99999,{'state':'value','value':[False]*99999})
run('nodes-over-limit','boolean-list-False-exact',[False]*100000,None,refuse=True)
# Every recursive record expands leaf, note and next; missing next is a node too.
def chain(count):
 value={'leaf':1};expected={'leaf':'1','note':{'state':'absent'},'next':{'state':'absent'}}
 for _ in range(count-1):value={'leaf':1,'next':value};expected={'leaf':'1','note':{'state':'absent'},'next':{'state':'value','value':expected}}
 return value,{'state':'value','value':expected}
v,e=chain(127);run('depth-below-limit','cyclic-False-exact',v,e)
v,e=chain(128);run('depth-at-limit','cyclic-False-exact',v,None,refuse=True)
assert hashlib.sha256(binary.read_bytes()).hexdigest()==sha
(OUT/'compile-artifacts.jsonl').write_text(''.join(json.dumps(x,ensure_ascii=False)+'\n' for x in artifacts))
(OUT/'summary.json').write_text(json.dumps(dict(state='passed',cases=len(outcomes),outcomes=outcomes,compilerBinarySha256=sha,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),qualification='Read-only synthetic owner relation; codec/name/resource controls only, no publication or host qualification.'),ensure_ascii=False,indent=2)+'\n')
print(json.dumps(outcomes)[:1000])
