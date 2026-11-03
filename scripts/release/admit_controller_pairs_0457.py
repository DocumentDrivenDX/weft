"""Actual GitHub admission for separately registered Mac/ARM R/Q profiles."""
from pathlib import Path
import base64,json,re
from datetime import datetime,timedelta
from current_macos_pair_0457 import *
from current_macos_pair_0457 import _gate
from admit_current_sdk_0457 import GitHub,PREFIX,REPOSITORY,collection,archive_files,sha,save,actual_run,source_files,actual_job,actual_artifact,artifact_job_binding
from admit_current_sources_0457 import actual_job as source_job

def pair_archive_files(raw):
 import io,zipfile
 need(len(raw)<=70*1024*1024,'Original pair artifact ZIP cap exceeded')
 with zipfile.ZipFile(io.BytesIO(raw)) as z:
  rows=z.infolist();need(len(rows)<=160 and sum(i.file_size for i in rows)<=70*1024*1024,'Original pair expanded/count cap exceeded')
  for row in rows:
   name=row.filename;parts=Path(name).parts
   need(len(parts)<=2 and (len(parts)==1 or parts[0] in ['web','current-tools','native-builder-tools']),'Pair artifact layout differs')
   limit=60*1024*1024 if name.endswith('.whl') else 20*1024*1024 if name=='web/weft_wasm_bg.wasm' else 5*1024*1024 if name=='web/weft_wasm.js' else 2*1024*1024
   need(row.file_size<=limit,'Original per-file artifact cap exceeded')
 return archive_files(raw)

def content(c,commit,tree,path,label):
 rows=tree_rows(tree,tree['sha']);need(path in rows and rows[path]['mode'] in ['100644','100755'],'Missing/unsafe tracked control file')
 value=c.json(PREFIX+'/contents/'+path+'?ref='+commit,label);need(value.get('path')==path and value.get('encoding')=='base64' and value.get('type')=='file','Control API type differs')
 raw=base64.b64decode(''.join(value['content'].split()),validate=True);blob=__import__('hashlib').sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest();need(value.get('sha')==rows[path]['sha']==blob,'Control Git blob differs');return raw

def registry(c,registration_commit):
 need(re.fullmatch('[0-9a-f]{40}',registration_commit) is not None,'Registration commit malformed')
 main=c.json(PREFIX+'/git/ref/heads/main','registration-main');need(main.get('object',{}).get('sha')==registration_commit,'Registration is not reviewed current main')
 commit=c.json(PREFIX+'/git/commits/'+registration_commit,'registration-commit');tree=c.json(PREFIX+'/git/trees/'+commit['tree']['sha']+'?recursive=1','registration-tree')
 raw=content(c,registration_commit,tree,REGISTRY,'registration-content');value=load_json(raw)
 closed(value,{'schema','status','sourceCommit','sourceTree','macos','arm64'},'pair registration')
 need(value['schema']=='weft-0457-controller-pair-registration/1' and value['status']=='reviewed' and value['sourceCommit']==R and value['sourceTree']==RTREE,'Pair registration not reviewed/source differs')
 if value['macos'] is not None:pair(value['macos'])
 if value['arm64'] is not None:arm_pair(value['arm64'])
 active=value['arm64'] or value['macos'];need(active is not None,'No separately reviewed pair')
 anchor=c.json(PREFIX+'/git/trees/'+active['controllerTree']+'?recursive=1','registration-controller-tree')
 before=tree_rows(anchor,active['controllerTree']);after=tree_rows(tree,commit['tree']['sha'])
 changed={p for p in before.keys()|after.keys() if before.get(p)!=after.get(p)}
 need(changed=={REGISTRY},'Registration commit changes controller/product bytes beyond exact-pair data')
 return value,main

