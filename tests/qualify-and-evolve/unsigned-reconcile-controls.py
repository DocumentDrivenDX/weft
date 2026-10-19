"""@covers US-006-AC1: reject corruption in saved unsigned boundary evidence."""
import copy,hashlib,json,os,pathlib,subprocess,sys,tempfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-007-unsigned-boundaries-native'
artifacts=[json.loads(l) for l in (BASE/'compile-artifacts.jsonl').read_text().splitlines()]
receipts=[json.loads(l) for l in (BASE/'statements.jsonl').read_text().splitlines()]
controls=['sql','parameter','sum-rounded','guard-zeroed','overflow-succeeded','overflow-wrong-error','truncation','row-count','carrier','duplicate-label','duplicate-statement-id','module-pin','binding-pin','uint64-accepted']
reports=[]
with tempfile.TemporaryDirectory(prefix='weft-unsigned-controls-') as temp:
 dest=pathlib.Path(temp)
 for name in ['baseline']+controls:
  a=copy.deepcopy(artifacts);r=copy.deepcopy(receipts)
  q=r[0];last=r[-1]
  if name=='sql':q['sql']+=' changed'
  elif name=='parameter':q['parameters'][0]['value']='changed'
  elif name=='sum-rounded':next(x for x in r if x['label']=='63-valid-sum')['response']['result']['data_array']=[['9223372036854775800']]
  elif name=='guard-zeroed':next(x for x in r if x['label']=='1-invalid-guard')['response']['result']['data_array']=[['0']]
  elif name=='overflow-succeeded':last['response']['status']['state']='SUCCEEDED'
  elif name=='overflow-wrong-error':last['response']['status']['error']['message']='unrelated transport failure'
  elif name=='truncation':q['response']['manifest']['truncated']=True
  elif name=='row-count':q['response']['result']['row_count']=0
  elif name=='carrier':q['response']['manifest']['schema']['columns'][0]['type_name']='DOUBLE'
  elif name=='duplicate-label':r[1]['label']=r[0]['label']
  elif name=='duplicate-statement-id':r[1]['response']['statement_id']=r[0]['response']['statement_id']
  elif name=='module-pin':a[0]['request']['modules'][0]['pin']['sha256']='0'*64
  elif name=='binding-pin':a[0]['request']['target']['bindingSha256']='0'*64
  elif name=='uint64-accepted':a[-1]['response']['status']='compiled'
  for filename,data in [('compile-artifacts.jsonl',a),('statements.jsonl',r)]:
   (dest/filename).write_text('\n'.join(json.dumps(x) for x in data)+'\n')
  env=dict(os.environ,WEFT_UNSIGNED_RECONCILE_INPUT=str(dest),WEFT_UNSIGNED_RECONCILE_OUTPUT=str(dest/'out'))
  run=subprocess.run([sys.executable,str(ROOT/'tests/qualify-and-evolve/reconcile-unsigned-boundaries.py')],env=env,capture_output=True,text=True)
  if name=='baseline':assert run.returncode==0,run.stderr
  else:assert run.returncode!=0 and 'AssertionError' in run.stderr,(name,run.stdout,run.stderr)
  reports.append({'control':name,'exitCode':run.returncode,'expectedOutcome':'accepted' if name=='baseline' else 'refused'})
report={'status':'passed','controls':reports,'sourceSha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest(),'scope':'Baseline plus 14 temporary evidence corruptions; no native database execution.'}
OUT=ROOT/'docs/helix/04-build/evidence/B-007-unsigned-reconciliation'
(OUT/'controls.json').write_text(json.dumps(report,indent=2)+'\n')
print(json.dumps({'status':'passed','controls':len(reports)}))
