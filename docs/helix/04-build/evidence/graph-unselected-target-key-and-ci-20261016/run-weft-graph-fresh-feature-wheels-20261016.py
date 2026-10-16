from pathlib import Path
import os,subprocess,zipfile,json,hashlib
root=Path('/private/tmp/weft-graph-main-bridge-owner-20261016');env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-graph-requirements-core-target-20261016',CARGO_BUILD_JOBS='1',PYO3_PYTHON='/private/tmp/truss-practical-performance-env/bin/python',MACOSX_DEPLOYMENT_TARGET='11.0')
rows=[]
for mode,feature in [('default',None),('old-model','current-presence-model-requirements-candidate'),('graph','current-presence-graph-requirements-candidate')]:
 out=Path('/private/tmp/weft-graph-fresh-feature-wheels-20261016')/mode
 args=['/private/tmp/truss-clean-install-20261013/bin/maturin','build','--locked','--offline','--profile','dev','--manifest-path','crates/weft-python/Cargo.toml','--no-default-features','--out',str(out)]
 if feature:args+=['--features',feature]
 log=Path('/private/tmp/weft-graph-fresh-'+mode+'-build-20261016.log')
 with log.open('w') as stream:subprocess.run(args,cwd=root,env=env,stdout=stream,stderr=subprocess.STDOUT,check=True)
 wheels=list(out.glob('*.whl'));assert len(wheels)==1;w=wheels[0]
 target=Path('/private/tmp/weft-graph-fresh-feature-installed-20261016')/mode;target.mkdir(parents=True,exist_ok=False)
 with zipfile.ZipFile(w) as z:z.extractall(target)
 result=subprocess.run(['/private/tmp/truss-practical-performance-env/bin/python','tests/current-query-presence/python-graph-requirements-bridge.py',str(target),mode],cwd=root,capture_output=True,text=True)
 testlog=Path('/private/tmp/weft-graph-fresh-'+mode+'-tests-20261016.log');testlog.write_text(result.stdout+result.stderr);assert result.returncode==0,testlog
 rows.append({'mode':mode,'wheel':str(w),'sha256':hashlib.sha256(w.read_bytes()).hexdigest(),'test':json.loads(result.stdout)})
 print(mode+' build/test passed',flush=True)
Path('/private/tmp/weft-graph-fresh-feature-wheels-20261016.json').write_text(json.dumps(rows,indent=2)+'\n')
