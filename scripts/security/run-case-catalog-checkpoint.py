"""Freeze declared repository inputs before the complete Rust workspace test run."""
import datetime,hashlib,json,os,re,subprocess,sys
from pathlib import Path
root=Path.cwd().resolve();source=Path(__file__).resolve()
if source!=root/'scripts/security/run-case-catalog-checkpoint.py' or len(sys.argv)!=1 or Path(sys.argv[0]).resolve()!=source:raise RuntimeError('Unknown exact checkpoint invocation')
previous='docs/helix/04-build/evidence/security-requirement-templates/start.json';previous_bytes=(root/previous).read_bytes();inventory=json.loads(previous_bytes)
paths=sorted(set(inventory['sourcePins'])|{previous,'scripts/security/run-case-catalog-checkpoint.py','crates/weft-core/src/security_case_catalog.rs','crates/weft-core/tests/security-required-case-plan.json','crates/weft-core/tests/security-required-case-plan.provenance.json'})
if any(Path(p).is_absolute() or '..' in Path(p).parts or not (root/p).resolve().is_relative_to(root) for p in paths):raise RuntimeError('Unknown declared input path')
captured={p:(root/p).read_bytes() for p in paths}
if captured[previous]!=previous_bytes:raise RuntimeError('Input inventory changed while freezing')
command=['cargo','test','--release','--locked','--workspace','--offline']
out=root/'docs/helix/04-build/evidence/security-case-catalog';out.mkdir(parents=True,exist_ok=True)
base=subprocess.run(['git','rev-parse','HEAD'],check=True,capture_output=True,text=True).stdout.strip()
start={'version':'weft.security.case-catalog-run/0.1.0','base':base,'startedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'command':command,'scope':'Declared repository build/test inputs frozen before execution; historical manifest supplies paths only, all bytes/hash values are captured fresh. External Cargo registry/runtime dependencies are excluded. No native qualification.','sourcePins':{p:hashlib.sha256(raw).hexdigest() for p,raw in captured.items()}}
(out/'start.json').write_text(json.dumps(start,indent=2)+'\n')
with (out/'workspace.log').open('w') as log:result=subprocess.run(command,stdout=log,stderr=subprocess.STDOUT)
raw=(out/'workspace.log').read_bytes();rows=re.findall(rb'^test result: ok\. ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored; ([0-9]+) measured; ([0-9]+) filtered out; finished in [0-9.]+s$',raw,re.M)
changed=[p for p,old in captured.items() if not (root/p).is_file() or (root/p).read_bytes()!=old]
terminal={'version':start['version'],'exitCode':result.returncode,'finishedAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'sourcePinsUnchanged':not changed,'changedInputs':changed,'testGroups':len(rows),'passed':sum(int(r[0]) for r in rows),'failed':sum(int(r[1]) for r in rows),'ignored':sum(int(r[2]) for r in rows),'filtered':sum(int(r[4]) for r in rows),'measured':sum(int(r[3]) for r in rows),'workspaceLogSha256':hashlib.sha256(raw).hexdigest(),'declaredSources':len(captured)}
(out/'terminal.json').write_text(json.dumps(terminal,indent=2)+'\n');print(json.dumps(terminal))
if result.returncode or changed or not rows or terminal['failed'] or terminal['ignored'] or terminal['filtered'] or terminal['measured']:raise RuntimeError('Complete source-qualified workspace execution refused')
