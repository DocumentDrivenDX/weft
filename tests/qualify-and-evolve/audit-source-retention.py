"""@covers US-006-AC3: inspect actual preparation reports, not transport equality."""
import collections,hashlib,json,os,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
cases_path=ROOT/'docs/helix/03-test/fixtures/cases.json'
reports_path=pathlib.Path(os.environ['WEFT_RETENTION_REPORTS'])
cases=json.loads(cases_path.read_text());reports=json.loads(reports_path.read_text())
assert len(cases)==len(reports)==636
assert len({c['id'] for c in cases})==636
resolved=0;refused=0;documents=0;opaque=0
for c,r in zip(cases,reports,strict=True):
 assert c['id']==r['id'] and json.loads(r['raw'])==r['response']
 response=r['response']
 if response['status']=='resolved':
  resolved+=1;assert response['retainedModules']==c['request']['modules']
  assert response['logicalPlan']['modulePins']==[m['pin'] for m in c['request']['modules']]
  for m in response['retainedModules']:
   assert hashlib.sha256(m['documentJson'].encode()).hexdigest()==m['pin']['sha256']
   documents+=1
   doc=json.loads(m['documentJson'])
   if doc.get('extensions',{}).get('future.vendor'):opaque+=1
 else:
  refused+=1;assert response['status']=='blocked' and 'logicalPlan' not in response
assert resolved==303 and refused==333 and opaque>0
public_path=pathlib.Path(os.environ['WEFT_RETENTION_PUBLIC_REPORTS'])
public=json.loads(public_path.read_text())
counts=collections.Counter((r['response']['status'],r['response']['diagnostics'][0]['code'] if r['response']['diagnostics'] else '') for r in public)
report={'status':'passed','initialCases':636,'resolvedWithExactRetainedModules':resolved,'refusedWithoutPlan':refused,'retainedDocumentInstances':documents,'instancesWithOpaqueRootExtension':opaque,'publicResponseDistribution':[{'status':status,'code':code,'cases':n} for (status,code),n in sorted(counts.items())],'inputs':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [cases_path,reports_path,public_path]},'scope':'Audit of retained preparation receipts: exact source strings and pins. Public fixture corpus has seven compiled successes and 1266 authored refusals; transport parity does not imply native backend coverage.'}
OUT=ROOT/'docs/helix/04-build/evidence/B-007-source-retention-audit';OUT.mkdir(exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
