"""Replay B-007 saved-evidence controls; never advertises full release qualification.
@covers US-006-AC1 @covers US-006-AC2
"""
import gzip,hashlib,json,pathlib,re,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]
HERE=pathlib.Path(__file__).resolve().parent
components=[
 ('audit-qualified-hosts.py','status','hostArtifactJoins',4362),
 ('qualified-host-controls.py','status','corruptionsRejected',8),
 ('audit-qualified-registration-receipts.py','status','cases',2181),
 ('reconcile-truss-review-numerics.py','status','cases',1124),
 ('truss-review-numeric-controls.py','status','corruptionsRejected',8),
 ('reconcile-conjunction-native.py','status','cases',32),
 ('conjunction-reconcile-controls.py','status','corruptionsRejected',8),
 ('audit-native-registration-join.py','status','cases',1025),
 ('unsigned-reconcile-controls.py','status','controls',15),
 ('reconcile-unsigned-boundaries.py','status','cases',8),
 ('reconcile-ashlar-engines.py','status','sameStatementEngineResults',7),
 ('ashlar-engine-controls.py','status','corruptionsRejected',6),
 ('reconcile-ashlar-warehouse-boundaries.py','status','sameStatementEngineResults',7),
 ('ashlar-warehouse-controls.py','status','corruptionsRejected',6),
 ('audit-ashlar-support-reports.py','status','casesAudited',22),
 ('reconcile-ashlar-warehouse-counts.py','status','cases',32),
 ('warehouse-capture-controls.py','status','controls',9),
 ('ashlar-count-controls.py','status','corruptionsRejected',15),
 ('audit-ashlar-count-hosts.py','status','hostArtifactJoins',64),
 ('reconcile-ashlar-warehouse-application.py','status','cases',112),
 ('reconcile-ashlar-warehouse-compounds.py','status','cases',48),
 ('reconcile-ashlar-warehouse-relationships.py','status','cases',52),
 ('reconcile-warehouse-settings.py','status','nativeStatements',3),
 ('reconcile-ashlar-warehouse-scalars.py','status','cases',10),
 ('reconcile-ashlar-warehouse-values.py','status','cases',133),
 ('../ashlar-databricks/host-obligation-check.py','state','cases',53),
 ('reconcile-unsigned-all-widths.py','status','cases',64),
 ('reconcile-signed-boundaries.py','status','cases',64),
 ('reconcile-decimal-domains.py','status','cases',434),
 ('decimal-reconcile-controls.py','status','corruptionsRejected',7),
 ('signed-reconcile-controls.py','status','corruptionsRejected',7),
 ('audit-current-hosts.py','status','hostArtifactJoins',1182),
 ('reconcile-truss-archive.py','status','cases',76),
 ('audit-truss-sessions.py','status','cases',76),
 ('truss-session-controls.py','status','corruptionsRejected',6),
 ('audit-native-profile-scopes.py','status','independentRowComparisons',76),
 ('native-scope-controls.py','status','corruptionsRejected',11),
 ('audit-truss-support-reports.py','status','casesAudited',76),
 ('host-receipt-controls.py','status','corruptionsRejected',16),
 ('evidence-check.py','state','cases',41),
 ('reconcile-ashlar-application.py','status','cases',112),
 ('reconcile-controls.py','status','controls',11),
 ('reconcile-ashlar-compound-pages.py','status','cases',48),
 ('reconcile-ashlar-relationships.py','status','cases',52),
]
results=[]
for name,status,count,expected in components:
 run=subprocess.run([sys.executable,str(HERE/name)],cwd=ROOT,capture_output=True,text=True)
 assert run.returncode==0,(name,run.stdout,run.stderr)
 report=json.loads(run.stdout)
 assert report[status]=='passed' and report[count]==expected,(name,report)
 if name=='reconcile-ashlar-warehouse-counts.py':
  assert report['logicalMetadataCases']==32
 if name=='reconcile-ashlar-warehouse-application.py':
  assert report['independentRowComparisons']==112 and report['sameStatementWarehouseCases']==96 and report['separateEmptyQueryProbes']==16
 if name=='reconcile-ashlar-warehouse-compounds.py':
  assert report['independentDecodedComparisons']==48 and report['sameStatementWarehouseCases']==24 and report['separateEmptyPageProbes']==24
 if name=='reconcile-ashlar-warehouse-relationships.py':
  assert report['positive']==28 and report['refusals']==24 and report['uniqueNativeGuards']==272 and report['sameStatementWarehouseResults']==296 and report['separateEmptyQueryProbes']==4
 if name=='reconcile-ashlar-warehouse-scalars.py':
  assert report['positive']==2 and report['refusals']==8 and report['integrityStatements']==40 and report['sameStatementWarehouseResults']==41 and report['separateEmptyQueryProbes']==1
 if name=='reconcile-unsigned-all-widths.py':
  assert report['nativeReceipts']==190 and report['sameStatementEngineResults']==63 and report['successfulReceipts']==189 and report['expectedFailedReceipts']==1
 if name=='reconcile-signed-boundaries.py':
  assert report['nativeReceipts']==194 and report['successfulReceipts']==192 and report['expectedFailedReceipts']==2 and report['sameStatementWarehouseCases']==64
 if name=='audit-current-hosts.py':
  assert report['cases']==591 and report['ashlarCases']==515 and report['trussCases']==76
 if name=='reconcile-ashlar-warehouse-values.py':
  assert report['positive']==56 and report['refusals']==77 and report['independentDecodedComparisons']==56 and report['integrityStatements']==288 and report['sameStatementWarehouseResults']==343 and report['separateEmptyQueryProbes']==1
 if name=='audit-truss-support-reports.py':
  assert report['scopesAudited']==21 and report['nativeSessionProvenance']['sameTransactionCases']==76
 if name=='audit-ashlar-support-reports.py':
  assert report['scopesAudited']==7 and report['successfulNativeStatements']==21 and report['expectedNativeFailures']==1
 if name=='audit-native-profile-scopes.py':
  assert report['distinctArtifactScopes']==21 and report['orderedComparisons']==65
 results.append({'component':name,'result':report,'sourceSha256':hashlib.sha256((HERE/name).read_bytes()).hexdigest()})
