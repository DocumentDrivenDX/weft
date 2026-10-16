"""Fresh source-bound candidate/standard wheel and Chromium WASM cohort.
No database is executed and no host permission is inferred.
"""
import argparse,hashlib,json,os,pathlib,shutil,signal,subprocess,time,tomllib
p=argparse.ArgumentParser()
for name in ['cargo','cargo-home','python','maturin','bun','wasm-bindgen','playwright','target','output']:p.add_argument('--'+name,type=pathlib.Path,required=True)
a=p.parse_args();root=pathlib.Path(__file__).resolve().parents[2];out=a.output.resolve();target=a.target.resolve()
assert not out.is_relative_to(root) and not target.is_relative_to(root)
out.mkdir(parents=True,exist_ok=False)
env={**os.environ,'PATH':str(a.cargo.parent)+':'+os.environ.get('PATH',''),'RUSTC':str(a.cargo.parent/'rustc'),'CARGO_HOME':str(a.cargo_home),'CARGO_TARGET_DIR':str(target)}
records=[]
def sources():
 paths={q for folder in ['crates','tests','spec','vendor','docs/helix/02-design/contracts'] for q in (root/folder).rglob('*')if q.is_file()and 'target'not in q.parts and '__pycache__'not in q.parts}
 paths.update(q for q in [root/'Cargo.toml',root/'Cargo.lock']if q.is_file());paths.update(q for q in(root/'.cargo').rglob('*')if q.is_file())
 return{str(q.relative_to(root)):hashlib.sha256(q.read_bytes()).hexdigest()for q in sorted(paths)}
def run(label,command,cwd=root,timeout=600):
 command=list(map(str,command));start=time.monotonic()
 with(out/(label+'.stdout')).open('w')as stdout,(out/(label+'.stderr')).open('w')as stderr:
  process=subprocess.Popen(command,cwd=cwd,env=env,stdout=stdout,stderr=stderr,start_new_session=True)
  timed_out=False
  try:code=process.wait(timeout=timeout)
  except subprocess.TimeoutExpired:
   timed_out=True
   try:os.killpg(process.pid,signal.SIGTERM)
   except ProcessLookupError:pass
   try:code=process.wait(timeout=10)
   except subprocess.TimeoutExpired:
    try:os.killpg(process.pid,signal.SIGKILL)
    except ProcessLookupError:pass
    code=process.wait(timeout=10)
 records.append(dict(id=label,command=command,cwd=str(cwd),exitCode=code,timedOut=timed_out,seconds=time.monotonic()-start));assert code==0 and not timed_out,label
 return(out/(label+'.stdout')).read_text().strip()
def fingerprint(path):return hashlib.sha256(path.read_bytes()).hexdigest()
before=sources();status='failed';failure=None;versions={};wheels={};webs={}
try:
 versions={}
 for name,command,prefix in [('cargo',[a.cargo,'--version'],'cargo 1.90.0'),('rustc',[a.cargo.parent/'rustc','--version'],'rustc 1.90.0'),('python',[a.python,'--version'],'Python 3.12.14'),('maturin',[a.maturin,'--version'],'maturin 1.9.6'),('bun',[a.bun,'--version'],'1.4.2'),('bindgen',[a.wasm_bindgen,'--version'],'wasm-bindgen 0.2.105')]:
  versions[name]=run('version-'+name,command);assert versions[name].startswith(prefix),(name,versions[name])
 playwright_package=a.playwright.parent/'package.json';versions['playwright']=json.loads(playwright_package.read_text())['version'];assert versions['playwright']=='1.62.1'
 versions['playwrightPackageSha256']=fingerprint(playwright_package)
 baseline=tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text())['tool']['maturin']['features']
 assert 'bounded-current-string-candidate'not in baseline
 wheels={};webs={}
 for bounded in [True,False]:
  mode='bounded'if bounded else'standard';wheel_out=out/(mode+'-wheels');features=['--features',','.join(baseline+(['bounded-current-string-candidate']if bounded else[]))]
  run(mode+'-wheel-build',[a.maturin,'build','--locked','--offline',*features,'--interpreter',a.python,'--out',wheel_out],root/'crates/weft-python')
  found=list(wheel_out.glob('*.whl'));assert len(found)==1;wheels[mode]=found[0]
  venv=out/(mode+'-python');run(mode+'-venv',[a.python,'-m','venv',venv]);python=venv/'bin/python'
  run(mode+'-wheel-install',[python,'-m','pip','install','--no-index','--no-deps',found[0]])
  if bounded:
   run('bounded-native',[python,root/'tests/current-query-bounded/python-bridge.py','--wheel',found[0],'--output',out/'native-vectors.json'])
  else:
   run('standard-native',[python,root/'tests/current-query-bounded/standard-wheel.py','--wheel',found[0],'--packet',out/'native-vectors.json','--output',out/'standard-native.json'])
  run(mode+'-legacy-refinement',[python,root/'tests/current-query-refinement/wheel-smoke.py','--wheel',found[0],'--output',out/(mode+'-legacy-refinement.json')])
  run(mode+'-legacy-abi',[python,root/'tests/truss-query-abi/wheel-smoke.py'])
  run(mode+'-legacy-designation',[python,root/'tests/current-query-designation/python-bridge.py'])
  feature='bounded-current-string-candidate'if bounded else'application-lineage-candidate'
  run(mode+'-wasm-build',[a.cargo,'build','--locked','--offline','-p','weft-wasm','--features',feature,'--target','wasm32-unknown-unknown'])
  raw=out/(mode+'-raw.wasm');shutil.copy2(target/'wasm32-unknown-unknown/debug/weft_wasm.wasm',raw)
  web=out/(mode+'-web');webs[mode]=web;run(mode+'-bindgen',[a.wasm_bindgen,raw,'--target','web','--out-name','weft_wasm','--out-dir',web])
  run(mode+'-browser',[a.bun,root/'tests/current-query-bounded/browser-bridge.ts',web,out/'native-vectors.json',a.playwright,out/(mode+'-browser.json'),mode])
 status='passed'
except Exception as error:
 failure=type(error).__name__+': '+str(error)
finally:
 after=sources();unchanged=before==after
 if not unchanged:status='failed';failure='Captured source inputs changed during cohort'
 artifacts={}
 for mode in wheels:
  artifacts[mode]={'wheelSha256':fingerprint(wheels[mode])}
  if mode in webs:
   for name in ['weft_wasm.js','weft_wasm_bg.wasm','weft_wasm.d.ts']:
    if(webs[mode]/name).is_file():artifacts[mode][name]=fingerprint(webs[mode]/name)
  raw=out/(mode+'-raw.wasm')
  if raw.exists():artifacts[mode]['rawWasmSha256']=fingerprint(raw)
 receipt=dict(status=status,failure=failure,sourceInputs=before,sourceInputsUnchanged=unchanged,toolVersions=versions,executions=records,artifacts=artifacts,scope='Exact local source-bound compiler/bridge cohort, not a hermetic toolchain, native database admission, multi-platform release or final main integration qualification.')
 (out/'cohort.json').write_text(json.dumps(receipt,indent=2)+'\n')
assert status=='passed',failure
print(json.dumps(dict(status=status,sourceInputs=len(before),sourceInputsUnchanged=unchanged)))
