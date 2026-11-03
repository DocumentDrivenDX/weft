"""Actual final-R raw-temporal WASM/Node/Chromium compiler component proof."""
from pathlib import Path
import argparse,json,os,subprocess,time,shutil
from verify_current_inputs_0457 import verify,sha,need

def save(p,v):Path(p).write_text(json.dumps(v,indent=2)+'\n')
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);p.add_argument('--preflight-only',action='store_true');a=p.parse_args();out=a.out.resolve();out.mkdir(parents=True,exist_ok=True);root=Path(__file__).resolve().parents[2]
 inputs=verify(root);R=inputs['sourceCommit'];fs='raw-postgresql-reads,'+','.join('weft-runtime/'+x for x in inputs['features'])
 corpus=root/'tests/engine-catalog/temporal-native-495-corpus.json';need(sha(corpus)=='c8a13365a0a479d2156bf765a4718999f8e59a8d77f3f88cfb613e427f917c5e','Rust oracle changed')
 save(out/'inputs.json',dict(**inputs,workflowCommit=os.environ['GITHUB_SHA'],runID=os.environ['GITHUB_RUN_ID'],wasmBuild=dict(target='wasm32-unknown-unknown',features=fs),oracle='14 retained495 Rust COMPILER-ONLY cases; no native database execution'))
 if a.preflight_only:return
 from verify_current_tools_0457 import inspect
 inspect(root,out/'tools',wasm=True)
 rows=[]
 def run(name,args):
  need(verify(root)==inputs,'Source/lock drift before '+name)
  row=dict(name=name,argv=list(map(str,args)),started=time.time());save(out/'current-stage.json',row)
  with (out/(name+'.stdout')).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:r=subprocess.run(row['argv'],cwd=root,stdout=stdout,stderr=stderr)
  row.update(exitCode=r.returncode,finished=time.time());rows.append(row);save(out/'stages.json',rows)
  need(r.returncode==0,'WASM stage failed: '+name)
 run('WASM-metadata',['cargo','metadata','--locked','--format-version','1','--manifest-path','crates/weft-wasm/Cargo.toml','--filter-platform','wasm32-unknown-unknown','--features',fs])
 run('WASM-features',['cargo','tree','--locked','-p','weft-wasm','--target','wasm32-unknown-unknown','--features',fs,'--edges','normal,build,features','--invert','serde_json@1.0.145','--no-dedupe','--format','{p} features=[{f}]'])
 run('wasm-build',['cargo','build','--locked','--release','-p','weft-wasm','--features',fs,'--target','wasm32-unknown-unknown','--message-format=json-render-diagnostics'])
 raw=Path(os.environ['CARGO_TARGET_DIR'])/'wasm32-unknown-unknown/release/weft_wasm.wasm'
 retained=out/'raw-weft_wasm.wasm';shutil.copyfile(raw,retained)
 save(out/'compiled-wasm-retained.json',dict(sourceCommit=R,sourceTree=inputs['sourceTree'],sha256=sha(retained),features=fs,status='compiled; Node/browser not yet completed'))
 for target in ['web','nodejs']:run('bindgen-'+target,['wasm-bindgen',raw,'--target',target,'--out-dir',out/target])
 run('Node14',['node',root/'tests/engine-catalog/temporal-boundary.mjs',out/'nodejs/weft_wasm.js',corpus])
 run('Chromium14',['node',root/'tests/engine-catalog/temporal-browser.mjs',out/'web',corpus])
 need(verify(root)==inputs,'Source/lock drift after WASM execution')
 files=[raw,out/'web/weft_wasm.js',out/'web/weft_wasm_bg.wasm',out/'nodejs/weft_wasm.js',out/'nodejs/weft_wasm_bg.wasm']
 save(out/'wasm-manifest.json',dict(status='passed',producerRunId=int(os.environ['GITHUB_RUN_ID']),producerRunAttempt=int(os.environ['GITHUB_RUN_ATTEMPT']),producerJobName=os.environ['GITHUB_JOB'],sourceCommit=R,sourceTree=inputs['sourceTree'],workflowCommit=R,runID=os.environ['GITHUB_RUN_ID'],canonicalUmf=inputs['canonicalUmf'],cargoLockSha256=inputs['cargoLockSha256'],target='wasm32-unknown-unknown',features=fs,artifacts=[dict(path=str(p),bytes=p.stat().st_size,sha256=sha(p)) for p in files],NodeCases=14,ChromiumCases=14,carrierRefusalsPerBoundary=3,nativeDatabaseExecution=False,publicationAdmission=False))
if __name__=='__main__':main()
