"""Advance the pinned corpus by one reviewed declaration-only change.

Requests and every response member outside operation languageProfiles remain exact.
This is compatibility evidence; it grants no new native qualification.
"""
import copy,gzip,hashlib,json,pathlib,subprocess,sys
OLD=[{'dialectProfile':'weft-sql/0.1.0','irVersion':'weft-ir/0.1.0'},{'dialectProfile':'weft-sql/0.2.0','irVersion':'weft-ir/0.2.0'}]
ADDITION={'dialectProfile':'weft-sql/0.2.0','irVersion':'weft-ir/engine-navigation/0.1'}
def advance_declarations(response):
 result=copy.deepcopy(response)
 if result.get('backend')!={'backendId':'ashlar.databricks','backendVersion':'0.1.0-qualified','interfaceVersion':'weft-backend/0.2.0','targetProfile':'dbsql2026.39-qualified'}:return result
 for operation in result.get('qualification',{}).get('operations',[]):
  declaration=operation.get('declaration',{})
  if 'languageProfiles' not in declaration:continue
  if declaration['languageProfiles']!=OLD:raise RuntimeError('Unexpected prior declaration profiles')
  declaration['languageProfiles'].append(copy.deepcopy(ADDITION))
 return result

def canonical(value):return json.dumps(value,sort_keys=True,ensure_ascii=False,separators=(',',':'),allow_nan=False)
def diff(a,b,path=''):
 if canonical(a)==canonical(b):return []
 if isinstance(a,dict) and isinstance(b,dict) and a.keys()==b.keys():return [d for k in sorted(a) for d in diff(a[k],b[k],path+'/'+k)]
 if isinstance(a,list) and isinstance(b,list) and len(a)==len(b):return [d for i,(x,y) in enumerate(zip(a,b,strict=True)) for d in diff(x,y,path+'/'+str(i))]
 return [{'path':path,'historical':a,'main':b}]
def main():
 if sys.flags.optimize:raise RuntimeError('Qualification requires nonoptimized Python')
 root,binary,out=map(lambda p:pathlib.Path(p).resolve(),sys.argv[1:])
 if subprocess.check_output(['git','status','--porcelain'],cwd=root):raise RuntimeError('Reference source must be clean')
 checkpoint=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()
 if checkpoint!='2b29257475259d761f6b6cd6cba176cf4020a992':raise RuntimeError('Unexpected reference checkpoint')
 sys.path.insert(0,str(root/'scripts'))
 from reliability.custody import snapshot
 from reliability.fresh_hosts import reconcile_main_baseline
 BASELINE=pathlib.Path('docs/helix/04-build/evidence/main-ed2-integration-20261010/baseline')
 source=root/'docs/helix/04-build/evidence/B-007-qualified-registration/compiled-artifacts.jsonl.gz'
 historical=[json.loads(s) for s in gzip.decompress(source.read_bytes()).decode().splitlines()]
 prior_receipt=json.loads((root/BASELINE/'baseline.json').read_bytes())
 for path,key in [(source,'historicalArtifactsSha256'),(root/BASELINE/'main-reference-artifacts.jsonl.gz','baselineSha256'),(root/BASELINE/'historical-to-main-differences.json.gz','differencesSha256'),(root/BASELINE/'main-reference-inputs.json','sourceInputsSha256'),(root/BASELINE/'baseline-generator.py','generatorSha256')]:
  if hashlib.sha256(path.read_bytes()).hexdigest()!=prior_receipt[key]:raise RuntimeError('Prior baseline custody mismatch')
 prior=[json.loads(s) for s in gzip.decompress((root/BASELINE/'main-reference-artifacts.jsonl.gz').read_bytes()).decode().splitlines()]
 prior,prior_receipt=reconcile_main_baseline(historical,prior,prior_receipt,json.loads(gzip.decompress((root/BASELINE/'historical-to-main-differences.json.gz').read_bytes())))
 frozen=snapshot(root);sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest();binary_sha=sha(binary)
 run=subprocess.run([str(binary)],input=''.join(json.dumps(r['request'],ensure_ascii=False)+'\n' for r in prior),text=True,capture_output=True,timeout=1800,check=True)
 actual=[json.loads(s) for s in run.stdout.splitlines()]
 if len(actual)!=2181:raise RuntimeError('Corpus cardinality changed')
 records=[];reviewed=[];changes=[]
 for old,pinned,response in zip(historical,prior,actual,strict=True):
  expected=advance_declarations(pinned['response'])
  if canonical(expected)!=canonical(response):raise RuntimeError('Unreviewed response change: '+pinned['id'])
  delta=diff(pinned['response'],response)
  if delta:reviewed.append({'id':pinned['id'],'differences':delta})
  records.append({'id':pinned['id'],'request':pinned['request'],'response':response})
  delta=diff(old['response'],response)
  if delta:changes.append({'id':old['id'],'differences':delta,'qualification':'Current main compatibility and fresh cross-host parity only; native requalification remains open.'})
 if len(reviewed)!=964 or sum(len(r['differences']) for r in reviewed)!=5266:raise RuntimeError('Reviewed declaration delta cardinality changed')
 if frozen!=snapshot(root) or binary_sha!=sha(binary):raise RuntimeError('Reference inputs changed')
 out.mkdir(parents=True,exist_ok=True)
 (out/'main-reference-artifacts.jsonl.gz').write_bytes(gzip.compress((''.join(json.dumps(r,ensure_ascii=False,separators=(',',':'))+'\n' for r in records)).encode(),mtime=0))
 (out/'historical-to-main-differences.json.gz').write_bytes(gzip.compress(json.dumps(changes,ensure_ascii=False,separators=(',',':')).encode(),mtime=0))
 (out/'reviewed-declaration-differences.json.gz').write_bytes(gzip.compress(json.dumps(reviewed,ensure_ascii=False,separators=(',',':')).encode(),mtime=0))
 (out/'main-reference-inputs.json').write_text(json.dumps(frozen,indent=2)+'\n')
 receipt={**prior_receipt,'checkpoint':checkpoint,'historicalIdenticalOutputs':2181-len(changes),'changedOutputs':len(changes),'referenceCompilerSha256':binary_sha,'integrationCompilerSha256':binary_sha,'integratedMainResponseParity':True,'independentlyBuiltReference':False,'referenceSourceClean':True,'generatorSha256':sha(pathlib.Path(__file__)),'baselineSha256':sha(out/'main-reference-artifacts.jsonl.gz'),'differencesSha256':sha(out/'historical-to-main-differences.json.gz'),'sourceInputsSha256':sha(out/'main-reference-inputs.json'),'lockSha256':sha(root/'Cargo.lock'),'previousBaselinePath':str(BASELINE),'previousBaselineReceiptSha256':sha(root/BASELINE/'baseline.json'),'reviewedDeclarationCases':964,'reviewedDeclarationChanges':5266,'reviewedDeclarationDifferencesSha256':sha(out/'reviewed-declaration-differences.json.gz'),'scope':'Clean pinned-main compiler compatibility baseline constrained to the exact reviewed additive engine-navigation declaration. Every request and every other response member matches the prior pinned baseline. Not an independent native oracle or independently built second reference; changed outputs do not inherit native qualification. Fresh cross-host qualification and hosted CI remain separate gates.'}
 (out/'baseline.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({k:receipt[k] for k in ['checkpoint','historicalIdenticalOutputs','changedOutputs','reviewedDeclarationCases','reviewedDeclarationChanges']}))
if __name__=='__main__':main()