def authenticate_pair_code(c,p,target):
 selected_pair(p,target);r=c.json(PREFIX+'/git/trees/'+RTREE+'?recursive=1',target+'-R-tree');q=c.json(PREFIX+'/git/commits/'+p['controllerCommit'],target+'-Q-commit');need(q['sha']==p['controllerCommit'] and q['tree']['sha']==p['controllerTree'],'Actual Q/tree differs')
 t=c.json(PREFIX+'/git/trees/'+p['controllerTree']+'?recursive=1',target+'-Q-tree');proof=closed_closure(p,r,t,target)
 need(sha(content(c,p['controllerCommit'],t,p['workflowPath'],target+'-workflow'))==p['workflowSha256'],'Actual workflow bytes differ')
 for i,(name,h) in enumerate(sorted(p['controllerHashes'].items())):need(sha(content(c,p['controllerCommit'],t,name,target+'-code-'+str(i)))==h,'Registered Q controller bytes differ')
 return proof

def producer_identity(run,run_id,p,*,final_success=True):
 need(type(run_id) is int and run_id>0 and type(run.get('repository',{}).get('id')) is int and run['repository']['id']>0,'Producer repository/run ID malformed')
 need(run.get('id')==run_id and run.get('repository',{}).get('full_name')==REPOSITORY and run.get('head_repository',{}).get('id')==run.get('repository',{}).get('id'),'Producer repository/run differs')
 need(run.get('head_sha')==run.get('head_commit',{}).get('id')==p['controllerCommit'] and run.get('head_commit',{}).get('tree_id')==p['controllerTree'],'Actual run head/tree is not Q')
 need(run.get('head_branch')==p['controllerRef'] and run.get('event')=='workflow_dispatch' and run.get('path')==p['workflowPath'],'Closed controller workflow/ref differs')
 allowed=('success',) if final_success else ('success','failure','cancelled','timed_out')
 need(run.get('status')=='completed' and run.get('conclusion') in allowed,'Final/original platform producer not completed with admitted outcome')

def authenticate_platform(c,p,target,run_id,out):
 closure=authenticate_pair_code(c,p,target);run=c.json(PREFIX+f'/actions/runs/{run_id}',target+'-run');producer_identity(run,run_id,p)
 artifacts=collection(c,PREFIX+f'/actions/runs/{run_id}/artifacts','artifacts',target+'-artifacts');selected=[]
 prefix=f'weft-pair-{target}-{R}-{p["controllerCommit"]}-{run_id}-attempt-'
 for a in artifacts:
  name=a.get('name','')
  if not name.startswith(prefix):continue
  n=name[len(prefix):];need(re.fullmatch('[1-9][0-9]*',n) is not None,'Artifact attempt malformed');attempt=int(n);need(attempt<=run['run_attempt'],'Future artifact attempt')
  original=c.json(PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}',target+'-attempt-'+n)
  producer_identity(original,run_id,p,final_success=False)
  need(original.get('head_sha')==p['controllerCommit'] and original.get('path')==p['workflowPath'] and original.get('run_attempt')==attempt and original.get('workflow_id')==run['workflow_id'],'Original attempt identity differs')
  jobs=collection(c,PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}/jobs','jobs',target+'-jobs-'+n);job_name='macos-pair' if target=='aarch64-apple-darwin' else 'arm64-pair';jobs=[j for j in jobs if j.get('name')==job_name]
  need(len(jobs)==1,'Original platform job missing/duplicate');j=jobs[0]
  need(j.get('head_sha')==p['controllerCommit'] and j.get('run_id')==run_id and j.get('run_attempt',attempt)==attempt,'Original job binding differs')
  if j.get('conclusion')!='success':continue
  need(j.get('status')=='completed' and ('macos-14' if target=='aarch64-apple-darwin' else 'ubuntu-24.04-arm') in j.get('labels',[]),'Actual platform runner differs')
  uploads=[s for s in j['steps'] if s['name']=='Retain exact pair qualification and partials'];need(len(uploads)==1 and uploads[0].get('conclusion')=='success','Original final upload not successful')
  def stamp(s):
   need(type(s) is str,'Actual timestamp absent');value=datetime.fromisoformat(s.replace('Z','+00:00'));need(value.utcoffset()==timedelta(0),'Actual upload timestamp is not UTC');return value
  need(stamp(uploads[0]['started_at'])<=stamp(a['created_at'])<=stamp(uploads[0]['completed_at']),'Artifact outside original upload window')
  need(a.get('expired') is False and a.get('workflow_run',{}).get('id')==run_id and a['workflow_run'].get('head_sha')==p['controllerCommit'] and re.fullmatch('sha256:[0-9a-f]{64}',a.get('digest','')) is not None,'Artifact API binding/digest differs')
  raw=c.download(PREFIX+f'/actions/artifacts/{a["id"]}/zip',str(a['id'])+'.zip');need('sha256:'+sha(raw)==a['digest'],'Actual artifact digest differs');files=pair_archive_files(raw)
  result=validate_component(load_json(files['component.json']),files,p,target,run_id,attempt)
  selected.append((result,files,dict(artifactID=a['id'],artifactDigest=a['digest'],actualJob=j['id'],originalAttempt=attempt,runID=run_id,actualWorkflowId=run['workflow_id'],closure=closure)))
 need(len(selected)==1,'One successful original platform component required')
 again=c.json(PREFIX+f'/actions/runs/{run_id}',target+'-run-after');producer_identity(again,run_id,p);need(again['run_attempt']==run['run_attempt'],'Attempt changed during admission');return selected[0]

