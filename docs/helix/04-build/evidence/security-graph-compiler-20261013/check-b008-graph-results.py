import json,subprocess,hashlib
from pathlib import Path
source=Path('/private/tmp/weft-b008-graph-request.json').read_text();producer='/private/tmp/weft-compatible-latest-target/debug/examples/security_graph_candidate';checker='/private/tmp/weft-compatible-latest-target/debug/examples/security_graph_result';p=subprocess.run([producer],input=source.encode(),capture_output=True);assert p.returncode==0,p.stderr;original=p.stdout.decode();out=json.loads(original)
batch={'version':'weft.security.cells/0.1.0','resultContractSha256':out['query']['resultContractSha256'],'rows':[[{'outcomeId':'original','disposition':'original','value':{'string':'RA'}}]]}
checks=[]
for case in range(8):
 probe={'version':'weft.truss.security.graph-result/0.1-candidate','sourceRequestJson':source,'sourceOutputJson':original,'batchJson':json.dumps(batch,separators=(',',':'))}
 if case==1:probe['sourceOutputJson']=original.rstrip('\n')
 elif case==2:
  changed=json.loads(original);changed['query']['execution']['sql']='SELECT 1';probe['sourceOutputJson']=json.dumps(changed,separators=(',',':'))+'\n'
 elif case==3:
  changed=dict(batch);changed['rows']=[[{'outcomeId':'withheld','disposition':'withheld'}]];probe['batchJson']=json.dumps(changed)
 elif case==4:
  changed=dict(batch);changed['rows']=[[]];probe['batchJson']=json.dumps(changed)
 elif case==5:
  changed=dict(batch);changed['rows']=[[{'outcomeId':'original','disposition':'original','value':{'string':'RA'},'extra':True}]];probe['batchJson']=json.dumps(changed)
 elif case==6:probe['extra']=True
 elif case==7:probe['sourceRequestJson']=source+' '
 p=subprocess.run([checker],input=json.dumps(probe,separators=(',',':')).encode(),capture_output=True);assert (p.returncode==0)==(case==0),(case,p.returncode,p.stderr)
 if case:assert not p.stdout
 else:assert json.loads(p.stdout)['securityReady'] is False
 checks.append({'case':case,'status':'inspected' if case==0 else 'refused'})
Path('/private/tmp/weft-b008-graph-result-checks.json').write_text(json.dumps({'scope':'Original context/output bytes and owner result-cell correspondence; no native selection/permission or release authority.','checkerSha256':hashlib.sha256(Path(checker).read_bytes()).hexdigest(),'checks':checks},indent=2)+'\n');print('8 source/result producer controls pass')
