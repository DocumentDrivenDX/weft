from pathlib import Path
import subprocess,json,os
root=Path('/private/tmp/weft-native-model-requirements-owner-20261016');probe=Path('/private/tmp/weft-model-requirements-feature-probe-20261016');(probe/'src').mkdir(parents=True,exist_ok=True)
(probe/'src/main.rs').write_text('use weft_core::current_presence_model_requirements::CurrentPresenceModelRequirements;\nfn main(){let _:Option<CurrentPresenceModelRequirements>=None;}\n')
env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-model-requirements-target-20261016',CARGO_BUILD_JOBS='1')
rows=[]
for label,features,expect in [('default',[],False),('old-context-only',['current-presence-context-candidate'],False),('explicit-new',['current-presence-model-requirements-candidate'],True)]:
 (probe/'Cargo.toml').write_text('[package]\nname="weft-model-requirements-boundary-probe"\nversion="0.0.0"\nedition="2021"\n[dependencies]\nweft-core={path='+json.dumps(str(root/'crates/weft-core'))+',default-features=false,features='+json.dumps(features)+'}\n[patch.crates-io]\nahash={path='+json.dumps(str(root/'vendor/ahash'))+'}\n')
 result=subprocess.run(['cargo','check','--offline','--manifest-path',str(probe/'Cargo.toml')],env=env,capture_output=True,text=True)
 log=Path('/private/tmp/weft-model-requirements-feature-'+label+'-20261016.log');log.write_text(result.stdout+result.stderr)
 valid=result.returncode==0 if expect else result.returncode==101 and 'error[E0432]' in result.stderr and 'unresolved import `weft_core::current_presence_model_requirements`' in result.stderr and result.stderr.count('error[')==1
 rows.append({'profile':label,'features':features,'exitCode':result.returncode,'expectedPresence':expect,'passed':valid,'log':str(log)})
 if not valid:break
out=Path('/private/tmp/weft-model-requirements-feature-boundary-20261016.json');out.write_text(json.dumps({'version':'weft-model-requirements-feature-probe/0.1','cases':rows},indent=2)+'\n');print(out);print(json.dumps(rows));raise SystemExit(0 if len(rows)==3 and all(r['passed'] for r in rows) else 1)
