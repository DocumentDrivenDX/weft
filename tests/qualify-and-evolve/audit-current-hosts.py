"""Join freshly built host receipts to original native-tested compiler artifacts.
@covers US-005-AC1 @covers US-005-AC2 @covers US-005-AC3 @covers US-006-AC1
"""
import ast,copy,gzip,hashlib,json
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'docs/helix/04-build/evidence/B-007-current-initial-hosts'
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
custody=strict((BASE/'custody.json').read_text())
for name,digest in custody['inputHashes'].items():assert sha(BASE/name)==digest,name
summary=strict((BASE/'cli-summary.json').read_text());assert summary['status']=='passed' and summary['cases']==591
assert summary['harnessSha256']==sha(Path(__file__).with_name('prepare-current-host-corpus.py'))
for name,digest in summary['sourceNativeArtifactHashes'].items():assert sha(ROOT/name)==digest,name
cases=strict(gzip.decompress((BASE/'cases.json.gz').read_bytes()).decode());reports=strict(gzip.decompress((BASE/'cli-reports.json.gz').read_bytes()).decode())
requests={c['id']:c['request'] for c in cases};responses={r['id']:r['raw'] for r in reports}
assert len(cases)==len(requests)==len(reports)==len(responses)==591 and set(requests)==set(responses)
expected={};expected_requests={}
for scope,count in [('scalar',10),('application',112),('compound',48),('relationship',52),('values',133),('count',32)]:
 source=ROOT/f'docs/helix/04-build/evidence/B-007-ashlar-warehouse-{scope}-native/compile-artifacts.jsonl'
 originals=[strict(l) for l in source.read_text().splitlines()];assert len(originals)==count
 for a in originals:
  id=f'ashlar-{scope}-{a["id"]}';expected[id]=copy.deepcopy(a['response']);expected_requests[id]=a['request']
  profile=next(o for o in expected[id]['obligations'] if o['id']=='ashlar.candidate.publication')['parameters']['nativeProfile']
  assert profile.pop('versionReported')=='4.2.0 zero build hash'
for scope in ['unsigned','signed']:
 source=ROOT/f'docs/helix/04-build/evidence/B-007-{scope}-all-widths-native/compile-artifacts.jsonl'
 originals=[strict(l) for l in source.read_text().splitlines()];assert len(originals)==64
 for a in originals:
  id=f'ashlar-{scope}-{a["id"]}';expected[id]=a['response'];expected_requests[id]=a['request']
source=ROOT/'docs/helix/04-build/evidence/B-007-truss-application-native/reports.json.gz'
originals=strict(gzip.decompress(source.read_bytes()).decode());assert len(originals)==76
for r in originals:expected['truss-'+r['id']]=r['response']
# Truss requests are separately authored in its native application harness.
harness=ROOT/'tests/truss-postgresql/application-native.py';namespace={'__file__':str(harness)}
module=ast.parse(harness.read_text().split('reports=[]\n',1)[0]);module.body=[n for n in module.body if not(isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id in {'BINARY','BINARY_SHA'} for t in n.targets))]
exec(compile(module,str(harness),'exec'),namespace)
for c in namespace['cases']:expected_requests['truss-'+c['id']]=c['request']
assert set(expected)==set(responses)==set(expected_requests)
for id,request in requests.items():
 if id in expected_requests:assert request==expected_requests[id],id
 for module in request['modules']:assert hashlib.sha256(module['documentJson'].encode()).hexdigest()==module['pin']['sha256'],id
 assert hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()==request['target']['bindingSha256'],id
 assert strict(responses[id])==expected[id],id
package=strict((BASE/'package-summary.json').read_text());assert package['status']=='passed' and package['features']==['ashlar-databricks-candidate','truss-postgresql-candidate']
assert len(package['members'])==package['recordEntriesVerified'] and package['nativeExtensionSha256']
joined=0
for host in ['python','browser']:
 receipt=strict((BASE/f'{host}-receipts.json').read_text());runtime=receipt['summary'];rows=receipt['cases'];index={r['id']:r for r in rows}
 assert len(rows)==len(index)==runtime['cases']==591 and set(index)==set(responses) and runtime['byteParity'] is True
 assert runtime==strict((BASE/f'{host}-summary.json').read_text())
 if host=='python':
  assert runtime['extensionSha256']==package['nativeExtensionSha256'] and runtime['subprocessDisabled'] is True and runtime['version']=='0.1.0'
 else:
  assert runtime['wasmSha256']==package['wasmSha256'] and runtime['nodeGlobals'] is False and runtime['playwrightVersion']=='1.62.1'
 for id,row in index.items():
  digest=hashlib.sha256(responses[id].encode()).hexdigest();assert row['actualSha256']==row['expectedSha256']==digest,id
  raw=json.dumps(requests[id],ensure_ascii=False,**({'separators':(',',':')} if host=='browser' else {}))
  assert row['requestSha256']==hashlib.sha256(raw.encode()).hexdigest(),id
  joined+=1
print(json.dumps(dict(status='passed',cases=591,hostArtifactJoins=joined,ashlarCases=515,trussCases=76,scope='Fresh native Python and actual Chromium byte parity for 591 original native-tested requests, including the UInt64 compile refusal. Exact artifact equivalence after the specified obsolete Spark-field removal. No separate host database executions, supported registration promotion or platform-wide distribution claim.')))
