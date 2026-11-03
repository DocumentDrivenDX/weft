"""Reviewed controller code executes outside an untouched product-R checkout."""
from pathlib import Path
import argparse,hashlib,json,os,platform,shutil,stat,subprocess,sys,time,tomllib,zipfile
from current_macos_pair_0457 import R,RTREE,REGISTRY,need,validate_payload,validate_parity,MAC_PROFILE,ARM_PROFILE
from admit_controller_pairs_0457 import GitHub,registry,authenticate_pair_code
from admit_current_sdk_0457 import save,sha
CONTROLLER=Path(__file__).resolve().parents[2]
TARGET_MAC='aarch64-apple-darwin';TARGET_ARM='aarch64-unknown-linux-gnu'

def git(root,*args):return subprocess.check_output(['git','-C',str(root),*args],text=True).strip()
def source_snapshot(root,initial=False):
 need(root.is_dir() and not root.is_symlink() and git(root,'rev-parse','HEAD')==R and git(root,'rev-parse','HEAD^{tree}')==RTREE,'Product checkout is not exact R/tree')
 need(not git(root,'diff','--name-only',R),'Product tracked source changed')
 status_rows=git(root,'status','--porcelain','--untracked-files=all','--ignored').splitlines()
 if initial:need(not status_rows,'Product checkout is not initially clean including ignored files')
 else:
  for row in status_rows:
   name=row[3:]
   need(row[:2] in ['??','!!'] and (name.startswith(('target/','dist/','node_modules/','packages/weft-browser/dist/','packages/weft-browser/node_modules/')) or ('__pycache__' in Path(name).parts and name.startswith(('tests/','scripts/')))),'Unexpected product checkout addition')
 raw=subprocess.check_output(['git','-C',str(root),'ls-tree','-rz','--full-tree',R]);rows=[]
 for entry in raw.split(b'\0'):
  if not entry:continue
  header,name=entry.split(b'\t',1);mode,kind,blob=header.decode().split();rel=name.decode();file=root/rel
  need(kind=='blob' and mode in ['100644','100755'],'Unsupported product Git entry')
  at=root
  for part in Path(rel).parts:
   at=at/part;need(not at.is_symlink(),'Product source symlink refused')
  st=file.stat();need(stat.S_ISREG(st.st_mode) and bool(st.st_mode&0o111)==(mode=='100755'),'Product source file/type/mode differs')
  h=hashlib.sha1(b'blob '+str(st.st_size).encode()+b'\0')
  with file.open('rb') as f:
   for chunk in iter(lambda:f.read(1024*1024),b''):h.update(chunk)
  need(h.hexdigest()==blob,'Product source byte differs: '+rel);rows.append([rel,mode,blob])
 return dict(sourceCommit=R,sourceTree=RTREE,trackedFiles=len(rows),sourceByteModeInventorySha256=hashlib.sha256(json.dumps(rows,separators=(',',':')).encode()).hexdigest())

def bounded(root,out,name,seconds,args,*,grace=30,reserve=60,env=None):
 cmd=[sys.executable,str(CONTROLLER/'scripts/release/run_bounded_0454.py'),'--stage',name,'--seconds',str(seconds),'--grace-seconds',str(grace),'--reserve-seconds',str(reserve),'--log',str(out/(name+'.log')),'--receipt',str(out/(name+'-command.json')),'--',*map(str,args)]
 subprocess.run(cmd,cwd=root,check=True,env=env)

def preflight(root,out,target,registration_commit):
 need((platform.system(),platform.machine())==(('Darwin','arm64') if target==TARGET_MAC else ('Linux','aarch64')),'Actual runner platform differs')
 need(os.environ['GITHUB_EVENT_NAME']=='workflow_dispatch' and os.environ['GITHUB_REPOSITORY']=='DocumentDrivenDX/weft','Wrong event/repository')
 api=out.parent/'registration-api';api.mkdir(exist_ok=False);c=GitHub(api);enrollment,main=registry(c,registration_commit);pair=enrollment['macos' if target==TARGET_MAC else 'arm64'];Q=pair['controllerCommit']
 need(os.environ['GITHUB_SHA']==Q and os.environ['GITHUB_REF']=='refs/heads/'+pair['controllerRef'] and git(CONTROLLER,'rev-parse','HEAD')==Q and git(CONTROLLER,'rev-parse','HEAD^{tree}')==pair['controllerTree'],'Actual controller checkout/head/ref differs from registered Q')
 need(not git(CONTROLLER,'status','--porcelain','--untracked-files=all'),'Controller checkout dirty')
 proof=authenticate_pair_code(c,pair,target);before=source_snapshot(root,initial=True);save(out/'product-source-before.json',before);save(out/'controller-closure.json',proof);save(out/'pair-registration.json',enrollment);save(out/'registration-main.json',main)
 component=dict(profile=pair['profile'],status='incomplete',sourceCommit=R,sourceTree=RTREE,controllerCommit=Q,controllerTree=pair['controllerTree'],workflowCommit=Q,producerRunId=int(os.environ['GITHUB_RUN_ID']),producerRunAttempt=int(os.environ['GITHUB_RUN_ATTEMPT']),producerJobName=os.environ['GITHUB_JOB'],target=target,cargoLockSha256='92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d',canonicalUmf='e3f38ef6f05ae636d2837f9d69f8d6805222664b',nativeAdmission=False,publicationAdmission=False)
 save(out/'component.json',component)

