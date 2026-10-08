"""Independent support-claim audit controls. @covers US-006-AC1 @covers US-006-AC2."""
import copy,hashlib,json,tempfile
from pathlib import Path
from evidence_audit import audit,Refused
profile=dict(compilerVersion='0.1.0',dialect='weft-sql/0.2.0',irVersion='weft-ir/0.2.0',backendVersion='0.1.0-candidate',targetProfile='isolated-test',engineVersion='PostgreSQL 17.9',layoutRevision='test-layout/1',modelSha256='a'*64,bindingSha256='b'*64,settings=dict(comparison='C',arithmetic='exact-or-error'))
expected=[['9007199254740993','0.01','é',{'state':'absent'}],['9007199254740993','0.01','é',{'state':'absent'}]]
report=dict(status='passed',layer='native',profile=profile,cases=[dict(id='exact-bag',status='passed',actual=expected)])
claim=dict(status='supported',profile=profile,requiredLayers=['native'],cases=[dict(id='exact-bag',comparison='bag',expected=expected)])
passed=[]
with tempfile.TemporaryDirectory() as temporary:
 path=Path(temporary)/'report.json'
 def probe(name,edit_report=None,edit_claim=None,expected_pass=False,hash_bad=False,missing=False,raw_transform=None):
  r=copy.deepcopy(report);c=copy.deepcopy(claim)
  if edit_report:edit_report(r)
  if edit_claim:edit_claim(c)
  data=json.dumps(r,ensure_ascii=False).encode()
  if raw_transform:data=raw_transform(data)
  path.write_bytes(data);c['evidence']=[dict(path=str(Path(temporary)/'missing.json' if missing else path),sha256=('0'*64 if hash_bad else hashlib.sha256(data).hexdigest()))]
  try:
   result=audit(c);assert expected_pass,name;assert result==dict(status='verified',cases=1,layers=['native'])
  except Refused:assert not expected_pass,name
  passed.append(name)
 probe('complete-exact-bag',expected_pass=True)
 for state in ['failed','skipped','pending']:probe('report-'+state,lambda r,s=state:r.update(status=s))
 for state in ['failed','skipped','pending']:probe('case-'+state,lambda r,s=state:r['cases'][0].update(status=s))
 for key in profile:probe('profile-'+key,lambda r,k=key:r['profile'].update({k:'changed'}))
 probe('sha256-mismatch',hash_bad=True)
 probe('library-is-not-native',lambda r:r.update(layer='library'))
 probe('missing-case',lambda r:r.update(cases=[]))
 probe('duplicate-case',lambda r:r['cases'].append(copy.deepcopy(r['cases'][0])))
 probe('lost-bag-copy',lambda r:r['cases'][0].update(actual=expected[:1]))
 probe('rounded-integer',lambda r:r['cases'][0]['actual'][0].__setitem__(0,'9007199254740992'))
 probe('rounded-decimal',lambda r:r['cases'][0]['actual'][0].__setitem__(1,'0.00'))
 probe('normalized-unicode',lambda r:r['cases'][0]['actual'][0].__setitem__(2,'e\u0301'))
 probe('absent-is-not-null',lambda r:r['cases'][0]['actual'][0].__setitem__(3,{'state':'null'}))
 probe('unknown-report-meaning',lambda r:r.update(unknown=True))
 probe('unknown-case-meaning',lambda r:r['cases'][0].update(unknown=True))
 probe('unqualified-engine',edit_claim=lambda c:c['profile'].update(engineVersion='unqualified-warehouse-release'))
 probe('candidate-is-not-supported',edit_claim=lambda c:c.update(status='candidate'))
 probe('missing-layer-declaration',edit_claim=lambda c:c.update(requiredLayers=[]))
 # JSON distinguishes booleans and numbers; Python's True == 1 must not hide loss.
 probe('boolean-is-not-integer',lambda r:r['cases'][0].update(actual=[[True]]),lambda c:c['cases'][0].update(expected=[[1]]))
 # Ordered pages need ordered proof; bag equality alone is insufficient.
 probe('ordered-row-loss',lambda r:r['cases'][0].update(actual=[['second'],['first']]),lambda c:c['cases'][0].update(comparison='ordered',expected=[['first'],['second']]))
 probe('ordered-exact',lambda r:r['cases'][0].update(actual=[['first'],['second']]),lambda c:c['cases'][0].update(comparison='ordered',expected=[['first'],['second']]),expected_pass=True)
 probe('unknown-comparison',edit_claim=lambda c:c['cases'][0].update(comparison='set'))
 probe('missing-evidence-file',missing=True)
 probe('missing-python-layer',edit_claim=lambda c:c.update(requiredLayers=['native','python']))
 probe('duplicate-json-members',raw_transform=lambda b:b.replace(b'"status": "passed"',b'"status": "failed", "status": "passed"',1))
 probe('matched-unqualified-profile',lambda r:r['profile'].update(engineVersion='unqualified'),lambda c:c['profile'].update(engineVersion='unqualified'))
 for key in ['modelSha256','bindingSha256']:
  probe('matched-invalid-'+key,lambda r,k=key:r['profile'].update({k:'not-a-digest'}),lambda c,k=key:c['profile'].update({k:'not-a-digest'}))
print(json.dumps(dict(state='passed',cases=len(passed),names=passed,qualification='Synthetic audit-algorithm controls only; no native support claim is made.')))
