"""@covers US-006-AC1: reject corrupted saved-native receipt inputs."""
import copy,json,os,pathlib,subprocess,tempfile,sys
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=ROOT/'docs/helix/04-build/evidence/B-006-application-native'
artifacts=[json.loads(x) for x in (BASE/'compile-artifacts.jsonl').read_text().splitlines()]
statements=[json.loads(x) for x in (BASE/'statements.jsonl').read_text().splitlines()]
summary=json.loads((BASE/'summary.json').read_text())
query=next(i for i,x in enumerate(statements) if x['label'].endswith('-user-query'))
guard=next(i for i,x in enumerate(statements) if 'scalarIntegrity' in x['label'])
controls=['sql','parameter','terminal','truncation','row-loss','row-count','guard-value','missing-guard','duplicate-artifact','duplicate-outcome']
reports=[]
with tempfile.TemporaryDirectory(prefix='weft-receipt-controls-') as temp:
 dest=pathlib.Path(temp)
 for name in ['baseline']+controls:
  a=copy.deepcopy(artifacts);r=copy.deepcopy(statements);s=copy.deepcopy(summary)
  q=r[query]
  if name=='sql':q['sql']+=' changed'
  elif name=='parameter':q['parameters'][0]['value']='changed'
  elif name=='terminal':q['response']['status']['state']='FAILED'
  elif name=='truncation':q['response']['manifest']['truncated']=True
  elif name=='row-loss':q['response']['result']['data_array']=[]
  elif name=='row-count':q['response']['result']['row_count']+=1
  elif name=='guard-value':r[guard]['response']['result']['data_array']=[['1']]
  elif name=='missing-guard':r.pop(guard)
  elif name=='duplicate-artifact':a.append(copy.deepcopy(a[0]))
  elif name=='duplicate-outcome':s['outcomes'].append(copy.deepcopy(s['outcomes'][0]))
  (dest/'compile-artifacts.jsonl').write_text('\n'.join(json.dumps(x) for x in a)+'\n')
  (dest/'statements.jsonl').write_text('\n'.join(json.dumps(x) for x in r)+'\n')
  (dest/'summary.json').write_text(json.dumps(s))
  env=dict(os.environ,WEFT_RECONCILE_INPUT=str(dest),WEFT_RECONCILE_OUTPUT=str(dest/'out'))
  run=subprocess.run([sys.executable,str(ROOT/'tests/qualify-and-evolve/reconcile-ashlar-application.py')],env=env,capture_output=True,text=True)
  if name=='baseline':assert run.returncode==0,run.stderr
  else:assert run.returncode!=0 and ('AssertionError' in run.stderr or 'KeyError' in run.stderr),(name,run.stdout,run.stderr)
  reports.append({'control':name,'exitCode':run.returncode,'expectedOutcome':'accepted' if name=='baseline' else 'refused'})
OUT=ROOT/'docs/helix/04-build/evidence/B-007-ashlar-application-reconciliation'
(OUT/'controls.json').write_text(json.dumps({'status':'passed','controls':reports,'scope':'Verifier corruption controls; not native execution.'},indent=2)+'\n')
print(json.dumps({'status':'passed','controls':len(reports)}))
