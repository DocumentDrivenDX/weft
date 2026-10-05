"""Collect native frontend reports for independent validation and browser parity."""
import json,subprocess
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
reports=[]
for c in json.loads((ROOT/'tests/application/fixtures/cases.json').read_text()):
    raw=subprocess.check_output([ROOT/'target/debug/frontend'],input=json.dumps(c['request'],ensure_ascii=False).encode()).decode().strip()
    r=json.loads(raw)
    assert r['status']==c['expected']['status'],(c['id'],r)
    if r['status']=='blocked':assert r['diagnostics'][0]['code']==c['expected']['code'],(c['id'],r)
    else:assert r['retainedModules']==c['request']['modules']
    reports.append(dict(id=c['id'],raw=raw,response=r))
OUT=ROOT/'target/b002a';OUT.mkdir(exist_ok=True)
(OUT/'reports.json').write_text(json.dumps(reports,ensure_ascii=False))
print('Application native frontend decisions:',len(reports))
