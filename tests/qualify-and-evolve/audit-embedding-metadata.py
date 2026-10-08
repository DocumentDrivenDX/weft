"""@covers US-005-AC3: explicit exact-value metadata in executed Ashlar host reports."""
import hashlib,json,os,pathlib
ROOT=pathlib.Path(__file__).resolve().parents[2]
path=pathlib.Path(os.environ['WEFT_EMBEDDING_METADATA_REPORTS'])
reports=json.loads(path.read_text());assert len(reports)==463
by_id={r['id']:json.loads(r['raw']) for r in reports};assert len(by_id)==463
for id in ['global-native:empty-compile','global-native:exact-compile']:
 response=by_id[id];assert response['status']=='compiled'
 column=response['columns'][0]
 assert column['aggregate']=='sum' and column['carrier']=='text' and column['decoder']=='exact-decimal'
 assert column['nullable'] is True and column['logicalType']=={'facets':{'scale':2},'family':'decimal','nullable':True}
wide=[(id,r) for id,r in by_id.items() if id.startswith('columns-native:64-')]
assert len(wide)==24
for id,r in wide:
 assert r['status']=='compiled'
 column=r['columns'][1]
 assert column['carrier']=='text' and column['decoder']=='exact-integer' and column['logicalType']['family']=='integer'
parameter_slots=0
for r in by_id.values():
 if r['status']=='compiled':
  for p in r['parameters']:
   assert isinstance(p['value'],str)
   parameter_slots+=1
report={'status':'passed','reports':463,'wideIntegerArtifacts':len(wide),'nullableExactDecimalSumArtifacts':2,'textParameterSlots':parameter_slots,'inputSha256':hashlib.sha256(path.read_bytes()).hexdigest(),'sourceSha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'scope':'Inspect actual native Python response metadata with independent exact-carrier assertions. Matching browser/CLI response bytes are established in separate host receipts; no new engine or host execution.'}
OUT=ROOT/'docs/helix/04-build/evidence/B-007-embedding-audit';OUT.mkdir(exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report))
