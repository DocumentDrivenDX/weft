from pathlib import Path
import subprocess,os,json,hashlib
root=Path('/private/tmp/weft-graph-requirements-core-owner-20261016');p=root/'crates/weft-core/src/current_presence_graph_requirements.rs';original=p.read_bytes()
env=os.environ.copy();env.update(PATH='/Users/erik/.rustup/toolchains/1.90.0-aarch64-apple-darwin/bin:/opt/homebrew/bin:'+env['PATH'],CARGO_HOME='/private/tmp/ashlar-weft-compatible-toolchain-20261013-a/cargo',CARGO_TARGET_DIR='/private/tmp/weft-graph-requirements-core-target-20261016',CARGO_BUILD_JOBS='1')
rows=[]
try:
 for name,old,new,test in [
 ('detail','self.detail_budget.copy_json(v).map_err(|_|fail())','{let _=v;Ok(())}','intermediate_detail_budget_refuses_before_retaining_match_ordinal'),
 ('key','self.key_budget.copy_json(&key).map_err(|_|fail())?;','/* mutation: missing key charge */','intermediate_key_budget_refuses_before_retaining_new_cache_entry')]:
  s=original.decode();assert s.count(old)==1;p.write_text(s.replace(old,new));log=Path('/private/tmp/weft-graph-budget-mutant-'+name+'-20261016.log')
  with log.open('w') as out:r=subprocess.run(['cargo','test','--offline','-p','weft-core','--features','current-presence-graph-requirements-candidate','--lib',test],cwd=root,env=env,stdout=out,stderr=subprocess.STDOUT)
  body=log.read_text();assert r.returncode==101 and 'test result: FAILED' in body and test+' ... FAILED' in body
  rows.append({'mutant':name,'test':test,'exitCode':r.returncode,'killed':True,'log':str(log)});print(name+' killed',flush=True)
  p.write_bytes(original)
finally:p.write_bytes(original)
Path('/private/tmp/weft-graph-budget-mutations-20261016.json').write_text(json.dumps({'sourceSha256':hashlib.sha256(original).hexdigest(),'sourceRestored':p.read_bytes()==original,'results':rows},indent=2)+'\n')
