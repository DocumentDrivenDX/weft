from pathlib import Path
import subprocess,os,json,hashlib
root=Path('/private/tmp/weft-graph-main-bridge-owner-20261016');p=root/'crates/weft-core/src/current_presence_graph_requirements.rs';original=p.read_bytes();old='budget.copy_json(&entry).map_err(|_|fail())?;entries.push(entry);Ok(())';assert original.decode().count(old)==1
env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-graph-requirements-core-target-20261016',CARGO_BUILD_JOBS='1')
try:
 p.write_text(original.decode().replace(old,'let _=budget;entries.push(entry);Ok(())'))
 with open('/private/tmp/weft-graph-output-retention-mutant-20261016.log','w') as out:r=subprocess.run(['cargo','test','--offline','-p','weft-core','--features','current-presence-graph-requirements-candidate','--lib','intermediate_output_budget'],cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT)
 log=Path('/private/tmp/weft-graph-output-retention-mutant-20261016.log').read_text();assert r.returncode==101 and 'test result: FAILED' in log
finally:p.write_bytes(original)
Path('/private/tmp/weft-graph-output-retention-mutant-20261016.json').write_text(json.dumps({'killed':True,'exitCode':r.returncode,'sourceRestored':p.read_bytes()==original,'restoredSourceSha256':hashlib.sha256(original).hexdigest(),'baselineGraphReportSha256':'fea5b5468a1d9e2e79d2a96dd1107777a6a56022dad892de4d4ebd92b24b870a','baselineGraphReportBytes':30718},indent=2)+'\n');print('output retention mutant killed; source restored')
