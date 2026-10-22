from pathlib import Path
import json,subprocess,hashlib,sys
root=Path('/private/tmp/weft-truss-hf04');sys.path.insert(0,str(root/'tests/qualify-and-evolve'));from licensed_source import source_custody
refs={}
def visit(v):
 if isinstance(v,dict):
  if set(v)=={'path','sha256'}:refs[(v['path'],v['sha256'])]=v
  for c in v.values():visit(c)
 elif isinstance(v,list):
  for c in v:visit(c)
for name in ('B-007-acceptance-matrix.json','B-007-support-inventory.json'):visit(json.loads((root/'docs/helix/04-build/evidence'/name).read_bytes()))
r=root/'docs/helix/04-build/evidence/truss-selective-sources-20261022/retained-input-repair';records=[]
for path,digest in refs:
 try:source_custody(root/path,digest);continue
 except ValueError:pass
 commits=subprocess.check_output(['git','log','--all','--format=%H','--',path],cwd=root,text=True).splitlines();found=False
 for commit in commits:
  done=subprocess.run(['git','show',commit+':'+path],cwd=root,capture_output=True)
  if done.returncode==0 and hashlib.sha256(done.stdout).hexdigest()==digest:
   target=r/'sources'/(digest+Path(path).suffix);target.parent.mkdir(exist_ok=True);target.write_bytes(done.stdout);records.append({'path':path,'sha256':digest,'archive':str(target.relative_to(root)),'gitCommit':commit});found=True;break
 if not found:raise RuntimeError('Missing original git source '+path+' '+digest)
 print(path,'recovered',flush=True)
(r/'source-custody-map.json').write_text(json.dumps({'scope':'Original B007 references only; byte custody does not assert current implementation equivalence or runtime support','sources':records},indent=2)+'\n');print(len(refs),len(records))
