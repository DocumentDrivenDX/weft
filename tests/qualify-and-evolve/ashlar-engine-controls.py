"""Reject corrupted engine/result receipts without submitting native statements."""
import hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-engine-native'
rejected=[]
for label in ['engine','sum','sql','parameter','statement-id','custody']:
 with tempfile.TemporaryDirectory() as folder:
  path=Path(folder)
  for name in ['summary.json','compile-artifacts.jsonl','statements.jsonl','custody.json']:(path/name).write_bytes((BASE/name).read_bytes())
  records=[json.loads(line) for line in (path/'statements.jsonl').read_text().splitlines()]
  target=next(r for r in records if r['label']=='1-valid-sum')
  if label=='engine':target['response']['result']['data_array'][0][0]='different engine'
  elif label=='sum':target['response']['result']['data_array'][0][1]='3'
  elif label=='sql':target['sql']=target['sql'].replace('version()',"'invented'")
  elif label=='parameter':target['parameters'][0]['value']='changed'
  elif label=='statement-id':records[1]['response']['statement_id']=records[0]['response']['statement_id']
  (path/'statements.jsonl').write_text('\n'.join(json.dumps(r) for r in records)+'\n')
  custody=json.loads((path/'custody.json').read_text())
  custody['inputHashes']['statements.jsonl']=hashlib.sha256((path/'statements.jsonl').read_bytes()).hexdigest()
  if label=='custody':custody['inputHashes']['statements.jsonl']='0'*64
  (path/'custody.json').write_text(json.dumps(custody))
  env=dict(os.environ,WEFT_UNSIGNED_ENGINE_RECEIPTS='1',WEFT_UNSIGNED_RECONCILE_INPUT=str(path),WEFT_UNSIGNED_RECONCILE_OUTPUT=str(path/'output'))
  result=subprocess.run([sys.executable,str(Path(__file__).with_name('reconcile-unsigned-boundaries.py'))],env=env,capture_output=True,text=True)
  assert result.returncode!=0 and 'AssertionError' in result.stderr,(label,result.stdout,result.stderr)
  rejected.append(label)
print(json.dumps({'status':'passed','corruptionsRejected':len(rejected),'controls':rejected}))
