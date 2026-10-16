from pathlib import Path
import subprocess,json,os
root=Path('/private/tmp/weft-model-requirements-bridge-main-integration-20261016');probe=Path('/private/tmp/weft-model-requirements-main-runtime-feature-probe-20261016');(probe/'src').mkdir(parents=True,exist_ok=True)
(probe/'src/main.rs').write_text('use weft_runtime::CurrentPresencePreparation;\nfn probe(c:&CurrentPresencePreparation){let _=c.model_requirements_json();}\nfn main(){}\n')
env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-model-requirements-bridge-main-target-20261016',CARGO_BUILD_JOBS='1')
rows=[]
for label,features,error in [('default',[],'E0432'),('old-presence',['current-presence-candidate'],'E0599'),('new',['current-presence-model-requirements-candidate'],None)]:
 (probe/'Cargo.toml').write_text('[package]\nname="weft-model-requirements-main-runtime-boundary-probe"\nversion="0.0.0"\nedition="2021"\n[dependencies]\nweft-runtime={path='+json.dumps(str(root/'crates/weft-runtime'))+',default-features=false,features='+json.dumps(features)+'}\n[patch.crates-io]\nahash={path='+json.dumps(str(root/'vendor/ahash'))+'}\n')
 result=subprocess.run(['cargo','check','--offline','--manifest-path',str(probe/'Cargo.toml')],env=env,capture_output=True,text=True)
 log=Path('/private/tmp/weft-model-requirements-main-runtime-feature-'+label+'-20261016.log');log.write_text(result.stdout+result.stderr)
 valid=result.returncode==0 if error is None else result.returncode==101 and 'error['+error+']' in result.stderr and result.stderr.count('error[')==1 and ('no method named `model_requirements_json`' if error=='E0599' else 'unresolved import `weft_runtime::CurrentPresencePreparation`') in result.stderr
 rows.append({'profile':label,'features':features,'exitCode':result.returncode,'expectedError':error,'passed':valid,'log':str(log)})
 if not valid:break
out=Path('/private/tmp/weft-model-requirements-main-runtime-feature-boundary-20261016.json');out.write_text(json.dumps({'version':'weft-model-requirements-main-runtime-feature-probe/0.1','cases':rows},indent=2)+'\n');print(json.dumps(rows));raise SystemExit(0 if len(rows)==3 and all(r['passed'] for r in rows) else 1)
