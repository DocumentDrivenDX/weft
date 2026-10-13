from pathlib import Path
import json,copy,subprocess,hashlib
out=Path(__file__).resolve().parent
binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike')
base=json.loads(Path('/private/tmp/truss-security-main-integration/packages/python/tests/fixtures/security-mapping-request.json').read_text())
dump=lambda v:json.dumps(v,separators=(',',':'),ensure_ascii=False)
sha=lambda s:hashlib.sha256(s.encode()).hexdigest()
sel={'sourceSha256':base['modules'][0]['pin']['sha256'],'identity':{'module':'m','element':'Staff','key':'pk'}}
results=[]
def call(name,value,positive=False,code=None):
 raw=value if isinstance(value,str) else dump(value)
 run=subprocess.run([str(binary)],input=raw.encode(),capture_output=True,timeout=20)
 if positive:
  assert run.returncode==0,(name,run.stderr.decode());assert json.loads(run.stdout)=={'securityReady':False,'subjectKeyInspectionPassed':True},(name,run.stdout)
 else:
  assert run.returncode!=0 and not run.stdout,(name,run.returncode,run.stdout);error=json.loads(run.stderr)
  if code:assert error['code']==code,(name,error)
 results.append({'name':name,'passed':True,'exitCode':run.returncode,'inputBytes':len(raw.encode()),'stdoutBytes':len(run.stdout),'diagnostic':json.loads(run.stderr)['code'] if run.stderr else None})
def packet(req=None,s=None):return {'request':copy.deepcopy(base if req is None else req),'subjectKeyProbe':copy.deepcopy(sel if s is None else s)}
def repin_source(req):
 req['modules'][0]['pin']['sha256']=sha(req['modules'][0]['documentJson']);p=json.loads(req['queryProfileJson']);p['modelPins']=[req['modules'][0]['pin']];req['queryProfileJson']=dump(p)
def repin(req,field,key):
 p=json.loads(req['queryProfileJson']);p[key]=sha(req[field]);req['queryProfileJson']=dump(p)
call('original source-only subject succeeds',packet(),True)
for key,value in [('module','other'),('element','Resource'),('key','missing')]:
 p=packet();p['subjectKeyProbe']['identity'][key]=value;call('wrong selected '+key,p,code='WFT-SECURITY-SUBJECT-KEY')
for value in ['0'*64,1,None,{'sha256':sel['sourceSha256']}]:
 p=packet();p['subjectKeyProbe']['sourceSha256']=value;call('wrong source hash '+str(type(value)),p,code='WFT-SECURITY-SUBJECT-KEY')
for s in [None,[],{}, {'sourceSha256':sel['sourceSha256']}, {**sel,'extra':True}, {**sel,'identity':{**sel['identity'],'extra':True}}, {**sel,'identity':{**sel['identity'],'key':None}},{**sel,'identity':{**sel['identity'],'key':''}},{**sel,'identity':{**sel['identity'],'key':'p\0k'}},{**sel,'identity':{**sel['identity'],'key':'x'*4097}}]:
 call('closed selection '+str(len(results)),packet(s=s) if s is not None else {'request':base,'subjectKeyProbe':None},code='WFT-SECURITY-SUBJECT-KEY')
rawreq=dump(base);rawsel=dump(sel)
call('duplicate selection member','{"request":'+rawreq+',"subjectKeyProbe":{"sourceSha256":"'+sel['sourceSha256']+'","sourceSha256":"'+sel['sourceSha256']+'","identity":'+dump(sel['identity'])+'}}')
call('duplicate escaped identity member','{"request":'+rawreq+',"subjectKeyProbe":{"sourceSha256":"'+sel['sourceSha256']+'","identity":{"module":"m","element":"Staff","key":"pk","\\u006bey":"pk"}}}')
for extra in [('extra',True),('cut',{}),('evaluationRequest',{}),('resultProbe',{})]:
 p=packet();p[extra[0]]=extra[1];call('closed outer extra '+extra[0],p)
for request_extra in ['ir','actions','resolvedQuery']:
 p=packet();p['request'][request_extra]={};call('request override '+request_extra,p,code='WFT-SECURITY-MAPPING-TRANSPORT')
p=packet();p['request']['sql']='SELECT nope FROM';call('invalid owner SQL cannot pass selection',p)
p=packet();p['request']['sql']='SELECT p.projectId FROM Project p';call('valid SQL outside admitted target refuses',p)
p=packet();p['request']['queryProfileJson']='{}';call('missing query profile refuses',p)
for field,pin in [('ontologyJson','ontologySha256'),('policyJson','policySha256')]:
 p=packet();p['request'][field]+=' ';call('stale '+field+' pin refuses',p)
 repin(p['request'],field,pin);call('rehashed original '+field+' whitespace passes',p,True)
p=packet();p['request']['modules'][0]['documentJson']+=' ';call('stale original model pin refuses',p)
repin_source(p['request']);call('changed model hash refuses old subject selection',p,code='WFT-SECURITY-SUBJECT-KEY')
p['subjectKeyProbe']['sourceSha256']=p['request']['modules'][0]['pin']['sha256'];call('matching changed original source selection passes',p,True)
# Exact request budget uses original-source whitespace, with all original pins refreshed.
p=packet();repin_source(p['request']);padding=4_000_000-len(dump(p['request']).encode());p['request']['modules'][0]['documentJson']+=' '*padding;repin_source(p['request']);p['subjectKeyProbe']['sourceSha256']=p['request']['modules'][0]['pin']['sha256'];assert len(dump(p['request']).encode())==4_000_000
call('exact 4000000-byte original request admitted',p,True)
p['request']['modules'][0]['documentJson']+=' ';repin_source(p['request']);p['subjectKeyProbe']['sourceSha256']=p['request']['modules'][0]['pin']['sha256'];assert len(dump(p['request']).encode())==4_000_001
call('4000001-byte original request refused',p,code='WFT-SECURITY-MAPPING-TRANSPORT')
(out/'binary-results.json').write_text(json.dumps({'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'scope':'Actual original rebuilt Weft diagnostic binary; source-only selection correspondence, no native execution or authority. Every refusal requires nonzero exit and empty stdout.','count':len(results),'results':results},indent=2)+'\n')
print(json.dumps({'controls':len(results),'allPassed':True}))
