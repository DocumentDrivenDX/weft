from pathlib import Path
import sys,json,hashlib
out=Path('/private/tmp/subject-consumer-review-20261013')
module=Path('/private/tmp/subject-consumer-python/weft.so').resolve()
sys.path.insert(0,str(module.parent))
import weft
assert Path(weft.__file__).resolve()==module
parent=json.loads(Path('/private/tmp/subject-consumer-python.json').read_text())
browser=json.loads(Path('/private/tmp/subject-consumer-browser.json').read_text())
sha=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest()
assert sha(module)==parent['moduleSha256']
assert len(parent['cases'])==len(browser['cases'])==14
assert all(p['id']==b['id'] and p['response']==b['response'] for p,b in zip(parent['cases'],browser['cases']))
for name,digest in browser['artifactHashes'].items():assert sha(Path('/private/tmp/subject-consumer-wasm')/name)==digest
results=[]
def check(name,request,selection,status,exact=None):
 raw=weft.inspect_security_mapping_subject_key_json(request,selection);v=json.loads(raw)
 assert set(v)==({'interfaceVersion','status','securityReady'} if status=='inspected' else {'interfaceVersion','status','securityReady','diagnostics'}),name
 assert v['interfaceVersion']=='weft-security-subject-selection/0.1.0' and v['status']==status and v['securityReady'] is False,name
 if status=='refused':assert len(v['diagnostics'])==1 and v['diagnostics'][0]['severity']=='error',name
 if exact is not None:assert raw==exact,name
 results.append({'name':name,'passed':True,'status':status,'responseSha256':hashlib.sha256(raw.encode()).hexdigest()})
for c in parent['cases']:check(c['id'],c['request'],c['selection'],c['status'],c['response'])
r,s=parent['cases'][0]['request'],parent['cases'][0]['selection']
for name,v in [('empty',''),('null','null'),('array','[]')]:check('selection-'+name,r,v,'refused')
check('selection-exact-16384',r,s+' '*(16384-len(s.encode())),'inspected')
check('selection-over-16384',r,s+' '*(16385-len(s.encode())),'refused')
check('request-exact-4000000',r+' '*(4000000-len(r.encode())),s,'inspected')
check('request-over-4000000',r+' '*(4000001-len(r.encode())),s,'refused')
check('duplicate-request',r[:-1]+',"sql":"SELECT anything"}',s,'refused')
(out/'results.json').write_text(json.dumps({'status':'passed','modulePath':str(module),'moduleSha256':sha(module),'count':len(results),'results':results,'retainedBrowserCorrespondence':{'count':14,'chromium':browser['chromium'],'externalRequests':browser['externalRequests'],'artifactHashesVerified':True,'rerunByReviewer':False}},indent=2)+'\n')
print('Passed',len(results),'actual extension controls; exact module path verified; retained browser14 response/artifact correspondence verified')
