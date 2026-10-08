"""Reaudit retained engine-registration equivalence and refusal controls.
@covers US-006-AC1 @covers US-006-AC3
"""
import copy,gzip,hashlib,json,os
from pathlib import Path
from importlib.util import spec_from_file_location,module_from_spec
ROOT=Path(__file__).resolve().parents[2]
BASE=Path(os.environ.get('WEFT_REGISTRATION_EVIDENCE',str(ROOT/'docs/helix/04-build/evidence/B-007-native-registration-join')))
spec=spec_from_file_location('registration_join',Path(__file__).with_name('join-native-registrations.py'));join=module_from_spec(spec);spec.loader.exec_module(join)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
summary=json.loads((BASE/'summary.json').read_text());assert summary['status']=='passed' and summary['cases']==1025
assert summary['harnessSha256']==sha(Path(__file__).with_name('join-native-registrations.py'))
for name,digest in summary['inputHashes'].items():assert sha(ROOT/name)==digest,name
assert summary['artifactSha256']==sha(BASE/'joined-artifacts.jsonl.gz')
assert summary['manifestSha256']==sha(BASE/'manifests.json')
manifests=json.loads((BASE/'manifests.json').read_text())
assert {m['backendId'] for m in manifests}=={'truss.postgresql','ashlar.databricks'}
for m in manifests:
 assert m['backendVersion']=='0.1.0-native-review' and m['evidence']==[]
 assert all(c['status']=='candidate' and c['evidence']==[] for c in m['capabilities'])
 assert sorted(set(c['id'] for c in m['capabilities'])-set(summary['operationIds'][m['backendId']]))==summary['unobservedDeclaredOperations'][m['backendId']]==['and']
rows=[json.loads(l) for l in gzip.decompress((BASE/'joined-artifacts.jsonl.gz').read_bytes()).decode().splitlines()]
originals={c['id']:c for c in join.corpus()};assert len(rows)==len(originals)==len({r['id'] for r in rows})==1025
compiled=blocked=0
for row in rows:
 old=originals[row['id']];expected=copy.deepcopy(old['request'])
 expected['target']['backendVersion']='0.1.0-native-review'
 backend=expected['target']['backendId'];expected['target']['targetProfile']='pg17.9-native-review' if backend=='truss.postgresql' else 'dbsql2026.39-native-review'
 assert row['request']==expected
 for module in expected['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256']
 assert hashlib.sha256(expected['target']['bindingJson'].encode()).hexdigest()==expected['target']['bindingSha256']
 join.check(old['response'],row['response'],backend)
 if row['response']['status']=='compiled':compiled+=1
 else:blocked+=1
assert (compiled,blocked)==(summary['compiled'],summary['blocked'])
seed=next(r for r in rows if r['response']['status']=='compiled' and r['response']['parameters'])
old=originals[seed['id']]['response'];backend=seed['request']['target']['backendId']
controls=[]
def mutate(name,fn):
 value=copy.deepcopy(seed['response']);fn(value)
 try:join.check(old,value,backend)
 except (AssertionError,KeyError,TypeError):controls.append(name)
 else:raise AssertionError('Accepted corrupted registration '+name)
mutate('SQL body',lambda x:x.__setitem__('sql',x['sql']+' '))
mutate('parameters',lambda x:x['parameters'][0].__setitem__('value','0'))
mutate('result decoder',lambda x:x['columns'][0].__setitem__('decoder','float'))
mutate('model pins',lambda x:x.__setitem__('modelPins',[]))
mutate('binding digest',lambda x:x.__setitem__('bindingSha256','0'*64))
mutate('old host obligations',lambda x:x.__setitem__('obligations',x['obligations'][-1:]))
mutate('native host obligation',lambda x:x.__setitem__('obligations',[o for o in x['obligations'] if not o['id'].endswith('.nativeProfile')]))
mutate('silent support promotion',lambda x:x['qualification'].__setitem__('status','supported'))
mutate('engine settings',lambda x:x['targetContext']['sessionSettings'].__setitem__('comparison','locale-default'))
mutate('profile drift',lambda x:x['backend'].__setitem__('targetProfile','newer'))
print(json.dumps(dict(status='passed',cases=1025,compiled=compiled,blocked=blocked,corruptionsRejected=len(controls),controls=controls,scope='Retained engine-registration artifact/pin equivalence; controls corrupt checked semantics rather than a file hash. No new native SQL executions or support promotion.')))