def authenticate_x86(c,run_id,out):
 # Parent run may have a failed ARM sibling. Authenticate the original positive
 # job separately; do not change its conclusion/attempt or fabricate /1 identity.
 run=c.json(PREFIX+f'/actions/runs/{run_id}','x86-run');actual_run(run,run_id,R,RTREE,'.github/workflows/release.yml',final_success=False)
 rows=collection(c,PREFIX+f'/actions/runs/{run_id}/artifacts','artifacts','x86-artifacts');selected=[];target='x86_64-unknown-linux-gnu'
 from admit_current_sdk_0457 import artifact_candidates
 for attempt,a in artifact_candidates(rows,run,R,target):
  need(attempt<=run['run_attempt'],'Future x86 attempt');original=c.json(PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}','x86-attempt-'+str(attempt));actual_run(original,run_id,R,RTREE,'.github/workflows/release.yml',final_success=False)
  need(original['run_attempt']==attempt,'Original x86 attempt differs');jobs=collection(c,PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}/jobs','jobs','x86-jobs-'+str(attempt));j=actual_job(jobs,target,original,R,successful=False)
  if j['conclusion']!='success':continue
  actual_artifact(a,run,R,target,attempt,require_live=True);artifact_job_binding(a,j,target);raw=c.download(PREFIX+f'/actions/artifacts/{a["id"]}/zip',str(a['id'])+'.zip');need('sha256:'+sha(raw)==a['digest'],'x86 actual ZIP differs');files=archive_files(raw);component=load_json(files['component.json'])
  need(component.get('profile')=='weft-current-sdk-platform-observation/1' and component.get('status')=='passed' and component.get('sourceCommit')==component.get('workflowCommit')==R and component.get('sourceTree')==RTREE,'Original x86 component identity differs')
  need(component.get('producerRunId')==run_id and component.get('producerRunAttempt')==str(attempt) and component.get('producerJobName')=='wheels' and component.get('workflowRef')==REPOSITORY+'/.github/workflows/release.yml@refs/heads/main','Original x86 provenance differs')
  need(component.get('target')==target and [component.get(n) for n in ['temporalCases','routingControls','sdkControls']]==[14,5,42] and component.get('nativeAdmission') is False and component.get('publicationAdmission') is False,'Original x86 scope differs')
  stages=component.get('stageResults',[]);need([s.get('name') for s in stages]==['venv','normal-install','pip-check','packaging','native-origin','temporal14','routing5','sdk42'] and all(type(s.get('exitCode')) is int and s['exitCode']==0 for s in stages),'Original x86 stages differ')
  codehash=source_files(c,R,RTREE,out);need(component.get('controllerCodeSha256')==codehash and component.get('controllerScript')=='scripts/release/qualify_current_sdk_0457.py','Original R SDK controller differs')
  w=validate_payload(target,files)
  for k,v in w.items():need(component.get(k)==v,'Original x86 actual byte binding differs')
  selected.append((w,files,dict(artifactID=a['id'],artifactDigest=a['digest'],runID=run_id,actualJob=j['id'],originalAttempt=attempt,originalAttemptConclusion=original['conclusion'],finalRunConclusion=run['conclusion'])))
 need(len(selected)==1,'Unique successful original x86 job/artifact required');return selected[0]

