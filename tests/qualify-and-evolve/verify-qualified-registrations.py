"""Default-supported registration preservation across all native-qualified inputs.
@covers US-002-AC2 @covers US-006-AC1 @covers US-006-AC3
Pure compiler checks; existing independently reconciled native executions supply semantics.
"""
import copy,gzip,hashlib,json,os,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];E=ROOT/'docs/helix/04-build/evidence'
OUT=Path(os.environ.get('WEFT_QUALIFIED_OUT','/private/tmp/weft-b007-qualified-registration'));OUT.mkdir(parents=True,exist_ok=True)
BINARY=Path(os.environ.get('WEFT_QUALIFIED_BINARY','/private/tmp/weft-b007-parser-target/debug/examples/compile_qualified'))
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
profiles={'truss.postgresql':'pg17.9-qualified-fixtures','ashlar.databricks':'dbsql2026.39-qualified'}
cases=[]
for line in gzip.decompress((E/'B-007-native-registration-join/joined-artifacts.jsonl.gz').read_bytes()).decode().splitlines():cases.append(json.loads(line))
for line in (E/'B-007-conjunction-native/compile-artifacts.jsonl').read_text().splitlines():
 c=json.loads(line);c['id']='and-'+c['id'];cases.append(c)
for line in gzip.decompress((E/'B-007-truss-review-numerics/compile-artifacts.jsonl.gz').read_bytes()).decode().splitlines():
 c=json.loads(line);c['id']='truss-numeric-'+c['id'];cases.append(c)
assert len(cases)==len({c['id'] for c in cases})==2181
manifests=json.loads(subprocess.check_output([str(BINARY),'--manifests']));declarations={m['backendId']:m for m in manifests}
assert set(declarations)==set(profiles)
for backend,m in declarations.items():
 assert m['backendVersion']=='0.1.0-qualified' and m['targetProfiles'][0]['id']==profiles[backend]
 qualification=E/('B-007-qualified-'+('truss' if backend=='truss.postgresql' else 'ashlar')+'.json')
 expectedEvidence='weft-native-qualification/'+('truss' if backend=='truss.postgresql' else 'ashlar')+'/0.1.0#sha256:'+sha(qualification)
 record=json.loads(qualification.read_text());assert record['status']=='native-semantics-qualified'
 for ref in record['evidence']:assert sha(ROOT/ref['path'])==ref['sha256'],ref['path']
 assert m['evidence']==[expectedEvidence]
 assert len(m['capabilities'])==27 and all(c['status']=='supported' and c['evidence']==[expectedEvidence] for c in m['capabilities'])
requests=[]
for c in cases:
 request=copy.deepcopy(c['request']);request['target']['backendVersion']='0.1.0-qualified';request['target']['targetProfile']=profiles[request['target']['backendId']];request['options']['allowCandidate']=False;requests.append(request)
run=subprocess.run([str(BINARY)],input=''.join(json.dumps(r)+'\n' for r in requests),text=True,capture_output=True);assert run.returncode==0,run.stderr
responses=[json.loads(l) for l in run.stdout.splitlines()];assert len(responses)==2181
compiled=blocked=0;records=[];operations={b:set() for b in profiles}
for c,request,after in zip(cases,requests,responses):
 before=c['response'];backend=request['target']['backendId'];profile=profiles[backend]
 assert before['status']==after['status'],(c['id'],after)
 if after['status']=='blocked':assert after==before;blocked+=1
 else:
  compiled+=1
  for key,value in before.items():
   if key not in ['backend','targetContext','qualification','obligations']:assert after[key]==value,(c['id'],key)
  assert set(after)==set(before)
  assert after['backend']==dict(before['backend'],backendVersion='0.1.0-qualified',targetProfile=profile)
  assert after['targetContext']==dict(before['targetContext'],id=profile,publicationRevision='host-verified-native-qualified/0.1')
  prefix='truss' if backend=='truss.postgresql' else 'ashlar';native=prefix+'.nativeProfile'
  expectedObligations=copy.deepcopy(before['obligations'])
  for obligation in expectedObligations:
   if obligation['id']==native:
    obligation['parameters']['targetProfile']=profile;obligation['parameters']['qualification']='evidence-qualified compiler semantics; host execution and publication remain conditional'
  assert after['obligations']==expectedObligations,(c['id'],'obligations')
  a=after['qualification'];assert a['status']=='conformance-verified' and a['assumptions']==before['qualification']['assumptions']==[] and a['evidence']==declarations[backend]['evidence']
  manifestCaps={cap['id']:cap for cap in declarations[backend]['capabilities']}
  assert len(a['operations'])==len(before['qualification']['operations'])
  for op,prior in zip(a['operations'],before['qualification']['operations']):
   id=op['assessment']['id'];operations[backend].add(id)
   assert op['declaration']==manifestCaps[id]
   assert op['assessment']==dict(prior['assessment'],status='supported',evidence=manifestCaps[id]['evidence'])
 records.append(dict(id=c['id'],request=request,response=after))
assert compiled==2180 and blocked==1
assert all(set(c['id'] for c in declarations[b]['capabilities'])==operations[b] for b in profiles)
# Explicit profile/version mismatch controls and unsupported home meaning; never choose a fallback.
controls=[]
for backend in profiles:
 seed=next(r['request'] for r in records if r['request']['target']['backendId']==backend and r['response']['status']=='compiled')
 for member,value in [('backendVersion','0.1.0-native-review'),('targetProfile','unregistered'),('backendVersion','newer')]:
  bad=copy.deepcopy(seed);bad['target'][member]=value
  refused=json.loads(subprocess.check_output([str(BINARY)],input=(json.dumps(bad)+'\n').encode()))
  assert refused['status']=='blocked' and 'sql' not in refused and refused['diagnostics'][0]['code']=='WFT-BACKEND-VERSION';controls.append(backend+' '+member+' '+value)
with gzip.open(OUT/'compiled-artifacts.jsonl.gz','wb') as f:f.write(''.join(json.dumps(r)+'\n' for r in records).encode())
(OUT/'manifests.json').write_text(json.dumps(manifests,indent=2)+'\n')
summary=dict(status='passed',cases=2181,compiled=compiled,blocked=blocked,defaultSupported=True,mismatchControls=len(controls),controls=controls,capabilitiesPerBackend=27,compilerSha256=sha(BINARY),harnessSha256=sha(Path(__file__)),inputHashes={str(p.relative_to(ROOT)):sha(p) for p in [E/'B-007-native-registration-join/joined-artifacts.jsonl.gz',E/'B-007-conjunction-native/compile-artifacts.jsonl',E/'B-007-truss-review-numerics/compile-artifacts.jsonl.gz']},artifactSha256=sha(OUT/'compiled-artifacts.jsonl.gz'),scope='Explicit qualified registrations preserve every meaning-bearing artifact across 2,181 native-tested inputs with candidate opt-in disabled. All 27 capabilities observed per backend; exact evidence/profile/obligations and mismatch refusals. No new database executions or final Python/browser qualification.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
