"""Real native report joins must reject altered host receipts."""
import copy,json,os,subprocess,sys,tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[2]
audit=root/'tests/qualify-and-evolve/audit-truss-support-reports.py'
mutations={
 'missing-case':lambda r:r['cases'].pop(),
 'duplicate-id':lambda r:r['cases'][1].update(id=r['cases'][0]['id']),
 'changed-actual':lambda r:r['cases'][0].update(actualSha256='0'*64),
 'changed-request':lambda r:r['cases'][0].update(requestSha256='0'*64),
 'false-parity':lambda r:r['summary'].update(byteParity=False),
}
passed=[]
with tempfile.TemporaryDirectory() as tmp:
 path=Path(tmp)/'receipt.json'
 for host in ['python','browser']:
  baseline=json.loads((root/f'docs/helix/04-build/evidence/B-007-truss-{host}-case-receipts/receipts.json').read_text())
  for name,edit in mutations.items():
   changed=copy.deepcopy(baseline);edit(changed);path.write_text(json.dumps(changed))
   result=subprocess.run([sys.executable,str(audit)],env=dict(os.environ,**{f'WEFT_{host.upper()}_RECEIPTS':str(path)}),capture_output=True,text=True)
   assert result.returncode!=0 and 'AssertionError' in result.stderr,(host,name,result.stderr)
   passed.append(f'{host}-{name}')
print(json.dumps({'status':'passed','corruptionsRejected':len(passed),'names':passed,'scope':'Host/native artifact receipt joins only; no native engine or host execution rerun.'}))
