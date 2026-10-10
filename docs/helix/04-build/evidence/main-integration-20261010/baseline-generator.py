import gzip,hashlib,json,pathlib,subprocess,sys
sys.path.insert(0,'scripts')
from reliability.custody import snapshot
root=pathlib.Path('/private/tmp/weft-main-reference-20261010');out=pathlib.Path('docs/helix/04-build/evidence/main-integration-20261010');out.mkdir(exist_ok=True)
source=pathlib.Path('docs/helix/04-build/evidence/B-007-qualified-registration/compiled-artifacts.jsonl.gz');records=[json.loads(l) for l in gzip.decompress(source.read_bytes()).decode().splitlines()]
binary=pathlib.Path('/private/tmp/weft-main-reference-target/debug/examples/compile_public_batch');integrated=pathlib.Path('/private/tmp/weft-integration-fresh-20261010/fresh-target/debug/examples/compile_public_batch')
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert not subprocess.check_output(['git','status','--porcelain'],cwd=root)
checkpoint=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
frozen=snapshot(root);inputs=''.join(json.dumps(r['request'],ensure_ascii=False)+'\n' for r in records)
def compile(path):
 p=subprocess.run([str(path)],input=inputs,text=True,capture_output=True,timeout=1800);assert p.returncode==0
 rows=[json.loads(s) for s in p.stdout.splitlines()];assert len(rows)==2181;return rows
current=compile(binary);merged=compile(integrated);assert current==merged
changes=[]
def diff(a,b,path=''):
 if a==b:return []
 if isinstance(a,dict) and isinstance(b,dict):return [x for k in sorted(set(a)|set(b)) for x in diff(a.get(k),b.get(k),path+'/'+k)]
 if isinstance(a,list) and isinstance(b,list) and len(a)==len(b):return [x for i,(v,w) in enumerate(zip(a,b)) for x in diff(v,w,path+'/'+str(i))]
 return [{'path':path,'historical':a,'main':b}]
baseline=[]
for r,response in zip(records,current,strict=True):
 baseline.append({'id':r['id'],'request':r['request'],'response':response})
 differences=diff(r['response'],response)
 if differences:changes.append({'id':r['id'],'differences':differences,'qualification':'Current main compatibility and fresh cross-host parity only; native requalification remains open.'})
assert frozen==snapshot(root)
raw=''.join(json.dumps(r,ensure_ascii=False,separators=(',',':'))+'\n' for r in baseline).encode();(out/'main-reference-artifacts.jsonl.gz').write_bytes(gzip.compress(raw,mtime=0))
(out/'historical-to-main-differences.json.gz').write_bytes(gzip.compress(json.dumps(changes,ensure_ascii=False,separators=(',',':')).encode(),mtime=0))
(out/'main-reference-inputs.json').write_text(json.dumps(frozen,indent=2)+'\n')
receipt={'version':'weft-main-integration-baseline/1','status':'passed','checkpoint':checkpoint,'requests':2181,'requestsUnchanged':True,'historicalIdenticalOutputs':2181-len(changes),'changedOutputs':len(changes),'integratedMainResponseParity':True,'referenceCompilerSha256':sha(binary),'integrationCompilerSha256':sha(integrated),'lockSha256':sha(root/'Cargo.lock'),'toolchain':'Rust/Cargo1.90.0','features':['truss-postgresql-qualified','ashlar-databricks-qualified'],'referenceBuildCommand':['cargo','build','-p','weft-runtime','--example','compile_public_batch','--features','truss-postgresql-qualified,ashlar-databricks-qualified','--locked','--offline'],'referenceSourceClean':True,'generatorSha256':sha(pathlib.Path(__file__)),'historicalArtifactsSha256':sha(source),'baselineSha256':sha(out/'main-reference-artifacts.jsonl.gz'),'differencesSha256':sha(out/'historical-to-main-differences.json.gz'),'sourceInputsSha256':sha(out/'main-reference-inputs.json'),'scope':'Independent clean pinned-main compiler reference for integration compatibility, not an independent native oracle. Unchanged outputs retain byte correspondence to historical native-qualified outputs. Changed outputs do not inherit native qualification. Fresh hosts and hosted CI remain separate gates.'}
(out/'baseline.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps(receipt));print(sorted({d['path'] for r in changes for d in r['differences']}))
