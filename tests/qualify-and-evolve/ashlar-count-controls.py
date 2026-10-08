"""Reject native COUNT receipt corruption without executing SQL."""
import hashlib,json,os,subprocess,sys,tempfile
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-warehouse-count-native'
rejected=[]
for label in ['count','group-multiplicity','group-order','warehouse','parameters','sql','statement-id','extra-receipt','model-pin','custody']:
 with tempfile.TemporaryDirectory() as directory:
  path=Path(directory)
  for name in ['summary.json','compile-artifacts.jsonl','statements.jsonl','warehouse_capture.py','custody.json']:(path/name).write_bytes((BASE/name).read_bytes())
  records=[json.loads(line) for line in (path/'statements.jsonl').read_text().splitlines()]
  count=next(r for r in records if r['label']=='8-props-props-count-user-query')
  group=next(r for r in records if r['label']=='8-props-props-group-count-user-query')
  if label=='count':count['response']['result']['data_array'][0][1]='8'
  elif label=='group-multiplicity':
   row=next(row for row in group['response']['result']['data_array'] if row[1]=='same');row[2]='1'
  elif label=='group-order':group['response']['result']['data_array'].reverse()
  elif label=='warehouse':
   identity=json.loads(count['response']['result']['data_array'][0][0]);identity['r_build_hash']='changed';count['response']['result']['data_array'][0][0]=json.dumps(identity)
  elif label=='parameters':count['parameters'][0]['value']='changed'
  elif label=='sql':
   before=count['sql'];count['sql']=before.replace('TRY_SUM(CAST(1 AS DECIMAL(38,0)))','TRY_SUM(CAST(0 AS DECIMAL(38,0)))');assert count['sql']!=before
  elif label=='statement-id':records[1]['response']['statement_id']=records[0]['response']['statement_id']
  elif label=='extra-receipt':
   extra=json.loads(json.dumps(count));extra['label']='unexpected';records.append(extra)
  elif label=='model-pin':
   artifacts=[json.loads(line) for line in (path/'compile-artifacts.jsonl').read_text().splitlines()];artifacts[0]['request']['modules'][0]['pin']['sha256']='0'*64
   (path/'compile-artifacts.jsonl').write_text('\n'.join(json.dumps(r) for r in artifacts)+'\n')
  (path/'statements.jsonl').write_text('\n'.join(json.dumps(r) for r in records)+'\n')
  custody=json.loads((path/'custody.json').read_text())
  for name in custody['inputHashes']:custody['inputHashes'][name]=hashlib.sha256((path/name).read_bytes()).hexdigest()
  if label=='custody':custody['inputHashes']['statements.jsonl']='0'*64
  (path/'custody.json').write_text(json.dumps(custody))
  env=dict(os.environ,WEFT_ASHLAR_COUNT_RECONCILE_INPUT=str(path))
  result=subprocess.run([sys.executable,str(Path(__file__).with_name('reconcile-ashlar-warehouse-counts.py'))],env=env,capture_output=True,text=True)
  assert result.returncode!=0 and 'AssertionError' in result.stderr,(label,result.stdout,result.stderr)
  rejected.append(label)
print(json.dumps({'status':'passed','corruptionsRejected':len(rejected),'controls':rejected}))
