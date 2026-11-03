"""Produce immediate per-platform current SDK evidence from an existing wheel.
No wheel compilation, database execution or publication occurs here.
"""
from pathlib import Path
import argparse,hashlib,json,os,platform,shutil,subprocess,sys,time,zipfile

LOCK='92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d'
S='e3f38ef6f05ae636d2837f9d69f8d6805222664b'
CORPUS='c8a13365a0a479d2156bf765a4718999f8e59a8d77f3f88cfb613e427f917c5e'
def sha(p):
 with Path(p).open('rb') as f:return hashlib.file_digest(f,'sha256').hexdigest()
def save(p,v):Path(p).write_text(json.dumps(v,indent=2)+'\n')
def main():
 p=argparse.ArgumentParser();p.add_argument('--wheel',type=Path,required=True);p.add_argument('--target',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
 if sys.flags.optimize:raise RuntimeError('Current SDK guards require unoptimized Python')
 root=Path(__file__).resolve().parents[2];out=a.out.resolve();out.mkdir(parents=True,exist_ok=True);wheel=a.wheel.resolve()
 R=os.environ['GITHUB_SHA'];tree=subprocess.check_output(['git','rev-parse','HEAD^{tree}'],cwd=root,text=True).strip()
 assert os.environ['GITHUB_REF']=='refs/heads/main' and os.environ['GITHUB_EVENT_NAME']=='workflow_dispatch'
 assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()==R
 assert sha(root/'Cargo.lock')==LOCK
 corpus=root/'tests/engine-catalog/temporal-native-495-corpus.json';assert sha(corpus)==CORPUS
 from verify_current_sdk_0457 import TARGETS
 assert a.target in TARGETS
 expected=('Darwin','arm64') if a.target=='aarch64-apple-darwin' else ('Linux','aarch64' if a.target.startswith('aarch64') else 'x86_64')
 assert (platform.system(),platform.machine())==expected
 destination=out/wheel.name
 if destination!=wheel:shutil.copyfile(wheel,destination)
 inputs=dict(profile='weft-current-sdk-platform-observation/1',status='incomplete',sourceCommit=R,sourceTree=tree,canonicalUmf=S,cargoLockSha256=LOCK,version='0.4.57',distribution='umf-weft',target=a.target,wheel=wheel.name,wheelSha256=sha(wheel),producerRunId=int(os.environ['GITHUB_RUN_ID']),producerRunAttempt=os.environ['GITHUB_RUN_ATTEMPT'],producerJobName=os.environ['GITHUB_JOB'],workflowRef=os.environ['GITHUB_WORKFLOW_REF'],workflowCommit=R,controllerScript='scripts/release/qualify_current_sdk_0457.py',controllerCodeSha256=sha(Path(__file__)),python=sys.version,platform=platform.platform(),nativeAdmission=False,publicationAdmission=False,producerRunConclusion='NOT CLAIMED while producer is running',oracle='14 retained495 Rust compiler-only cases, not native DB execution')
 save(out/'component.json',inputs);rows=[]
 def run(name,args):
  row=dict(name=name,argv=list(map(str,args)),started=time.time());save(out/'current-stage.json',row)
  with (out/(name+'.stdout')).open('wb') as stdout,(out/(name+'.stderr')).open('wb') as stderr:r=subprocess.run(row['argv'],cwd=root,stdout=stdout,stderr=stderr)
  row.update(exitCode=r.returncode,finished=time.time());rows.append(row);save(out/'stages.json',rows)
  if r.returncode:raise RuntimeError('Current platform stage failed: '+name)
 try:
  venv=Path(os.environ['RUNNER_TEMP'])/('current-sdk-installed-'+a.target)
  run('venv',[sys.executable,'-m','venv',venv]);python=venv/'bin/python'
  run('normal-install',[python,'-I','-m','pip','install',wheel])
  run('pip-check',[python,'-I','-m','pip','check'])
  run('packaging',[python,'-I',root/'scripts/release/verify-wheel-packaging.py','--wheel',wheel,'--version','0.4.57','--distribution','umf-weft','--output',out/'packaging.json'])
  run('native-origin',[python,'-I','-c','import weft.weft;print(weft.weft.__file__)'])
  native=Path((out/'native-origin.stdout').read_text().strip())
  run('temporal14',[python,'-I',root/'tests/engine-catalog/temporal-boundary.py',native,corpus])
  run('routing5',[python,'-I',root/'tests/engine-catalog/temporal-routing-boundary.py',native])
  run('sdk42',[python,'-I',root/'tests/engine-catalog/wheel-smoke.py','--wheel',wheel,'--output',out/'sdk-smoke.json'])
  report=json.loads((out/'sdk-smoke.json').read_text());assert report['sourceCommit']==R and report['controls']==42
  inputs.update(status='passed',nativeModuleSha256=sha(native),shimSha256=report['pinsBeforeAfter']['shimSha256'],sdkReportSha256=sha(out/'sdk-smoke.json'),packagingReportSha256=sha(out/'packaging.json'),temporalCases=14,routingControls=5,sdkControls=42)
 finally:
  inputs['stageResults']=rows;save(out/'component.json',inputs)
 return 0
if __name__=='__main__':sys.exit(main())
