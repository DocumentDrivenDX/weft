"""Fresh compiler comparison against all current-build native artifact corpora.
Only the obsolete Spark observation may disappear; SQL and semantic metadata must match.
@covers US-004-AC1 @covers US-004-AC3 @covers US-004-AC4
"""
import copy,hashlib,json,os,subprocess
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2]
binary=Path(os.environ['WEFT_ASHLAR_COMPILER'])
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
binary_sha=sha(binary);records=[];inputs={}
for scope,expected in [('scalar',10),('application',112),('compound',48),('relationship',52),('values',133)]:
 base=ROOT/'docs/helix/04-build/evidence'/f'B-007-ashlar-warehouse-{scope}-native'
 custody=strict((base/'custody.json').read_text())
 path=base/'compile-artifacts.jsonl';assert sha(path)==custody['inputHashes'][path.name]
 artifacts=[strict(line) for line in path.read_text().splitlines() if line.strip()]
 assert len(artifacts)==len({a['id'] for a in artifacts})==expected
 inputs[str(path.relative_to(ROOT))]=sha(path)
 for artifact in artifacts:
  raw=json.dumps(artifact['request'],ensure_ascii=False).encode()
  result=subprocess.run([str(binary)],input=raw,capture_output=True,check=True)
  after=strict(result.stdout.decode());before=artifact['response']
  assert before['status']==after['status']=='compiled',(scope,artifact['id'],after)
  adjusted=copy.deepcopy(before)
  pub=next(o for o in adjusted['obligations'] if o['id']=='ashlar.candidate.publication')
  assert pub['parameters']['nativeProfile'].pop('versionReported')=='4.2.0 zero build hash'
  assert adjusted==after,(scope,artifact['id'],'Unexpected artifact change')
  records.append(dict(scope=scope,id=artifact['id'],requestSha256=hashlib.sha256(raw).hexdigest(),responseSha256=hashlib.sha256(result.stdout).hexdigest()))
assert len(records)==355 and sha(binary)==binary_sha
out=ROOT/'docs/helix/04-build/evidence/B-007-warehouse-metadata-correction'
out.mkdir(exist_ok=True)
summary=dict(status='passed',cases=355,compilerBinarySha256=binary_sha,candidateSourceSha256=sha(ROOT/'crates/weft-databricks/src/candidate.rs'),harnessSha256=sha(Path(__file__)),inputHashes=inputs,scope='Fresh CLI compile of all 355 current-build native corpus requests. Only obsolete versionReported disappears; emitted SQL, slots, model/binding pins, logical plans, result metadata and candidate qualifications are identical. No fresh DB execution or profile promotion.',receipts=records)
(out/'artifact-comparison.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps({k:v for k,v in summary.items() if k!='receipts'}))
