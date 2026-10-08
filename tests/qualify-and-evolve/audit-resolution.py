"""@covers US-001-AC1 @covers US-001-AC2 @covers US-001-AC3 @covers US-001-AC4
Reconcile retained plans against named identities and independent bag expectations.
"""
import collections,hashlib,json,os,pathlib,runpy
ROOT=pathlib.Path(__file__).resolve().parents[2]
path=pathlib.Path(os.environ['WEFT_RESOLUTION_REPORTS'])
cases=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
reports=json.loads(path.read_text());by_id={r['id']:r['response'] for r in reports}
assert len(by_id)==len(reports)==636
oracle=runpy.run_path(str(ROOT/'tests/frontend/oracle.py'))
from decimal import localcontext
bags=[];refusals=[]
with localcontext() as context:
 context.prec=100
 for case in cases:
  response=by_id[case['id']];expected=case['expected']
  if expected['status']=='blocked':
   assert response['status']=='blocked' and response['diagnostics'][0]['code']==expected['code']
   assert 'logicalPlan' not in response and 'sql' not in response
   refusals.append(case['id'])
  elif 'rows' in expected:
   assert response['status']=='resolved'
   rows=[tuple(oracle['exact'](v.get('value'),v['kind']) for v in row) for row in expected['rows']]
   assert collections.Counter(oracle['node'](response['logicalPlan']['root']))==collections.Counter(rows),case['id']
   bags.append(case['id'])
for name in ['sales-join-truss.postgresql','sales-join-ashlar.databricks']:
 plan=by_id[name]['logicalPlan']['root'];assert plan['op']=='project'
 aggregate=plan['input'];assert aggregate['op']=='aggregate'
 join=aggregate['input'];assert join['op']=='innerJoin'
 assert join['left']['record']=={'documentId':'sales-fixture','revision':'fixture-r1','module':'sales','element':'customer'}
 assert join['right']['record']=={'documentId':'sales-fixture','revision':'fixture-r1','module':'sales','element':'orders'}
 assert aggregate['aggregates'][0]['argument']['identity']['element']=='order-total'
qualified=by_id['module-qualified']['logicalPlan']['root']
assert qualified['input']['record']['documentId']=='other-sales'
assert qualified['outputs'][0]['expression']['identity']['documentId']=='other-sales'
assert by_id['module-ambiguous']['diagnostics'][0]['code']=='WFT-NAME-AMBIGUOUS'
report={'status':'passed','independentBagCases':bags,'authoredRefusalCases':refusals,'qualifiedDocument':'other-sales','reportSha256':hashlib.sha256(path.read_bytes()).hexdigest(),'oracleSha256':hashlib.sha256((ROOT/'tests/frontend/oracle.py').read_bytes()).hexdigest(),'sourceSha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'scope':'Named identity assertions and independent interpreter replay of retained frontend plans; not new compiler or database execution.'}
OUT=ROOT/'docs/helix/04-build/evidence/B-007-resolution-audit';OUT.mkdir(exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':'passed','bagCases':len(bags),'refusalCases':len(refusals)}))