def authenticate_source(c,run_id,out):
 # Separate versioned R/Q profile admits the immutable already-successful R
 # source run while main holds the reviewed registration. Actual APIs remain R.
 run=c.json(PREFIX+f'/actions/runs/{run_id}','source-run');actual_run(run,run_id,R,RTREE,'.github/workflows/temporal-source-0457.yml')
 artifacts=collection(c,PREFIX+f'/actions/runs/{run_id}/artifacts','artifacts','source-artifacts');result={};payloads={}
 for label,prefix,job_name,manifest in [('source','current-source-0457','source-archives','source-manifest.json'),('wasm','current-temporal-wasm-0457','temporal-wasm','wasm-manifest.json')]:
  pattern=re.compile(re.escape(f'{prefix}-{R}-{run_id}-attempt-')+'([1-9][0-9]*)');success=[]
  for a in artifacts:
   match=pattern.fullmatch(a.get('name',''))
   if not match:continue
   attempt=int(match[1]);need(attempt<=run['run_attempt'],'Future source attempt');original=c.json(PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}',label+'-attempt-'+str(attempt));actual_run(original,run_id,R,RTREE,'.github/workflows/temporal-source-0457.yml',final_success=False);need(original['run_attempt']==attempt,'Source attempt changed')
   need(a.get('expired') is False and a.get('workflow_run',{}).get('id')==run_id and a['workflow_run'].get('head_sha')==R and re.fullmatch('sha256:[0-9a-f]{64}',a.get('digest','')) is not None,'Source artifact API differs')
   raw=c.download(PREFIX+f'/actions/artifacts/{a["id"]}/zip',str(a['id'])+'.zip');need('sha256:'+sha(raw)==a['digest'],'Source actual ZIP differs');files=archive_files(raw);m=load_json(files[manifest]);jobs=collection(c,PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}/jobs','jobs',label+'-jobs-'+str(attempt));j=source_job(jobs,m,a,run_id,R,job_name,attempt)
   if j is None:continue
   need(m.get('sourceCommit')==m.get('workflowCommit')==R and m.get('sourceTree')==RTREE and m.get('status')=='passed' and m.get('cargoLockSha256')=='92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d','Source actual manifest identity differs')
   if label=='source':
    need(m.get('archiveCorrespondenceVerified') is True,'Source archive correspondence absent')
    resolution=load_json(files['archive-selected-resolution.json']);need(resolution.get('checked') is True and resolution.get('selectedPackageCount')==225 and len(resolution.get('selectedLocalRoots',[]))==6,'Actual archived locked resolution absent')
    for role in ['sdist','sourceArchive']:
     v=m[role];name=Path(v.get('archive',v.get('path'))).name;matches=[b for n,b in files.items() if Path(n).name==name];need(len(matches)==1 and sha(matches[0])==v['sha256'],'Source archive payload differs');payloads[name]=matches[0]
   else:
    need(str(m.get('runID'))==str(run_id) and m.get('NodeCases')==m.get('ChromiumCases')==14 and m.get('carrierRefusalsPerBoundary')==3 and m.get('target')=='wasm32-unknown-unknown' and m.get('features','').startswith('raw-postgresql-reads,') and len(m.get('artifacts',[]))==5,'Actual WASM boundary inventory differs')
    for row in m['artifacts']:
     n=Path(row['path']);rel='raw-weft_wasm.wasm' if n.name=='weft_wasm.wasm' else '/'.join(n.parts[-2:]);need(rel in files and sha(files[rel])==row['sha256'] and len(files[rel])==row['bytes'],'Actual WASM payload differs');payloads['temporal-'+rel.replace('/','-')]=files[rel]
   success.append(dict(**j,artifactID=a['id'],artifactDigest=a['digest'],manifestSha256=sha(files[manifest])));payloads['current-'+label+'-manifest.json']=files[manifest]
  need(len(success)==1,'Unique original successful source component required');result[label]=success[0]
 return result,payloads

