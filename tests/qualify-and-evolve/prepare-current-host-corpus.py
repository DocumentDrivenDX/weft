"""Fresh CLI comparison with retained native corpora; no engine execution.
@covers US-005-AC1 @covers US-005-AC2 @covers US-005-AC3
"""
import copy,gzip,hashlib,json,os,subprocess
from pathlib import Path
root=Path(__file__).resolve().parents[2];out=Path(os.environ['WEFT_CURRENT_HOST_OUTPUT']);out.mkdir(parents=True,exist_ok=True)
binary=Path(os.environ['WEFT_CURRENT_HOST_COMPILER']);sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
records=[];inputs={}
for scope,expected in [('scalar',10),('application',112),('compound',48),('relationship',52),('values',133),('count',32)]:
 base=root/'docs/helix/04-build/evidence'/f'B-007-ashlar-warehouse-{scope}-native';source=base/'compile-artifacts.jsonl';custody=json.loads((base/'custody.json').read_text());assert sha(source)==custody['inputHashes'][source.name];inputs[str(source.relative_to(root))]=sha(source)
 artifacts=[json.loads(l) for l in source.read_text().splitlines()];assert len(artifacts)==expected
 for a in artifacts:
  before=copy.deepcopy(a['response']);profile=next(o for o in before['obligations'] if o['id']=='ashlar.candidate.publication')['parameters']['nativeProfile'];assert profile.pop('versionReported')=='4.2.0 zero build hash'
  records.append((f'ashlar-{scope}-{a["id"]}',a['request'],before))
for scope,prefix in [('unsigned','unsigned'),('signed','signed')]:
 source=root/f'docs/helix/04-build/evidence/B-007-{scope}-all-widths-native/compile-artifacts.jsonl';custody=json.loads((source.parent/'custody.json').read_text());assert sha(source)==custody['inputHashes'][source.name];inputs[str(source.relative_to(root))]=sha(source)
 artifacts=[json.loads(l) for l in source.read_text().splitlines()];assert len(artifacts)==64
 for a in artifacts:records.append((f'ashlar-{scope}-{a["id"]}',a['request'],a['response']))
source=root/'docs/helix/04-build/evidence/B-007-truss-application-native/reports.json.gz';custody=json.loads((source.parent/'reports-custody.json').read_text());assert sha(source)==custody['gzipSha256'];inputs[str(source.relative_to(root))]=sha(source)
# Reconstruct the governed fixture requests without executing the native harness.
import ast
harness=root/'tests/truss-postgresql/application-native.py';namespace={'__file__':str(harness)};module=ast.parse(harness.read_text().split('reports=[]\n',1)[0]);module.body=[n for n in module.body if not(isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id in {'BINARY','BINARY_SHA'} for t in n.targets))];exec(compile(module,str(harness),'exec'),namespace);requests={c['id']:c['request'] for c in namespace['cases']}
reports=json.loads(gzip.decompress(source.read_bytes()));assert len(reports)==76
for r in reports:records.append(('truss-'+r['id'],requests[r['id']],r['response']))
assert len(records)==591 and len({r[0] for r in records})==591
cases=[];reports=[]
for id,request,before in records:
 raw=subprocess.check_output([str(binary)],input=json.dumps(request,ensure_ascii=False).encode()).decode().strip();assert json.loads(raw)==before,id
 cases.append({'id':id,'request':request});reports.append({'id':id,'raw':raw})
(out/'cases.json').write_text(json.dumps(cases,ensure_ascii=False)+'\n');(out/'cli-reports.json').write_text(json.dumps(reports,ensure_ascii=False)+'\n')
(out/'cli-summary.json').write_text(json.dumps({'status':'passed','cases':591,'sourceNativeArtifactHashes':inputs,'compilerSha256':sha(binary),'harnessSha256':sha(Path(__file__)),'scope':'Fresh CLI byte expectations; full artifact equivalence to 591 retained native-tested requests. Exactly the obsolete Spark metadata field is removed from the 387 historical Ashlar artifacts; unchanged 128 current numeric artifacts and 76 Truss artifacts. No new host database execution.'},indent=2)+'\n')
print('591 current CLI artifacts reconciled')
