"""One public Linux consumer; consumes original P publication, never replays it."""
from pathlib import Path
from urllib.parse import urlparse,unquote
from datetime import datetime
import argparse,hashlib,json,os,platform,re,subprocess,sys,time,urllib.request,zipfile,io
from admit_current_sdk_0457 import GitHub,PREFIX,collection,archive_files
from publish_hosted_release import verify_release_api,api_tag_commit
ROOT=Path(__file__).resolve().parents[2]
WORKFLOW='.github/workflows/public-postflight-0457.yml'
R='c623b94b66b0b46d492d457c62faddc896e2e452'
P='fcd6e1d99ef05ad64dc07da27c98210e808b0b89'
def need(ok,message):
 if not ok:raise ValueError(message)
def sha(raw):return hashlib.sha256(raw).hexdigest()
def canonical(value):return json.dumps(value,sort_keys=True,separators=(',',':')).encode()
def save(path,value):Path(path).write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def public_url(value):
 p=urlparse(value);need(p.scheme=='https' and p.hostname=='files.pythonhosted.org' and p.port in (None,443) and not p.username and not p.password and not p.fragment,'Non-public file URL');return value

def run_record(value,run_id,head,path,*,terminal=True):
 need(value.get('id')==run_id and value.get('head_sha')==head and value.get('path')==path and value.get('repository',{}).get('full_name')=='DocumentDrivenDX/weft','Actual run repository/head/path/id differs')
 if terminal:need(value.get('status')=='completed' and value.get('conclusion')=='success','Original run not terminal/successful')
 return value

def publication_receipts(publication,source,expected):
 for row in [publication,source]:
  need(row.get('tag')=='v0.4.57' and row.get('source_sha')==R and row.get('distribution')=='umf-weft' and row.get('version')=='0.4.57' and row.get('files')==expected['archives'],'Original publication identity/files differ')
 need(source.get('schema')=='pypi-release-source.v1' and source.get('status')=='verified' and source.get('repository')=='DocumentDrivenDX/weft','Original verified source receipt absent')
 need(publication.get('schema')=='pypi-publication-receipt.v1' and publication.get('status')=='passed' and publication.get('registry')=='pypi' and publication.get('registry_bytes_verified') is True,'Original terminal PyPI byte proof absent')
 mode=publication.get('publication_mode');need((mode=='oidc_upload' and publication.get('oidc_trusted_publisher') is True) or (mode=='existing_exact_bytes' and publication.get('oidc_trusted_publisher') is False),'Publication mode/OIDC claim differs')

def selected_public_wheel(report,expected):
 rows=report.get('install');need(type(rows) is list and len(rows)==1,'Unexpected resolver dependency closure')
 row=rows[0];m=row.get('metadata',{});need(m.get('name')=='umf-weft' and m.get('version')=='0.4.57' and m.get('requires_dist',[])==[] and row.get('requested') is True,'Public selected distribution differs')
 d=row.get('download_info',{});url=public_url(d.get('url',''));name=unquote(Path(urlparse(url).path).name)
 need(name in expected['wheels'] and expected['wheels'][name]['target']=='x86_64-unknown-linux-gnu','Wrong public platform wheel')
 need(d.get('archive_info',{}).get('hashes',{}).get('sha256')==expected['archives'][name],'Public selected wheel hash differs')
 return name,url

def public_inventory(value,expected):
 need(value.get('info',{}).get('name')=='umf-weft' and value['info'].get('version')=='0.4.57','Public index identity differs')
 rows=value.get('urls');need(type(rows) is list,'Public index files absent');seen={}
 for row in rows:
  name=row.get('filename');need(type(name) is str and name not in seen,'Duplicate public archive');public_url(row.get('url',''));need(row.get('yanked') is False,'Yanked public archive');seen[name]=row.get('digests',{}).get('sha256')
 need(seen==expected['archives'],'Public archive inventory/digests differ');return rows

def installed_binding(record,wheel,venv):
 need(record.get('name')=='umf-weft' and record.get('version')=='0.4.57','Installed distribution differs')
 need(record.get('requiresDist')==[],'Installed dependency metadata differs')
 owned=record.get('ownedFiles');need(type(owned) is list and len(owned)==len(set(owned)),'Distribution ownership inventory differs')
 prefix=Path(venv).resolve()
 for role,key in [('native','nativeModuleSha256'),('shim','shimSha256')]:
  path=Path(record.get(role+'Path',''));need(path.is_absolute() and path.is_relative_to(prefix) and str(path) in owned,'Imported file is not owned by fresh distribution')
  need(record.get(role+'Sha256')==wheel[key],'Installed '+role+' hash differs')
 return record