references={}
for name in ['B-007-acceptance-matrix.json','B-007-support-inventory.json']:
 value=json.loads((ROOT/'docs/helix/04-build/evidence'/name).read_text())
 def visit(v):
  if isinstance(v,dict):
   if set(v)=={'path','sha256'}:
    p=ROOT/v['path'];assert p.is_relative_to(ROOT) and p.exists()
    assert hashlib.sha256(p.read_bytes()).hexdigest()==v['sha256'],v['path']
    references[v['path']]=v['sha256']
   for child in v.values():visit(child)
  elif isinstance(v,list):
   for child in v:visit(child)
 visit(value)
 if name=='B-007-acceptance-matrix.json':
  assert len(value['criteria'])==30 and len({r['id'] for r in value['criteria']})==30
  assert value['status']=='acceptance-complete'
  assert all(r['assessment']=='passed' and r['acceptanceReason'] and r['evidence'] for r in value['criteria'])
  assert value['unresolvedMergeGates']==['terminal corrected CI at the final PR head','final PR review and merge']
 else:
  assert value['status']=='compiler-conformance-qualified'
  assert len(value['supportedNativeProfiles'])==2
  assert {p['targetProfile'] for p in value['supportedNativeProfiles']}=={'pg17.9-qualified-fixtures','dbsql2026.39-qualified'}
  assert all(p['backendVersion']=='0.1.0-qualified' for p in value['supportedNativeProfiles'])
  assert value['releasedPackages'] is False
workspace_path=ROOT/'docs/helix/04-build/evidence/B-007-workspace-qualified-final'
workspace=json.loads((workspace_path/'summary.json').read_text())
assert workspace['status']=='passed'
source_bytes=(workspace_path/'sources.json').read_bytes()
assert hashlib.sha256(source_bytes).hexdigest()==workspace['sourceManifestSha256']
sys.path.insert(0,str(ROOT/'scripts'))
from reliability.custody import historical
historical_custody=historical(ROOT)
for composition,count in [('candidate',241),('qualified',242)]:
 receipt=workspace[composition]
 assert receipt['status']=='passed' and receipt['exitCode']==0 and receipt['testsExecuted']==count and receipt['terminalSuites']==35 and receipt['ignored']==receipt['filtered']==0
 raw=gzip.decompress((workspace_path/(composition+'.log.gz')).read_bytes())
 assert hashlib.sha256(raw).hexdigest()==receipt['logSha256']
 suites=re.findall(rb'test result: (\w+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out',raw)
 assert len(suites)==35 and sum(int(r[1]) for r in suites)==count
 assert all(r[0]==b'ok' and all(int(n)==0 for n in r[2:]) for r in suites)
review=json.loads((ROOT/'docs/helix/04-build/evidence/B-007-final-criterion-review.json').read_text())
matrix=json.loads((ROOT/'docs/helix/04-build/evidence/B-007-acceptance-matrix.json').read_text())
assert review['status']=='acceptance-complete; CI-and-merge-pending'
assert {r['id'] for r in review['criteria']}=={r['id'] for r in matrix['criteria']}
for row in review['criteria']:
 assert row['assessment']=='passed' and row['reason']
 for reference in row['evidence']:
  assert hashlib.sha256((ROOT/reference['path']).read_bytes()).hexdigest()==reference['sha256'],reference['path']
results.append({'component':'final-workspace-and-criterion-records','result':{'status':'passed','candidateTests':241,'qualifiedTests':242,'criteria':30,'sourceHashesVerified':len(json.loads(source_bytes))},'historicalSourceCustody':historical_custody,'scope':'Saved logs and immutable historical Git object custody only; current files and fresh CI are qualified separately at the final pushed head.'})
report={'status':'passed','components':results,'verifiedEvidenceReferences':len(references),'scope':'Retained receipt reconciliation and synthetic verifier controls only. Does not execute native databases, Rust properties, Python wheels or browser WASM; does not close release gates.'}
print(json.dumps({'status':'passed','components':len(results),'verifiedEvidenceReferences':len(references),'releaseQualified':False}))
