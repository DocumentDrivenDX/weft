"""Corrupt saved native rows and pins; the scope audit must refuse each."""
import copy,gzip,json,os,subprocess,sys,tempfile
from pathlib import Path
root=Path(__file__).resolve().parents[2]
audit=root/'tests/qualify-and-evolve/audit-native-profile-scopes.py'
source=root/'docs/helix/04-build/evidence/B-007-truss-application-native/reports.json.gz'
reports=json.loads(gzip.decompress(source.read_bytes()))
def edit_artifact(r,edit):
    artifact=json.loads(r[0]['raw']);edit(artifact);r[0]['response']=artifact;r[0]['raw']=json.dumps(artifact)
def reverse_order(r):
    report=r[0]  # whole-entity-props has an authored ORDER BY and multiple rows
    assert len(report['rows'])>1
    report['rows'].reverse()
mutations={
 'lost-row':lambda r:r[0]['rows'].pop(),
 'duplicate-row':lambda r:r[0]['rows'].append(copy.deepcopy(r[0]['rows'][0])),
 'rounded-integer':lambda r:r[0]['rows'][0].__setitem__(0,'9007199254740992'),
 'wrong-binding-pin':lambda r:edit_artifact(r,lambda a:a.update(bindingSha256='0'*64)),
 'wrong-model-pin':lambda r:edit_artifact(r,lambda a:a['modelPins'][0].update(revision='wrong')),
 'changed-raw-only':lambda r:r[0].update(raw='{}'),
 'duplicate-case-id':lambda r:r[1].update(id=r[0]['id']),
 'changed-order':reverse_order,
}
passed=[]
with tempfile.TemporaryDirectory() as tmp:
    path=Path(tmp)/'reports.json.gz'
    for name,edit in mutations.items():
        changed=copy.deepcopy(reports);edit(changed);path.write_bytes(gzip.compress(json.dumps(changed).encode()))
        env=dict(os.environ,WEFT_SCOPE_REPORTS=str(path))
        result=subprocess.run([sys.executable,str(audit)],env=env,capture_output=True,text=True)
        assert result.returncode!=0 and 'AssertionError' in result.stderr,(name,result.stderr)
        passed.append(name)
print(json.dumps({'status':'passed','corruptionsRejected':len(passed),'names':passed,'scope':'Saved native row/pin audit negative controls; no engine rerun.'}))
