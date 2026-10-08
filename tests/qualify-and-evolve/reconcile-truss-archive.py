"""@covers US-006-AC1 @covers US-006-AC2
Verify retained fresh PostgreSQL application archive and matching embedding counts.
"""
import gzip,hashlib,json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-truss-application-native'
EMBED=ROOT/'docs/helix/04-build/evidence/B-007-truss-embeddings'
compressed=(BASE/'reports.json.gz').read_bytes();custody=json.loads((BASE/'reports-custody.json').read_text())
assert hashlib.sha256(compressed).hexdigest()==custody['gzipSha256']
raw=gzip.decompress(compressed)
assert len(raw)==custody['bytes'] and hashlib.sha256(raw).hexdigest()==custody['uncompressedSha256']
reports=json.loads(raw);summary=json.loads((BASE/'summary.json').read_text())
corpus=ROOT/'tests/truss-postgresql/fixtures/application-cases.json'
assert summary['status']=='passed' and summary['cases']==len(reports)==76
assert hashlib.sha256(corpus.read_bytes()).hexdigest()==summary['corpusSha256']
assert hashlib.sha256((ROOT/'tests/truss-postgresql/application-native.py').read_bytes()).hexdigest()==summary['harnessSha256']
cases=json.loads(corpus.read_text());index={c['id']:c for c in cases}
assert len(index)==len(cases)==len(reports) and len({r['id'] for r in reports})==len(reports)
assert set(index)=={r['id'] for r in reports}
for r in reports:
 response=r['response'];assert json.loads(r['raw'])==response and response['status']=='compiled'
 request=index[r['id']]['request']
 assert response['bindingSha256']==hashlib.sha256(request['target']['bindingJson'].encode()).hexdigest()
 assert response['modelPins']==[m['pin'] for m in request['modules']]
 for m in request['modules']:assert hashlib.sha256(m['documentJson'].encode()).hexdigest()==m['pin']['sha256']
 for p in response['parameters']:assert isinstance(p['value'],str)
 for row in r['rows']:assert len(row)==len(response['columns'])
for name in ['python-summary.json','browser-summary.json']:
 h=json.loads((EMBED/name).read_text());assert h['cases']==76 and h['byteParity'] is True
wheel=json.loads((EMBED/'wheel.json').read_text());assert wheel['features']==['truss-postgresql-candidate']
print(json.dumps({'status':'passed','cases':76,'archiveBytes':len(raw),'scope':'Retained archive/pin/artifact consistency and recorded embedding parity; no new native execution or independent row oracle.'}))
