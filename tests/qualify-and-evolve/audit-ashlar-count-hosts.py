"""Join actual host compiler receipts to native COUNT artifacts; no execution."""
import hashlib,json
from pathlib import Path
from evidence_audit import strict
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-count-host-parity'
summary=strict((BASE/'summary.json').read_text());assert summary['status']=='passed' and summary['cases']==32
native_summary=strict((ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-count-native/summary.json').read_text())
assert summary['compilerSha256']==native_summary['compilerBinarySha256']
for name,digest in summary['inputHashes'].items():assert hashlib.sha256((BASE/name).read_bytes()).hexdigest()==digest
source=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-count-native/compile-artifacts.jsonl'
assert hashlib.sha256(source.read_bytes()).hexdigest()==summary['nativeArtifactsSha256']
artifacts={a['id']:a for a in (strict(line) for line in source.read_text().splitlines())}
reports=strict((BASE/'cli-reports.json').read_text());index={r['id']:r for r in reports};assert len(index)==len(reports)==32 and set(index)==set(artifacts)
for id,r in index.items():assert strict(r['raw'])==artifacts[id]['response']
joined=0
for host in ['python','browser']:
 receipt=strict((BASE/f'{host}-receipts.json').read_text());runtime=receipt['summary'];cases=receipt['cases'];ids={c['id']:c for c in cases}
 assert len(ids)==len(cases)==runtime['cases']==32 and set(ids)==set(artifacts) and runtime['byteParity'] is True
 retained=strict((ROOT/f'docs/helix/04-build/evidence/B-006-final-{host}-summary.json').read_text())
 if host=='python':
  assert runtime['extensionSha256']==retained['extensionSha256'] and runtime['python']==retained['python'] and runtime['version']=='0.1.0' and runtime['subprocessDisabled'] is True
 else:
  for key in ['wasmSha256','browser','playwrightVersion','wasmBytes']:assert runtime[key]==retained[key]
  assert runtime['nodeGlobals'] is False
 for id,c in ids.items():
  expected=hashlib.sha256(index[id]['raw'].encode()).hexdigest();assert c['actualSha256']==c['expectedSha256']==expected
  request=json.dumps(artifacts[id]['request'],ensure_ascii=False,**({'separators':(',',':')} if host=='browser' else {}))
  assert c['requestSha256']==hashlib.sha256(request.encode()).hexdigest();joined+=1
print(json.dumps({'status':'passed','hostArtifactJoins':joined,'cases':32,'scope':'Retained actual Python/browser compiler-artifact and runtime identity joins. No host database execution or support promotion.'}))
