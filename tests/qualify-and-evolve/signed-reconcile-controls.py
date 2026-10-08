"""Reject semantically corrupted signed native receipts after custody is rehashed."""
import copy,hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];BASE=ROOT/'docs/helix/04-build/evidence/B-007-signed-all-widths-native';checker=Path(__file__).with_name('reconcile-signed-boundaries.py')
original=[json.loads(l) for l in (BASE/'statements.jsonl').read_text().splitlines()];labels={r['label']:i for i,r in enumerate(original)}
controls=[('sum-bag',lambda r:r[labels['1-valid-sum']]['response']['result']['data_array'][0].__setitem__(1,'-1')),('range-count',lambda r:r[labels['32-invalid-guard']]['response']['result']['data_array'][0].__setitem__(0,'2')),('native-null',lambda r:r[labels['64-invalid-guard']]['response']['result']['data_array'][0].__setitem__(0,'0')),('overflow-code',lambda r:r[labels['64-carrier-overflow-below']]['response']['status']['error'].__setitem__('message','unrelated failure')),('parameters',lambda r:r[0]['parameters'][0].__setitem__('value','wrong-source')),('warehouse-build',lambda r:r[labels['64-valid-sum']]['response']['result']['data_array'][0].__setitem__(0,'{}')),('extra-receipt',lambda r:r.append(copy.deepcopy(r[0])))]
with tempfile.TemporaryDirectory() as folder:
 out=Path(folder)
 for name,mutate in controls:
  for p in BASE.iterdir():
   if p.is_file():(out/p.name).write_bytes(p.read_bytes())
  rows=copy.deepcopy(original);mutate(rows);(out/'statements.jsonl').write_text('\n'.join(json.dumps(r) for r in rows)+'\n')
  custody=json.loads((out/'custody.json').read_text());custody['inputHashes']['statements.jsonl']=hashlib.sha256((out/'statements.jsonl').read_bytes()).hexdigest();(out/'custody.json').write_text(json.dumps(custody))
  result=subprocess.run([sys.executable,str(checker)],cwd=ROOT,env={**os.environ,'WEFT_SIGNED_RECONCILE_INPUT':str(out),'WEFT_SIGNED_RECONCILE_OUTPUT':str(out/'checked')},capture_output=True,text=True)
  assert result.returncode!=0 and 'AssertionError' in result.stderr,(name,result.stdout,result.stderr)
print(json.dumps(dict(status='passed',corruptionsRejected=len(controls),controls=[c[0] for c in controls],scope='Semantic native receipt corruptions refused with recomputed custody hashes. No native execution.')))
