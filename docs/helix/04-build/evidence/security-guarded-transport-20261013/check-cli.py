import json,subprocess,hashlib
from pathlib import Path
BIN=Path('/private/tmp/weft-compatible-latest-target/debug/examples')
request=Path('/private/tmp/weft-b008-graph-request.json').read_text()
producer=BIN/'security_guarded_candidate';checker=BIN/'security_guarded_result';oldproducer=BIN/'security_graph_candidate';oldchecker=BIN/'security_graph_result'
def run(binary,value):
 raw=value.encode() if isinstance(value,str) else json.dumps(value,separators=(',',':')).encode()
 return subprocess.run([str(binary)],input=raw,capture_output=True)
p=run(producer,request);assert p.returncode==0,p.stderr;output=p.stdout.decode();wire=json.loads(output)
batch={'version':'weft.security.cells/0.1.0','resultContractSha256':wire['query']['resultContractSha256'],'rows':[[{'outcomeId':'original','disposition':'original','value':{'string':'RA'}}]]}
probe={'version':'weft.truss.security.guarded-result/0.1-candidate','sourceRequestJson':request,'sourceOutputJson':output,'batchJson':json.dumps(batch,separators=(',',':'))}
checks=[]
for case in range(9):
 value=dict(probe)
 if case==1:value['sourceOutputJson']=output.rstrip('\n')
 elif case==2:
  changed=json.loads(output);changed['subjectParameterBinding']['value']=1;value['sourceOutputJson']=json.dumps(changed,separators=(',',':'))+'\n'
 elif case==3:value['version']='weft.truss.security.graph-result/0.1-candidate'
 elif case==4:value['sourceRequestJson']=request+' '
 elif case==5:value['permit']=True
 elif case==6:
  corrupt=dict(batch);corrupt['rows']=[[{'outcomeId':'withheld','disposition':'withheld','value':{'string':'RA'}}]];value['batchJson']=json.dumps(corrupt)
 elif case==7:
  old=run(oldproducer,request);assert old.returncode==0,old.stderr;value['sourceOutputJson']=old.stdout.decode()
 elif case==8:
  changed=json.loads(output);changed['query']['execution']['sql']='SELECT 1';value['sourceOutputJson']=json.dumps(changed,separators=(',',':'))+'\n'
 r=run(checker,value);assert (r.returncode==0)==(case==0),(case,r.stderr)
 if case:assert not r.stdout
 else:assert json.loads(r.stdout)['securityReady'] is False
 checks.append({'case':case,'status':'inspected' if case==0 else 'refused','stdoutEmpty':bool(case and not r.stdout)})
r=run(checker,[probe['version'],request,output,probe['batchJson']]);assert r.returncode and not r.stdout;checks.append({'case':'positional-carrier','status':'refused','stdoutEmpty':True})
legacy=dict(probe);legacy['version']='weft.truss.security.graph-result/0.1-candidate';r=run(oldchecker,legacy);assert r.returncode and not r.stdout;checks.append({'case':'old-inspector-rejects-guarded-output','status':'refused','stdoutEmpty':True})
for field in ['subject','epoch','permit']:
 changed=json.loads(request);changed[field]=True;r=run(producer,changed);assert r.returncode and not r.stdout;checks.append({'case':'input-'+field,'status':'refused','stdoutEmpty':True})
changed=json.loads(request);changed['sql']='SELECT r.resourceId, r.salary FROM Resource r';r=run(producer,changed);assert r.returncode==0,r.stderr;other=json.loads(r.stdout);assert wire['sourceContextJson']==other['sourceContextJson'] and wire['queryContextSha256']!=other['queryContextSha256'];checks.append({'case':'distinct-query-same-sources','status':'prepared','securityReady':other['securityReady']})
receipt={'scope':'Actual guarded producer/inspector and old-inspector compatibility controls only; no native source/actor/selection/release proof.','producerSha256':hashlib.sha256(producer.read_bytes()).hexdigest(),'checkerSha256':hashlib.sha256(checker.read_bytes()).hexdigest(),'checks':checks}
Path('/private/tmp/weft-guarded-cli-checks.json').write_text(json.dumps(receipt,indent=2)+'\n');print(str(len(checks))+' actual guarded CLI controls pass')
