"""Audit retained default-supported registration receipts without rebuilding hosts.
@covers US-006-AC1 @covers US-006-AC3
"""
import gzip,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];E=ROOT/'docs/helix/04-build/evidence';BASE=E/'B-007-qualified-registration'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=json.loads((BASE/'custody.json').read_text())
for name,digest in custody['files'].items():assert sha(BASE/name)==digest,name
summary=json.loads((BASE/'summary.json').read_text());assert summary['status']=='passed' and summary['cases']==2181 and summary['compiled']==2180 and summary['blocked']==1 and summary['defaultSupported'] is True and summary['mismatchControls']==6
for name,digest in summary['inputHashes'].items():assert sha(ROOT/name)==digest,name
assert sha(BASE/'compiled-artifacts.jsonl.gz')==summary['artifactSha256']
rows=[json.loads(l) for l in gzip.decompress((BASE/'compiled-artifacts.jsonl.gz').read_bytes()).decode().splitlines()]
assert len(rows)==len({r['id'] for r in rows})==2181
manifests={m['backendId']:m for m in json.loads((BASE/'manifests.json').read_text())};assert len(manifests)==2
compiled=blocked=0;capabilities={b:set() for b in manifests}
for row in rows:
 request=row['request'];response=row['response'];backend=request['target']['backendId'];manifest=manifests[backend]
 assert request['options']['allowCandidate'] is False and request['target']['backendVersion']=='0.1.0-qualified'
 if response['status']=='blocked':assert 'sql' not in response;blocked+=1;continue
 compiled+=1
 assert response['qualification']['status']=='conformance-verified' and response['qualification']['evidence']==manifest['evidence']
 assert response['targetContext']['id']==request['target']['targetProfile']==manifest['targetProfiles'][0]['id']
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==response['bindingSha256']==request['target']['bindingSha256']
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 assert any(o['id'].endswith('.nativeProfile') and o['parameters']['targetProfile']==request['target']['targetProfile'] for o in response['obligations'])
 declarations={c['id']:c for c in manifest['capabilities']}
 for op in response['qualification']['operations']:
  id=op['assessment']['id'];capabilities[backend].add(id)
  assert op['declaration']==declarations[id] and op['assessment']['status']=='supported' and op['assessment']['evidence']==manifest['evidence']
assert (compiled,blocked)==(2180,1) and all(len(v)==27 for v in capabilities.values())
for name in ['truss','ashlar']:
 qualification=E/f'B-007-qualified-{name}.json';record=json.loads(qualification.read_text())
 for ref in record['evidence']:assert sha(ROOT/ref['path'])==ref['sha256'],ref['path']
 manifest=manifests[record['backend']]
 assert manifest['evidence']==[f'weft-native-qualification/{name}/0.1.0#sha256:'+sha(qualification)]
print(json.dumps(dict(status='passed',cases=2181,compiled=2180,blocked=1,capabilitiesPerBackend=27,scope='Saved supported-registration receipts, exact pins/profile/evidence and manifest disposition checks. Producer differential execution separately preserves native-tested SQL. No rebuilding or new native/host execution.')))