def runner(root,out):
 env=dict(os.environ,GITHUB_WORKSPACE=str(root),SOURCE_SHA=R,SOURCE_VERSION='0.4.57',EVIDENCE_ROOT=str(out))
 bounded(root,out,'runner-preflight',45,[sys.executable,CONTROLLER/'scripts/release/record_preflight_0454.py'],grace=10,reserve=0,env=env)
 p=json.loads((out/'runner-preflight.json').read_text());p.update(profile='weft-macos-pair-toolchain/1',sourceTree=RTREE,controllerTree=git(CONTROLLER,'rev-parse','HEAD^{tree}'),distribution='umf-weft');save(out/'runner-preflight.json',p)

def tools(root,out):
 env=dict(os.environ,GITHUB_WORKSPACE=str(root),EVIDENCE_ROOT=str(out))
 bounded(root,out,'verify-toolchain',45,[sys.executable,CONTROLLER/'scripts/release/verify_toolchain_0454.py'],grace=10,reserve=0,env=env)
 wasm_tools(root,out)

def wasm_tools(root,out):
 from verify_current_tools_0457 import inspect
 inspect(root,out/'current-tools',wasm=True)

def build_mac(root,out):
 env=os.environ.copy();env['PYO3_PYTHON']=sys.executable
 # Environment selects the actual Python; it never changes GitHub identities.
 os.environ['PYO3_PYTHON']=sys.executable
 bounded(root/'crates/weft-python',out,'macos-wheel-build',1800,['maturin','build','--release','--locked','--target',TARGET_MAC,'--out',out])
 wheels=list(out.glob('*.whl'));need(len(wheels)==1,'Exactly one ARM64 wheel required')
 bounded(root,out,'macos-wheel-venv',30,[sys.executable,'-m','venv',out.parent/'packaging-venv'],grace=10,reserve=0)
 python=out.parent/'packaging-venv/bin/python'
 bounded(root,out,'macos-wheel-install',180,[python,'-m','pip','install','--no-deps',wheels[0]],grace=10,reserve=0)
 bounded(root,out,'macos-wheel-check',30,[python,CONTROLLER/'scripts/release/verify-wheel-packaging.py','--wheel',wheels[0],'--version','0.4.57','--distribution','umf-weft','--output',out/'packaging-macos-arm64.json'],grace=10,reserve=0)
 p=json.loads((out/'packaging-macos-arm64.json').read_text())
 with zipfile.ZipFile(wheels[0]) as z:modules=[n for n in z.namelist() if n.endswith(('.so','.pyd','.dylib'))]
 need(len(modules)==1,'Unique Mac native module required');p.update(sourceCommit=R,distribution='umf-weft',target=TARGET_MAC,nativeModule=modules[0]);save(out/'packaging-macos-arm64.json',p)

def sdk(root,out,target):
 wheels=list(out.glob('*.whl'));need(len(wheels)==1,'One actual wheel required');wheel=wheels[0];rows=[]
 def run(name,args,seconds):
  before=time.time();bounded(root,out,'sdk-'+name,seconds,args,grace=10,reserve=60);row=json.loads((out/('sdk-'+name+'-command.json')).read_text());rows.append(dict(name=name,argv=list(map(str,args)),exitCode=row['exitCode'],started=before,finished=time.time()));save(out/'sdk-stages.json',rows)
 venv=out.parent/'sdk-installed';run('venv',[sys.executable,'-m','venv',venv],30);python=venv/'bin/python'
 run('normal-install',[python,'-I','-m','pip','install','--report',out/'normal-install-report.json',wheel],180)
 run('pip-check',[python,'-I','-m','pip','check'],30)
 run('packaging',[python,'-I',CONTROLLER/'scripts/release/verify-wheel-packaging.py','--wheel',wheel,'--version','0.4.57','--distribution','umf-weft','--output',out/'packaging.json'],30)
 # Bounded origin probe output is a private host path, not product API data.
 run('native-origin',[python,'-I','-c','import weft.weft;print(weft.weft.__file__)'],30)
 native_log=out/'sdk-native-origin.log';native=Path(native_log.read_text().strip());need(native.is_file(),'Installed native module absent')
 run('temporal14',[python,'-I',root/'tests/engine-catalog/temporal-boundary.py',native,root/'tests/engine-catalog/temporal-native-495-corpus.json'],120)
 run('routing5',[python,'-I',root/'tests/engine-catalog/temporal-routing-boundary.py',native],120)
 run('sdk42',[python,'-I',root/'tests/engine-catalog/wheel-smoke.py','--wheel',wheel,'--output',out/'sdk-smoke.json'],180)
 files={p.name:p.read_bytes() for p in [wheel,out/'sdk-smoke.json',out/'packaging.json']};observed=validate_payload(target,files);component=json.loads((out/'component.json').read_text());component.update(observed,sdkStages=rows,status='SDK-passed-parity-pending' if target==TARGET_MAC else 'SDK-passed-finalization-pending');save(out/'component.json',component)

