import json,subprocess,hashlib
from pathlib import Path
base=json.loads(Path('/private/tmp/weft-b008-graph-request.json').read_text());binary='/private/tmp/weft-compatible-latest-target/debug/examples/security_graph_candidate';sha=lambda s:hashlib.sha256(s.encode()).hexdigest();checks=[]
def run(req,success,label):
 raw=json.dumps(req,separators=(',',':'));p=subprocess.run([binary],input=raw.encode(),capture_output=True);assert (p.returncode==0)==success,(label,p.returncode,p.stderr.decode())
 if not success:assert not p.stdout,(label,'partial output')
 else:
  out=json.loads(p.stdout);assert out['securityReady'] is False;query=out['query'];assert sha(query['resultContractJson'])==query['resultContractSha256'];assert query['resultContract']==json.loads(query['resultContractJson']);assert out['contextSha256']==query['contextSha256'];assert len(query['integrityChecks'])==4;checks.append({'case':label,'status':'prepared','contextSha256':out['contextSha256'],'outputSha256':hashlib.sha256(p.stdout).hexdigest()});return out
 checks.append({'case':label,'status':'refused','stderr':p.stderr.decode()})
for sql,count in [('SELECT r.resourceId FROM Resource r',1),('SELECT r.resourceId,r.salary FROM Resource r',2),('SELECT r.resourceId AS second,r.resourceId AS first FROM Resource r',2)]:
 req=dict(base);req['sql']=sql;out=run(req,True,sql);assert len(out['query']['resultContract']['columns'])==count
for sql in ['SELECT COUNT(*) FROM Resource r','SELECT r.resourceId FROM Resource r WHERE r.resourceId = :key','SELECT r.resourceId FROM Resource r ORDER BY r.resourceId','SELECT r.resourceId FROM Resource r LIMIT 1','SELECT r.resourceId FROM Resource r JOIN Resource q ON r.resourceId=q.resourceId']:
 req=dict(base);req['sql']=sql;run(req,False,sql)
for mutation in range(7):
 req=dict(base);source=json.loads(req['bindingJson'])
 if mutation==0:source['physicalModule']['pin']['revision']='r2' # Separate authored revision is preserved; inventory pin still binds source shape.
 elif mutation==1:source['nativeSchemaRevision']='2'
 elif mutation==2:source['correspondenceJson']='{}'
 elif mutation==3:source['inventoryJson']+=' '
 elif mutation==4:source['version']='unknown'
 elif mutation==5:source['nativeSchemaRevision']='01'
 else:source['extra']=True
 req['bindingJson']=json.dumps(source,separators=(',',':'));profile=json.loads(req['queryProfileJson']);profile['binding']['sha256']=sha(req['bindingJson']);req['queryProfileJson']=json.dumps(profile,separators=(',',':'))
 if mutation==0:
  out=run(req,True,'separate-authored-revision-correspondence-only');assert out['contextSha256']!=run(base,True,'original-authored-revision')['contextSha256']
 else:run(req,False,f'corruption-{mutation}')
Path('/private/tmp/weft-b008-graph-cli-checks.json').write_text(json.dumps({'scope':'Actual source producer positive/refusal controls; not native authority. Authored revision names remain opaque, native counter is distinct.','binarySha256':hashlib.sha256(Path(binary).read_bytes()).hexdigest(),'checks':checks},indent=2)+'\n');print(len(checks),'actual producer controls pass')
