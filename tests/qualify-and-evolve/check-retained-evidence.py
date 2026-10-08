"""Replay B-007 saved-evidence controls; never advertises full release qualification.
@covers US-006-AC1 @covers US-006-AC2
"""
import hashlib,json,pathlib,subprocess,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]
HERE=pathlib.Path(__file__).resolve().parent
components=[
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
  assert value['status']=='in-progress'
 else:
  assert value['status']=='candidate-preparation' and value['supportedNativeProfiles']==[]
  assert value['releasedPackages'] is False
report={'status':'passed','components':results,'verifiedEvidenceReferences':len(references),'scope':'Retained receipt reconciliation and synthetic verifier controls only. Does not execute native databases, Rust properties, Python wheels or browser WASM; does not close release gates.'}
OUT=ROOT/'docs/helix/04-build/evidence/B-007-retained-evidence-replay';OUT.mkdir(exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'status':'passed','components':len(results),'verifiedEvidenceReferences':len(references),'releaseQualified':False}))
