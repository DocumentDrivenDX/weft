from pathlib import Path
import subprocess,os
root='/private/tmp/weft-graph-main-bridge-owner-20261016';env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-graph-requirements-core-target-20261016',CARGO_BUILD_JOBS='1')
for label,args in [('runtime',['-p','weft-runtime','--features','current-presence-graph-requirements-candidate','--lib','current_presence']),('core',['-p','weft-core','--features','current-presence-graph-requirements-candidate','--lib'])]:
 with open('/private/tmp/weft-graph-main-final-'+label+'-20261016.log','w') as out:subprocess.run(['cargo','test','--offline']+args,cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT,check=True)
 print(label+' passed',flush=True)
