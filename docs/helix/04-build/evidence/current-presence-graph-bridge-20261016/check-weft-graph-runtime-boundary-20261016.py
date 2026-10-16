from pathlib import Path
import subprocess,os,json
root=Path('/private/tmp/weft-graph-main-bridge-owner-20261016');probe=Path('/private/tmp/weft-graph-runtime-probe-20261016');(probe/'src').mkdir(parents=True,exist_ok=True)
(probe/'src/main.rs').write_text('use weft_runtime::CurrentPresencePreparation;\nfn use_graph(c:&CurrentPresencePreparation){let _=c.graph_requirements_json();}\nfn main(){}\n')
env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-graph-requirements-core-target-20261016',CARGO_BUILD_JOBS='1')
rows=[]
for label,features,code in [('default',[],'E0432'),('old-model',['current-presence-model-requirements-candidate'],'E0599'),('graph',['current-presence-graph-requirements-candidate'],None)]:
 (probe/'Cargo.toml').write_text('[package]\nname="weft-graph-runtime-probe"\nversion="0.0.0"\nedition="2021"\n[dependencies]\nweft-runtime={path='+json.dumps(str(root/'crates/weft-runtime'))+',default-features=false,features='+json.dumps(features)+'}\n[patch.crates-io]\nahash={path='+json.dumps(str(root/'vendor/ahash'))+'}\n')
 r=subprocess.run(['cargo','check','--offline','--manifest-path',str(probe/'Cargo.toml')],env=env,capture_output=True,text=True)
 log=Path('/private/tmp/weft-graph-runtime-feature-'+label+'-20261016.log');log.write_text(r.stdout+r.stderr)
 valid=(r.returncode==0) if code is None else r.returncode==101 and 'error['+code+']' in r.stderr and r.stderr.count('error[')==1
 rows.append({'profile':label,'exitCode':r.returncode,'passed':valid,'log':str(log)});assert valid,rows[-1]
Path('/private/tmp/weft-graph-runtime-boundary-20261016.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows))