def vectors(root,out):
 from finalize_hosted_macos import CASE_IDS
 corpus=root/'tests/compile/fixtures/cases.json';cases=json.loads(corpus.read_text());by_id={c['id']:c for c in cases};need(len(by_id)==len(cases) and all(i in by_id for i in CASE_IDS),'Frozen vector roster absent/duplicate');selected=[by_id[i] for i in CASE_IDS];need(all(v['expected'].get('status')=='blocked' for v in selected),'Non-refusal vector')
 save(out/'compiler-cases.json',selected);save(out/'compiler-roster.json',dict(profile='nine-frozen-refusal-responses/1',cases=CASE_IDS,corpus='tests/compile/fixtures/cases.json',corpusSha256=sha(corpus.read_bytes()),selectedCasesSha256=sha((out/'compiler-cases.json').read_bytes()),expected='exact Rust comparator response bytes; installed Python wheel and Chromium must match all nine ordered refusal responses',positiveQueryQualification=False,nativeDatabaseExecution=False))

def parity(root,out):
 features=','.join(tomllib.loads((root/'crates/weft-python/pyproject.toml').read_text())['tool']['maturin']['features']);target=Path(os.environ.get('CARGO_TARGET_DIR',root/'target'))
 bounded(root,out,'rust-comparator',600,['cargo','+1.90.0','build','-p','weft-runtime','--example','compile_public_batch','--features',features,'--locked'])
 fs=','.join('weft-runtime/'+x for x in features.split(','));bounded(root,out,'wasm-build',720,['cargo','+1.90.0','build','-p','weft-wasm','--features',fs,'--target','wasm32-unknown-unknown','--locked','--release'])
 bounded(root,out,'wasm-bindgen',60,['wasm-bindgen',target/'wasm32-unknown-unknown/release/weft_wasm.wasm','--target','web','--out-dir',out/'web'],grace=10,reserve=0)
 bounded(root,out,'bun-install',120,['bun','install','--frozen-lockfile'],grace=10,reserve=0);bounded(root,out,'browser-build',120,['bun','run','browser:build'],grace=10,reserve=0)
 bounded(root,out,'rust-compiler-vectors',60,[sys.executable,CONTROLLER/'scripts/release/compare_selected_rust_0454.py','--cases',out/'compiler-cases.json','--executable',target/'debug/examples/compile_public_batch','--output',out/'compiler-reports.json'],grace=10,reserve=0)
 bounded(root,out,'python-wheel-vectors',120,[out.parent/'packaging-venv/bin/python',CONTROLLER/'scripts/release/compare_selected_python.py','--cases',out/'compiler-cases.json','--reports',out/'compiler-reports.json','--output',out/'python-compiler-parity.json','--source-root',root,'--version','0.4.57','--distribution','umf-weft'],grace=10,reserve=0)
 pw=out.parent/'playwright';bounded(root,out,'playwright-install',300,['npm','install','--prefix',pw,'--ignore-scripts','--no-package-lock','playwright@1.62.1'],grace=10,reserve=0);bounded(root,out,'chromium-install',600,['node',pw/'node_modules/playwright/cli.js','install','chromium'],grace=10,reserve=0)
 env=dict(WEFT_PLAYWRIGHT_MODULE=str(pw/'node_modules/playwright/index.mjs'),WEFT_PROBE_API='compile_json',WEFT_PROBE_JS=str(out/'web/weft_wasm.js'),WEFT_PROBE_WASM=str(out/'web/weft_wasm_bg.wasm'),WEFT_BROWSER_WRAPPER=str(root/'packages/weft-browser/dist/index.js'),WEFT_FRONTEND_CORPUS=str(out/'compiler-cases.json'),WEFT_FRONTEND_REPORTS=str(out/'compiler-reports.json'),WEFT_FRONTEND_BROWSER_SUMMARY=str(out/'browser-compiler-parity.json'),WEFT_BROWSER_RECEIPTS=str(out/'browser-compiler-receipts.json'))
 os.environ.update(env);bounded(root,out,'browser-parity',300,['bun',root/'tests/compile/browser-check.mjs'],grace=10,reserve=0)
 files={p.name:p.read_bytes() for p in out.iterdir() if p.is_file()};c=json.loads((out/'component.json').read_text());c['parity']=validate_parity(files,c);save(out/'component.json',c)