def authenticate(registration_commit,linux_run,mac_run,arm_run,source_run,out):
 out=Path(out);need(not out.exists(),'Fresh external admission output required');out.mkdir(parents=True);c=GitHub(out);enrolled,main=registry(c,registration_commit);need(enrolled['macos'] is not None and enrolled['arm64'] is not None,'Separate Mac and ARM registrations required for cohort')
 source,assets=authenticate_source(c,source_run,out);rows=[];observations=[]
 for target,args in [('x86_64-unknown-linux-gnu',None),('aarch64-apple-darwin',(enrolled['macos'],mac_run)),('aarch64-unknown-linux-gnu',(enrolled['arm64'],arm_run))]:
  result,files,obs=authenticate_x86(c,linux_run,out) if args is None else authenticate_platform(c,args[0],target,args[1],out)
  if args is not None:
   workflow=c.json(PREFIX+'/actions/workflows/'+str(obs['actualWorkflowId']),target+'-workflow-identity');need(workflow.get('id')==obs['actualWorkflowId'] and workflow.get('path')==args[0]['workflowPath'],'Actual workflow API identity differs')
  if target=='aarch64-apple-darwin':obs['originalMacToolAndBoundProof']=validate_mac_tools(files,enrolled['macos'])
  if target=='aarch64-unknown-linux-gnu':
   command=load_json(files['arm-cold-acquisition-build-command.json']);need(command.get('stage')=='arm-cold-acquisition-build' and type(command.get('exitCode')) is int and command.get('exitCode')==0 and command.get('timedOut') is False and command.get('logOverflow') is False and type(command.get('allowedSeconds')) in (int,float) and 0<command['allowedSeconds']<=4200,'Separate ARM acquisition/build bound failed')
   inputs=load_json(files['arm-action-inputs.json']);need(inputs.get('actionCommit')=='e83996d129638aa358a18fbd1dfb82f0b0fb5d3b' and inputs.get('sourceCommit')==R and inputs.get('controllerCommit')==enrolled['arm64']['controllerCommit'] and inputs.get('INPUT_TARGET')=='aarch64-unknown-linux-gnu' and inputs.get('INPUT_MANYLINUX')=='2014' and inputs.get('INPUT_RUST-TOOLCHAIN')=='1.90.0' and inputs.get('INPUT_MATURIN-VERSION')=='1.9.6' and inputs.get('INPUT_SCCACHE')=='false' and inputs.get('INPUT_CONTAINER')=='quay.io/pypa/manylinux2014_aarch64@sha256:db68ee9e5e879da5945dfbc8cab446f02245da6522aca0a9e4cda9256c911d2f' and inputs.get('INPUT_ARGS')=='--manifest-path crates/weft-python/Cargo.toml --release --locked --out dist','ARM pinned Action/compiler inputs differ')
   for name,prefix in [('rust','rustc 1.90.0 '),('cargo','cargo 1.90.0 '),('maturin','maturin 1.9.6')]:need(files['native-builder-tools/'+name+'.stdout'].decode().startswith(prefix),'Actual manylinux tool identity differs')
  rows.append(result);observations.append(dict(target=target,**obs));name=result['wheel'];need(name not in assets,'Duplicate wheel');assets[name]=files[name]
  for key in ['component.json','sdk-smoke.json','packaging.json']:assets[target+'-'+key]=files[key]
  if target=='aarch64-apple-darwin':
   for key in ['compiler-cases.json','compiler-roster.json','compiler-reports.json','python-compiler-parity.json','browser-compiler-parity.json','browser-compiler-receipts.json']:assets['macos-'+key]=files[key]
 need({r['target'] for r in rows}==set(_gate['TARGETS']),'Exactly three required platforms')
 need(c.json(PREFIX+'/git/ref/heads/main','registration-main-after')==main,'Registration main moved during admission')
 proof=dict(profile=COHORT_PROFILE,status='checked',sourceCommit=R,sourceTree=RTREE,registrationCommit=registration_commit,registration=enrolled,linuxRun=linux_run,macosRun=mac_run,arm64Run=arm_run,sourceRun=source_run,wheels=rows,originalSourceComponents=source,actualGitHubObservations=observations,wheelSha256={r['wheel']:r['wheelSha256'] for r in rows},assetSha256={n:sha(b) for n,b in assets.items()},nativeAdmission=False,publicationAdmission=False,attemptReceiptsRewritten=False)
 save(out/'controller-pair-admission.json',proof);return proof,assets

