"""Join final actual Python/Chromium qualified receipts to native-tested artifacts.
@covers US-005-AC1 @covers US-005-AC2 @covers US-005-AC3 @covers US-005-AC4
"""
import gzip,hashlib,json,os
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];BASE=Path(os.environ.get('WEFT_QUALIFIED_HOST_EVIDENCE',str(ROOT/'docs/helix/04-build/evidence/B-007-qualified-hosts')))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
def content(name):
 p=BASE/name
 if p.exists():return p.read_text()
 return gzip.decompress((BASE/(name+'.gz')).read_bytes()).decode()
def verify(base):
 global BASE
 BASE=base
 if (BASE/'custody.json').exists():
  custody=json.loads((BASE/'custody.json').read_text())
  for name,digest in custody['files'].items():assert sha(BASE/name)==digest,name
 summary=json.loads(content('cli-summary.json'));assert summary['status']=='passed' and summary['cases']==2181
 source=ROOT/'docs/helix/04-build/evidence/B-007-qualified-registration/compiled-artifacts.jsonl.gz';assert sha(source)==summary['sourceNativeArtifactSha256']
 originals={r['id']:r for r in [json.loads(l) for l in gzip.decompress(source.read_bytes()).decode().splitlines()]}
 cases=[json.loads(l) for l in content('cases.jsonl').splitlines()];reports=json.loads(content('cli-reports.json'))
 requests={c['id']:c['request'] for c in cases};responses={r['id']:r['raw'] for r in reports}
 assert len(cases)==len(requests)==len(reports)==len(responses)==len(originals)==2181 and set(requests)==set(responses)==set(originals)
 for id in originals:assert requests[id]==originals[id]['request'] and json.loads(responses[id])==originals[id]['response'],id
 package=json.loads(content('package-summary.json'));assert package['status']=='passed' and package['features']==['truss-postgresql-qualified','ashlar-databricks-qualified'] and package['recordEntriesVerified']==5
 joined=0
 for host in ['python','browser']:
  receipt=json.loads(content(host+'-receipts.json'));runtime=receipt['summary'];rows=receipt['cases'];index={r['id']:r for r in rows}
  assert runtime==json.loads(content(host+'-summary.json'))
  assert len(rows)==len(index)==runtime['cases']==2181 and set(index)==set(responses) and runtime['byteParity'] is True
  if host=='python':assert runtime['extensionSha256']==package['nativeExtensionSha256'] and runtime['subprocessDisabled'] is True and runtime['version']=='0.1.0' and runtime['python'].startswith('3.12.14')
  else:
   assert runtime['wasmSha256']==package['wasmSha256'] and runtime['nodeGlobals'] is False and runtime['playwrightVersion']=='1.62.1' and runtime['browser']=='148.0.7778.96'
   assert all(i['module']=='wbg' and i['kind']=='function' for i in runtime['imports'])
  for id,row in index.items():
   digest=hashlib.sha256(responses[id].encode()).hexdigest();assert row['actualSha256']==row['expectedSha256']==digest,id
   request=json.dumps(requests[id],ensure_ascii=False,**({'separators':(',',':')} if host=='browser' else {}))
   assert row['requestSha256']==hashlib.sha256(request.encode()).hexdigest(),id
   joined+=1
 resources=json.loads(content('resource-cases.json'));assert len(resources)==7
 resource_digest=hashlib.sha256(content('resource-cases.json').encode()).hexdigest()
 for host in ['python','browser']:
  r=json.loads(content('resource-'+host+'-summary.json'));assert r['status']=='passed' and r['cases']==7 and r['casesAndResponsesSha256']==resource_digest
  if host=='python':assert r['extensionSha256']==package['nativeExtensionSha256'] and r['subprocessDisabled'] is True
  else:assert r['wasmSha256']==package['wasmSha256'] and r['wrapperSha256']==package['wrapperSha256'] and r['byteParity'] is True
 return dict(status='passed',cases=2181,hostArtifactJoins=joined,resourceCasesPerHost=7,scope='Complete actual Python and Chromium artifact/pin/runtime joins to native-qualified requests, with exact loaded binaries and package custody. No separate host SQL executions or broader platform/distribution claims.')
if __name__=='__main__':print(json.dumps(verify(BASE)))
