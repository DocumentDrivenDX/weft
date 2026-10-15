"""Installed optional native bridge: exact original report bytes and refusals.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import hashlib,json,pathlib,sys
import weft
root=pathlib.Path(__file__).resolve().parents[2]
checks=[];api=weft.prepare_application_lineage_json
for folder in ['application-lineage','lineage-v02']:
 for request in sorted((root/'tests'/folder/'fixtures').glob('*.request.json')):
  report=request.with_name(request.name.replace('.request.','.report.'))
  if not report.exists():continue
  raw=request.read_text();result=api(raw)
  assert result.encode()==report.read_bytes(),request.name
  parsed=json.loads(result);assert parsed['authority'] is False and parsed['securityReady'] is False
  checks.append({'case':folder+':'+request.stem,'reportSha256':hashlib.sha256(result.encode()).hexdigest(),'passed':True})
request=json.loads((root/'tests/lineage-v02/fixtures/distinct-required.request.json').read_text());request['readRequirements']={}
for raw in ['{}','[]','{"interfaceVersion":1,"interfaceVersion":2}',json.dumps(request),' '*4_000_001]:
 try:api(raw)
 except ValueError as e:assert str(e).startswith('WFT-') and len(str(e).encode())<8192
 else:raise AssertionError('Invalid source unexpectedly returned a report')
 checks.append({'refusal':True})
# Ordinary compilation cannot reinterpret the source-report envelope as permission.
ordinary=json.loads(weft.compile_json((root/'tests/lineage-v02/fixtures/distinct-required.request.json').read_text()))
assert ordinary['status']!='compiled' and 'sql' not in ordinary
print(json.dumps({'scope':'Installed native source-only bridge; no native data/current authority','module':weft.__file__,'version':weft.__version__,'checks':checks},indent=2))