def validate_mac_tools(files,registration):
 """Whole-cohort Mac verification retains the original tool/boundary gates."""
 p=pair(registration);preflight=load_json(files['runner-preflight.json'])
 need(preflight.get('profile')=='weft-macos-pair-toolchain/1' and preflight.get('status')=='passed' and preflight.get('sourceCommit')==R and preflight.get('workflowCommit')==p['controllerCommit'] and preflight.get('sourceTree')==RTREE and preflight.get('controllerTree')==p['controllerTree'],'Actual Mac runner source/controller differs')
 need(preflight.get('version')=='0.4.57' and preflight.get('distribution')=='umf-weft' and preflight.get('target')=='aarch64-apple-darwin','Mac runner distribution/target differs')
 runner=preflight.get('runner',{});need(runner.get('label')=='macos-14' and runner.get('os')=='Darwin' and runner.get('architecture')=='arm64','Mac runner identity differs')
 for k,expected in [('bun','1.4.2'),('maturin','maturin 1.9.6'),('wasmBindgen','wasm-bindgen 0.2.105')]:need(preflight.get(k)==expected,'Mac tool version differs: '+k)
 for k,prefix in [('python','Python 3.12.'),('node','v24.'),('rustc','rustc 1.90.0 '),('cargo','cargo 1.90.0 ')]:need(type(preflight.get(k)) is str and preflight[k].startswith(prefix),'Mac runtime/compiler differs: '+k)
 features=preflight.get('maturinFeatures');need(type(features) is list and features==['ashlar-sdk-ordered-page-candidate','raw-postgresql-reads','ashlar-databricks-paths-keys','ashlar-databricks-qualified','truss-postgresql-pg18','current-postgresql-scalar-candidate','current-databricks-scalar-candidate','current-query-refinement-candidate','current-presence-graph-requirements-candidate'],'Frozen actual R feature selection differs')
 need(preflight.get('maturinFeaturesSha256')==sha(__import__('json').dumps(features,separators=(',',':')).encode()),'Mac feature digest differs')
 binary={}
 for line in files['tool-binary-sha256.txt'].decode().splitlines():
  fields=line.split('  ',2);need(len(fields)==3 and re.fullmatch('[0-9a-f]{64}',fields[0]) is not None and fields[1] not in binary and Path(fields[2]).is_absolute(),'Actual tool binary inventory malformed');binary[fields[1]]=fields[0]
 need(set(binary)=={'python3','bun','node','npm','maturin','rustc','cargo','wasm-bindgen'},'Actual tool binary roster differs')
 bounds={'runner-preflight':45,'rust-toolchain':480,'maturin-install':300,'wasm-bindgen-install':480,'verify-toolchain':45,'macos-wheel-build':1800,'macos-wheel-venv':30,'macos-wheel-install':180,'macos-wheel-check':30,'rust-comparator':600,'wasm-build':720,'wasm-bindgen':60,'bun-install':120,'browser-build':120,'rust-compiler-vectors':60,'python-wheel-vectors':120,'playwright-install':300,'chromium-install':600,'browser-parity':300}
 deadlines=set()
 for name,bound in bounds.items():
  r=load_json(files[name+'-command.json']);need(r.get('stage')==name and type(r.get('exitCode')) is int and r['exitCode']==0 and r.get('timedOut') is False and r.get('logOverflow') is False,'Original Mac stage failed/missing: '+name)
  need(type(r.get('allowedSeconds')) in (int,float) and 0<r['allowedSeconds']<=bound,'Mac stage budget changed: '+name);deadlines.add(r['originalDeadlineMonotonic'])
 need(len(deadlines)==1,'Original global monotonic deadline was rewritten');return dict(toolBinarySha256=binary,stageBounds=bounds,sourceCommit=R,controllerCommit=p['controllerCommit'])
