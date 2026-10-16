from pathlib import Path
import subprocess,os,zipfile,json,hashlib
root=Path('/private/tmp/weft-graph-main-bridge-owner-20261016');env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-graph-requirements-core-target-20261016',CARGO_BUILD_JOBS='1',PYO3_PYTHON='/private/tmp/truss-practical-performance-env/bin/python',MACOSX_DEPLOYMENT_TARGET='11.0')
def run(label,args):
 with open('/private/tmp/weft-graph-output-final-'+label+'-20261016.log','w') as out:subprocess.run(args,cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
 print(label+' passed',flush=True)
run('core',['cargo','test','--locked','--offline','-p','weft-core','--features','current-presence-graph-requirements-candidate','--lib'])
run('docs',['cargo','test','--locked','--offline','-p','weft-core','--features','current-presence-graph-requirements-candidate','--doc','current_presence_graph_requirements'])
run('runtime',['cargo','test','--locked','--offline','-p','weft-runtime','--features','current-presence-graph-requirements-candidate','--lib','current_presence'])
out=Path('/private/tmp/weft-graph-output-final-wheels-20261016')
run('wheel',['/private/tmp/truss-clean-install-20261013/bin/maturin','build','--locked','--offline','--profile','dev','--manifest-path','crates/weft-python/Cargo.toml','--no-default-features','--features','current-presence-graph-requirements-candidate','--out',str(out)])
wheels=list(out.glob('*.whl'));assert len(wheels)==1;wheel=wheels[0];target=Path('/private/tmp/weft-graph-output-final-installed-20261016');target.mkdir(exist_ok=False)
with zipfile.ZipFile(wheel) as z:z.extractall(target)
run('python',['/private/tmp/truss-practical-performance-env/bin/python','tests/current-query-presence/python-graph-requirements-bridge.py',str(target),'graph'])
run('existing-model',['/private/tmp/truss-practical-performance-env/bin/python','tests/current-query-presence/python-model-requirements-bridge.py',str(target),'candidate','/private/tmp/weft-graph-output-final-model-receipt-20261016.json'])
Path('/private/tmp/weft-graph-output-final-receipt-20261016.json').write_text(json.dumps({'wheel':str(wheel),'sha256':hashlib.sha256(wheel.read_bytes()).hexdigest(),'nativeAdmission':False,'sourceRestored':json.loads(Path('/private/tmp/weft-graph-output-retention-mutant-20261016.json').read_text())['sourceRestored']},indent=2)+'\n')