def finalize(root,out,target):
 before=json.loads((out/'product-source-before.json').read_text());after=source_snapshot(root);need(before==after,'Product bytes/modes changed');save(out/'product-source-after.json',after)
 c=json.loads((out/'component.json').read_text());need(c['status'].startswith('SDK-passed'),'SDK not passed')
 if target==TARGET_MAC:need('parity' in c,'Mac9 refusal parity absent')
 else:c['parity']={'scope':'original ARM platform has no separate nine-refusal Mac parity; shared raw temporal Node/Chromium proof remains separately required'}
 c['status']='passed';save(out/'component.json',c)

def retain(out,destination):
 # Preserve original hosted Mac per-file/aggregate caps. Failure never uploads
 # an oversized payload or turns a preflight-only partial into qualification.
 need(not destination.exists(),'Fresh bounded upload directory required');destination.mkdir(parents=True);rows=[];withheld=[]
 for f in sorted(out.rglob('*')):
  need(not f.is_symlink(),'Artifact symlink refused')
  if f.is_dir():continue
  need(stat.S_ISREG(f.stat().st_mode),'Artifact special file refused')
  rel=f.relative_to(out).as_posix();limit=60*1024*1024 if f.suffix=='.whl' else 20*1024*1024 if rel=='web/weft_wasm_bg.wasm' else 5*1024*1024 if rel=='web/weft_wasm.js' else 2*1024*1024
  size=f.stat().st_size
  with f.open('rb') as source:actual_hash=hashlib.file_digest(source,'sha256').hexdigest() if size<=limit else None
  row=dict(path=rel,bytes=size,sha256=actual_hash)
  (rows if row['bytes']<=limit else withheld).append(row)
 if sum(r['bytes'] for r in rows)>70*1024*1024 or len(rows)>160:withheld+=rows;rows=[]
 for row in rows:
  dest=destination/row['path'];dest.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(out/row['path'],dest)
 save(destination/'upload-inventory.json',dict(files=rows,withheld=withheld,qualification=False if withheld else 'consult actual component and job'))
 if withheld:
  c=json.loads((out/'component.json').read_text()) if (out/'component.json').exists() else {};c.update(status='failed',withheldEvidence=withheld);save(destination/'component.json',c);raise ValueError('Original bounded artifact retention caps exceeded')

def main():
 p=argparse.ArgumentParser();p.add_argument('--phase',choices=['preflight','runner','tools','wasm-tools','build','sdk','vectors','parity','finalize','retain'],required=True);p.add_argument('--product-root',type=Path,required=True);p.add_argument('--out',type=Path,required=True);p.add_argument('--target',choices=[TARGET_MAC,TARGET_ARM],required=True);p.add_argument('--registration-sha');a=p.parse_args();root=a.product_root.resolve();out=a.out.resolve();need(not out.is_relative_to(root) and not CONTROLLER.is_relative_to(root),'Controller/output must be outside product R');out.mkdir(parents=True,exist_ok=True)
 try:
  if a.phase=='preflight':preflight(root,out,a.target,a.registration_sha)
  elif a.phase=='runner':need(a.target==TARGET_MAC,'Mac-only runner/toolchain record');runner(root,out)
  elif a.phase=='tools':need(a.target==TARGET_MAC,'Mac-only toolchain');tools(root,out)
  elif a.phase=='wasm-tools':need(a.target==TARGET_MAC,'Mac-only WASM tools');wasm_tools(root,out)
  elif a.phase=='build':need(a.target==TARGET_MAC,'ARM build requires its separate acquisition controller');build_mac(root,out)
  elif a.phase=='sdk':sdk(root,out,a.target)
  elif a.phase=='vectors':need(a.target==TARGET_MAC,'Mac-only vectors');vectors(root,out)
  elif a.phase=='parity':need(a.target==TARGET_MAC,'Mac-only9 parity');parity(root,out)
  elif a.phase=='retain':retain(out,out.parent/'upload')
  else:finalize(root,out,a.target)
 except Exception as error:
  save(out/'phase-failure.json',dict(status='failed',phase=a.phase,errorType=type(error).__name__,message=str(error),sourceCommit=R,workflowCommit=os.environ.get('GITHUB_SHA'),nativeAdmission=False,publicationAdmission=False));raise

if __name__=='__main__':main()
