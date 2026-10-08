"""Inspect actual retained native artifacts without promoting support status."""
import gzip,hashlib,json
from collections import Counter
from pathlib import Path
root=Path(__file__).resolve().parents[2]
source=root/'docs/helix/04-build/evidence/B-007-truss-application-native/reports.json.gz'
reports=json.loads(gzip.decompress(source.read_bytes()))
scopes={};states=Counter();seen=set()
for report in reports:
    assert report['id'] not in seen
    seen.add(report['id'])
    artifact=json.loads(report['raw'])
    assert artifact==report['response']
    states[artifact['status']]+=1
    if artifact['status']!='compiled':continue
    pins=artifact['logicalPlan']['modulePins']
    assert pins and all(len(p['sha256'])==64 for p in pins)
    profile=dict(compilerVersion=artifact['compilerVersion'],dialect=artifact['dialect'],irVersion=artifact['logicalPlan']['irVersion'],backend=artifact['backend'],modelPins=pins,bindingSha256=artifact['bindingSha256'])
    key=json.dumps(profile,sort_keys=True,separators=(',',':'))
    scopes.setdefault(key,[]).append(report['id'])
assert len(seen)==76
print(json.dumps(dict(status='passed',sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),cases=len(seen),artifactStates=dict(states),distinctArtifactScopes=len(scopes),scopes=[dict(profile=json.loads(k),cases=v) for k,v in sorted(scopes.items())],qualification='Artifact pin grouping only. Engine/session provenance, independent expectations and required host layers must be joined separately; no support promotion.'),indent=2))
