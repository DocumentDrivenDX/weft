from pathlib import Path
import subprocess,json,hashlib
root=Path('/private/tmp/weft-truss-hf04');r=root/'docs/helix/04-build/evidence/truss-selective-sources-20261022/retained-input-repair';r.mkdir(exist_ok=True)
raw=subprocess.check_output(['git','show','3ad557b7:tests/truss-postgresql/fixtures/application-cases.json'],cwd=root);summary=json.loads((root/'docs/helix/04-build/evidence/B-007-truss-application-native/summary.json').read_bytes());assert hashlib.sha256(raw).hexdigest()==summary['corpusSha256'];(r/'application-cases.original.json').write_bytes(raw)
(r/'recovered-custody.json').write_text(json.dumps({'scope':'Original historical evidence custody, not current compiler admission or new native execution','gitCommit':subprocess.check_output(['git','rev-parse','3ad557b7'],cwd=root,text=True).strip(),'gitPath':'tests/truss-postgresql/fixtures/application-cases.json','sha256':summary['corpusSha256'],'cases':76,'reason':'Current mutable fixture and harness now target UMF0.10, so cannot reconstruct original B007 inputs'},indent=2)+'\n')
p=root/'tests/qualify-and-evolve/licensed_source.py';p.write_text(p.read_text()+'''

def retained_truss_application_inputs(root: Path) -> tuple[Path, Path]:
    """Original producer/corpus, pinned by the immutable native run summary."""
    import json
    base = root / 'docs/helix/04-build/evidence/B-007-truss-application-native'
    summary = json.loads((base / 'summary.json').read_bytes())
    harness = base / 'harness.py'
    source_custody(harness, summary['harnessSha256'])
    corpus = root / 'docs/helix/04-build/evidence/truss-selective-sources-20261022/retained-input-repair/application-cases.original.json'
    if hashlib.sha256(corpus.read_bytes()).hexdigest() != summary['corpusSha256']:
        raise ValueError('Original retained Truss corpus custody mismatch')
    return harness, corpus
''')
p=root/'tests/qualify-and-evolve/audit-current-hosts.py';s=p.read_text().replace('from licensed_source import source_custody','from licensed_source import source_custody,retained_truss_application_inputs');start=s.index("native_base=ROOT/'docs/helix/04-build/evidence/B-007-truss-application-native'");end=s.index("exec(compile(module,str(harness),'exec'),namespace)",start);s=s[:start]+'''harness,corpus=retained_truss_application_inputs(ROOT)
# Use original producer and corpus. Archived location changes only path lookup.
namespace={'__file__':str(harness),'ROOT':ROOT,'cases':strict(corpus.read_text())}
module=ast.parse(harness.read_text().split('reports=[]\\n',1)[0]);module.body=[n for n in module.body if not(isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id in {'ROOT','cases','BINARY','BINARY_SHA'} for t in n.targets))]
'''+s[end:];p.write_text(s)
p=root/'tests/qualify-and-evolve/audit-native-profile-scopes.py';s=p.read_text().replace('from pathlib import Path','from pathlib import Path\nfrom licensed_source import retained_truss_application_inputs');s=s.replace("harness=root/'tests/truss-postgresql/application-native.py'","harness,corpus=retained_truss_application_inputs(root)");s=s.replace("namespace={'__file__':str(harness)}","namespace={'__file__':str(harness),'ROOT':root,'cases':json.loads(corpus.read_text())}");s=s.replace("{'BINARY','BINARY_SHA'}","{'ROOT','cases','BINARY','BINARY_SHA'}");p.write_text(s)
p=root/'tests/qualify-and-evolve/reconcile-truss-archive.py';s=p.read_text().replace('from pathlib import Path','from pathlib import Path\nfrom licensed_source import retained_truss_application_inputs');s=s.replace("corpus=ROOT/'tests/truss-postgresql/fixtures/application-cases.json'","_,corpus=retained_truss_application_inputs(ROOT)");p.write_text(s)
