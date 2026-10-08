"""Inspect actual retained native artifacts without promoting support status."""
import gzip,hashlib,json,os
from collections import Counter
from pathlib import Path
root=Path(__file__).resolve().parents[2]
source=root/'docs/helix/04-build/evidence/B-007-truss-application-native/reports.json.gz'
reports=json.loads(gzip.decompress(source.read_bytes()))
harness=root/'tests/truss-postgresql/application-native.py'
# Evaluate only trusted expectation definitions, stopping before the native loop.
# The prefix reads the pinned local compiler for its hash; it executes no SQL.
os.environ['WEFT_TRUSS_COMPILER']='/private/tmp/weft-b007-truss-compile-frozen'
namespace={'__file__':str(harness)}
prefix=harness.read_text().split('reports=[]\n',1)[0]
assert 'for c in cases:' not in prefix
exec(compile(prefix,str(harness),'exec'),namespace)
requests={c['id']:c['request'] for c in namespace['cases']}
ordered=0
scopes={};states=Counter();seen=set()
for report in reports:
    assert report['id'] not in seen
    seen.add(report['id'])
    artifact=json.loads(report['raw'])
    assert artifact==report['response']
    states[artifact['status']]+=1
    request=requests[report['id']]
    assert artifact['bindingSha256']==hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()
    assert artifact['modelPins']==[m['pin'] for m in request['modules']]
    for module in request['modules']:
        assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
    canonical=namespace['canonical'];family=namespace['family']
    columns=artifact['columns']
    actual=[tuple(None if value=='__WEFT_FIXTURE_NULL__' and col['nullable'] else canonical(value,family(col)) for value,col in zip(row,columns,strict=True)) for row in report['rows']]
    name=report['id'].rsplit('-',1)[0]
    expected=[tuple(canonical(value,family(col)) for value,col in zip(row,columns,strict=True)) for row in namespace['expected_rows'][name]]
    assert Counter(actual)==Counter(expected),report['id']
    if 'ORDER BY' in request['sql'].upper():
        assert actual==expected,report['id'];ordered+=1
    if artifact['status']!='compiled':continue
    pins=artifact['logicalPlan']['modulePins']
    assert pins and all(len(p['sha256'])==64 for p in pins)
    profile=dict(compilerVersion=artifact['compilerVersion'],dialect=artifact['dialect'],irVersion=artifact['logicalPlan']['irVersion'],backend=artifact['backend'],modelPins=pins,bindingSha256=artifact['bindingSha256'])
    key=json.dumps(profile,sort_keys=True,separators=(',',':'))
    scopes.setdefault(key,[]).append(report['id'])
assert len(seen)==76
print(json.dumps(dict(status='passed',sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),cases=len(seen),artifactStates=dict(states),distinctArtifactScopes=len(scopes),independentRowComparisons=len(reports),orderedComparisons=ordered,harnessSha256=hashlib.sha256(harness.read_bytes()).hexdigest(),scopes=[dict(profile=json.loads(k),cases=v) for k,v in sorted(scopes.items())],qualification='Artifact pins and independent authored row expectations reconciled. Engine/session provenance and required host layers must be joined separately; no support promotion.'),indent=2))