def main(args):
 need(sys.flags.optimize==0 and platform.system()=='Linux' and platform.machine()=='x86_64','One native Linux x86 public consumer required')
 need(os.environ.get('GITHUB_EVENT_NAME')=='workflow_dispatch' and os.environ.get('GITHUB_REPOSITORY')=='DocumentDrivenDX/weft','Manual canonical verification only')
 need(os.environ.get('GITHUB_REF')=='refs/heads/main','Reviewed default-branch V dispatch required')
 controller=os.environ.get('GITHUB_SHA');need(controller==args.controller_sha and re.fullmatch('[0-9a-f]{40}',controller or '') is not None,'Reviewed V controller differs')
 need(subprocess.check_output(['git','-C',str(ROOT),'rev-parse','HEAD']).decode().strip()==controller,'Actual controller checkout differs')
 need(not subprocess.check_output(['git','-C',str(ROOT),'status','--porcelain','--untracked-files=all']).strip(),'Dirty controller checkout')
 out=args.out.resolve();need(not out.exists() and not out.is_relative_to(ROOT),'Fresh external proof output required');out.mkdir(parents=True)
 save(out/'phase.json',dict(status='incomplete',sourceCommit=R,registrationCommit=P,controllerCommit=controller))
 c=GitHub(out);expected=json.loads((ROOT/'scripts/release/public-qualified-cohort-0457.json').read_bytes());deadline=time.monotonic()+540
 def run(label,argv,**kw):
  remaining=deadline-time.monotonic();need(remaining>0,'Consumer budget exhausted');result=subprocess.run([str(x) for x in argv],capture_output=True,timeout=min(180,remaining),**kw);(out/(label+'.stdout')).write_bytes(result.stdout);(out/(label+'.stderr')).write_bytes(result.stderr);save(out/(label+'-command.json'),dict(argv=[str(x) for x in argv],exitCode=result.returncode));need(result.returncode==0,'Consumer command failed: '+label)
 ownid=int(os.environ['GITHUB_RUN_ID']);attempt=int(os.environ['GITHUB_RUN_ATTEMPT']);own=run_record(c.json(PREFIX+f'/actions/runs/{ownid}','V-run'),ownid,controller,WORKFLOW,terminal=False);need(own.get('head_sha')==controller and own.get('path')==WORKFLOW and own.get('event')=='workflow_dispatch' and own.get('run_attempt')==attempt,'Actual V run identity differs')
 commit=c.json(PREFIX+'/git/commits/'+controller,'V-commit');jobs=collection(c,PREFIX+f'/actions/runs/{ownid}/attempts/{attempt}/jobs','jobs','V-jobs');jobs=[j for j in jobs if j.get('name')=='public-postflight'];need(len(jobs)==1 and jobs[0].get('head_sha')==controller and jobs[0].get('run_id')==ownid,'Actual V job differs');ownjob=jobs[0]['id']
 pr=run_record(c.json(PREFIX+f'/actions/runs/{args.pypi_run}','P-pypi-run'),args.pypi_run,P,'.github/workflows/publish-pypi.yml');need(pr.get('status')=='completed' and pr.get('conclusion')=='success' and pr.get('head_sha')==P and pr.get('path')=='.github/workflows/publish-pypi.yml' and pr.get('repository',{}).get('full_name')=='DocumentDrivenDX/weft','Original P publisher not terminal/successful')
 need(args.publication_attempt<=pr['run_attempt'],'Future original publication attempt')
 original=c.json(PREFIX+f'/actions/runs/{args.pypi_run}/attempts/{args.publication_attempt}','P-original-attempt');need(original.get('head_sha')==P and original.get('run_attempt')==args.publication_attempt,'Original publication attempt differs')
 jobs=collection(c,PREFIX+f'/actions/runs/{args.pypi_run}/attempts/{args.publication_attempt}/jobs','jobs','P-jobs');pub=[j for j in jobs if j.get('name')=='publish'];need(len(pub)==1 and pub[0].get('conclusion')=='success' and pub[0].get('head_sha')==P,'Original successful publish job absent')
 artifacts=collection(c,PREFIX+f'/actions/runs/{args.pypi_run}/artifacts','artifacts','P-artifacts');artifacts=[a for a in artifacts if a.get('name')=='weft-pypi-publication-0.4.57'];need(len(artifacts)==1,'Original publication artifact ambiguous/absent');artifact=artifacts[0]
 need(type(artifact.get('size_in_bytes')) is int and 0<artifact['size_in_bytes']<=2*1024*1024,'Original receipt artifact cap differs')
 need(not artifact.get('expired') and artifact.get('workflow_run',{}).get('head_sha')==P and artifact['workflow_run'].get('id')==args.pypi_run,'Original artifact source/run differs')
 uploads=[s for s in pub[0]['steps'] if 'upload-artifact' in s['name']];need(len(uploads)==1 and uploads[0].get('conclusion')=='success','Original receipt upload absent')
 stamp=lambda v:datetime.fromisoformat(v.replace('Z','+00:00'));need(stamp(uploads[0]['started_at'])<=stamp(artifact['created_at'])<=stamp(uploads[0]['completed_at']),'Original artifact upload window differs')
 raw=c.download(PREFIX+f'/actions/artifacts/{artifact["id"]}/zip',str(artifact['id'])+'.zip');need(len(raw)<=2*1024*1024 and 'sha256:'+sha(raw)==artifact.get('digest'),'Original receipt ZIP digest/cap differs');files=archive_files(raw);publication=json.loads(files['publication-receipt.json']);source=json.loads(files['source-receipt.json']);publication_receipts(publication,source,expected)
 release=c.json(PREFIX+f'/releases/{args.release_id}','immutable-release');assets=verify_release_api(release,release_id=args.release_id,tag='v0.4.57',draft=False,immutable=True);need(api_tag_commit('DocumentDrivenDX/weft','v0.4.57')==R,'Product tag differs')
 names={a['name']:a for a in assets};need(set(names)==set(expected['assetNames']),'Closed33 release inventory differs')
 def release_metadata(name):
  a=names[name];need(a['size']<=2*1024*1024,'Release metadata size exceeded');r=subprocess.run(['gh','api',PREFIX+f'/releases/assets/{a["id"]}','--header','Accept: application/octet-stream'],capture_output=True,timeout=60);need(r.returncode==0 and len(r.stdout)<=2*1024*1024 and 'sha256:'+sha(r.stdout)==a['digest'],'Immutable metadata download differs');(out/name).write_bytes(r.stdout);return json.loads(r.stdout)
 manifest=release_metadata('release-assets-0.4.57.json');need('sha256:'+args.release_manifest_sha256==names['release-assets-0.4.57.json']['digest'],'Reviewed actual public manifest differs');need(manifest.get('assetSha256')=={n:a['digest'][7:] for n,a in names.items() if n!='release-assets-0.4.57.json'},'Immutable release manifest inventory differs')
 pins=release_metadata('release-pins.json');need(pins.get('commit')==pins.get('sourceCommit')==R and pins.get('metadataCommit')==pins.get('workflowCommit')==P and sha(canonical(pins.get('currentQualification')))==expected['qualificationCanonicalSha256'],'Original reviewed qualified cohort differs')
 need(all(n in names and names[n]['digest']=='sha256:'+h for n,h in pins['currentQualification']['assetSha256'].items()),'Published original cohort payload digest differs')
 need(source.get('sums_sha256')==names['SHA256SUMS']['digest'][7:],'Original publisher checksum receipt differs')
 need(source.get('release_pins_sha256')==sha((out/'release-pins.json').read_bytes()),'Original publisher pins receipt differs')
 producer=c.json(PREFIX+f'/actions/runs/{pins["workflowRunId"]}','P-github-publisher');need(producer.get('head_sha')==P and producer.get('path')=='.github/workflows/release.yml' and producer.get('conclusion')=='success' and producer.get('status')=='completed','Original P GitHub publisher differs')
 with urllib.request.urlopen('https://pypi.org/pypi/umf-weft/0.4.57/json',timeout=30) as response:body=response.read(2*1024*1024+1)
 need(len(body)<=2*1024*1024,'Public metadata cap exceeded');(out/'public-pypi-version.json').write_bytes(body);rows=public_inventory(json.loads(body),expected)
 venv=out.parent/(out.name+'-venv');need(not venv.exists(),'Consumer venv already exists');run('venv',[sys.executable,'-m','venv',venv]);python=venv/'bin/python';env=dict(os.environ,PIP_CONFIG_FILE='/dev/null',PIP_INDEX_URL='',PIP_EXTRA_INDEX_URL='',PIP_FIND_LINKS='',PIP_NO_INDEX='',PYTHONPATH='');cwd=out.parent
 run('pip-install',[python,'-I','-m','pip','--isolated','install','--disable-pip-version-check','--no-cache-dir','--index-url','https://pypi.org/simple','--only-binary=:all:','--report',out/'pip-install-report.json','umf-weft==0.4.57'],cwd=cwd,env=env)
 name,url=selected_public_wheel(json.loads((out/'pip-install-report.json').read_bytes()),expected);byte_receipts=[]
 for row in rows:
  h=hashlib.sha256();count=0;selected=row['filename']==name;stream=(out/name).open('xb') if selected else None
  try:
   with urllib.request.urlopen(public_url(row['url']),timeout=60) as response:
    public_url(response.geturl())
    while chunk:=response.read(1024*1024):
     count+=len(chunk);need(count<=64*1024*1024,'Public archive cap exceeded');h.update(chunk)
     if stream:stream.write(chunk)
  finally:
   if stream:stream.close()
  need(h.hexdigest()==expected['archives'][row['filename']],'Public served bytes differ');byte_receipts.append(dict(name=row['filename'],url=row['url'],sha256=h.hexdigest(),bytes=count))
 save(out/'public-byte-verification.json',dict(status='passed',files=byte_receipts))
 run('pip-check',[python,'-I','-m','pip','--isolated','check'],cwd=cwd,env=env);run('pip-freeze',[python,'-I','-m','pip','--isolated','freeze','--all'],cwd=cwd,env=env)
 run('installed-packaging',[python,'-I',ROOT/'scripts/release/verify-wheel-packaging.py','--wheel',out/name,'--version','0.4.57','--distribution','umf-weft','--output',out/'installed-packaging.json'],cwd=cwd,env=env)
 code="import importlib.metadata as m,json,hashlib;from pathlib import Path;import weft;import weft.weft as n;d=m.distribution('umf-weft');h=lambda p:hashlib.sha256(Path(p).read_bytes()).hexdigest();print(json.dumps(dict(name=d.metadata['Name'],version=d.version,requiresDist=d.requires or [],ownedFiles=[str(d.locate_file(f).resolve()) for f in d.files],nativePath=str(Path(n.__file__).resolve()),shimPath=str(Path(weft.__file__).resolve()),nativeSha256=h(n.__file__),shimSha256=h(weft.__file__))))"
 run('installed-origin',[python,'-I','-c',code],cwd=cwd,env=env);record=json.loads((out/'installed-origin.stdout').read_bytes());installed_binding(record,expected['wheels'][name],venv);save(out/'installed-correspondence.json',record)
 save(out/'consumer-receipt.json',dict(schema='weft-public-postflight/1',status='consumer-controls-passed-final-job-upload-auth-pending',scope='One clean public-index Linux install/import/byte/ownership check; no fresh SDK42/native database proof',sourceCommit=R,registrationCommit=P,controllerCommit=controller,controllerTree=commit['tree']['sha'],controllerPayloadSha256={str(p.relative_to(ROOT)):sha(p.read_bytes()) for p in [Path(__file__),ROOT/WORKFLOW,ROOT/'scripts/release/public-qualified-cohort-0457.json']},actualRun=ownid,actualJob=ownjob,originalAttempt=attempt,originalPublication=dict(run=args.pypi_run,attempt=args.publication_attempt,job=pub[0]['id'],artifact=artifact['id'],digest=artifact['digest']),releaseId=args.release_id,releaseManifestSha256=args.release_manifest_sha256,selectedWheel=name,selectedWheelURL=url,selectedWheelSha256=expected['archives'][name],pipReportSha256=sha((out/'pip-install-report.json').read_bytes()),installedNativeSha256=record['nativeSha256'],installedShimSha256=record['shimSha256'],python=sys.version,platform=platform.platform(),freshSDK42=False))
 save(out/'phase.json',dict(status='consumer-controls-passed-final-job-upload-auth-pending',sourceCommit=R,registrationCommit=P,controllerCommit=controller))
 save(out/'proof-manifest.json',dict(files={str(p.relative_to(out)):dict(sha256=sha(p.read_bytes()),bytes=p.stat().st_size) for p in sorted(out.rglob('*')) if p.is_file()}))

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--controller-sha',required=True);p.add_argument('--pypi-run',type=int,required=True);p.add_argument('--publication-attempt',type=int,required=True);p.add_argument('--release-id',type=int,required=True);p.add_argument('--release-manifest-sha256',required=True);p.add_argument('--out',type=Path,required=True);args=p.parse_args()
 need(args.pypi_run>0 and args.publication_attempt>0 and args.release_id>0 and re.fullmatch('[0-9a-f]{64}',args.release_manifest_sha256) is not None,'Malformed reviewed operation binding')
 main(args)
