"""Read-only authentication of original final-R source/WASM producer jobs."""
from pathlib import Path,PurePosixPath
from datetime import datetime,timedelta
import argparse,re
from admit_current_sdk_0457 import GitHub,collection,actual_run,archive_files,sha,save,need,load_json,REPOSITORY,PREFIX,LOCK,source_files
WORKFLOW='.github/workflows/temporal-source-0457.yml'
def actual_job(jobs,manifest,item,run_id,R,job_name,attempt):
 need(type(attempt) is int and attempt>0,'Invalid original attempt')
 need(manifest.get('producerRunId')==run_id and manifest.get('producerRunAttempt')==attempt and manifest.get('producerJobName')==job_name,'Wrong claimed component job/attempt')
 found=[j for j in jobs if j.get('name')==job_name and j.get('run_id')==run_id]
 need(len(found)==1,'Actual original-attempt job missing/duplicate');j=found[0]
 need(j.get('run_attempt',attempt)==attempt and j.get('head_sha')==R,'Actual job attempt/source differs')
 if j.get('status')!='completed' or j.get('conclusion')!='success':return None
 upload='Retain source archives and correspondence immediately' if job_name=='source-archives' else 'Retain all temporal compiler outputs on success or failure'
 steps=[s for s in j.get('steps',[]) if s.get('name')==upload]
 need(len(steps)==1 and steps[0].get('conclusion')=='success','Actual upload not successful')
 def stamp(s):
  d=datetime.fromisoformat(s.replace('Z','+00:00'));need(d.utcoffset()==timedelta(0),'Timestamp not UTC');return d
 need(stamp(steps[0]['started_at'])<=stamp(item['created_at'])<=stamp(steps[0]['completed_at']),'Artifact creation outside original job upload window')
 return dict(jobID=j['id'],attempt=attempt,jobName=job_name)
def authenticate(R,run_id,out):
 out=Path(out).absolute();need(not out.exists() and not out.is_symlink(),'Fresh external output required');need(not out.resolve().is_relative_to(Path(__file__).resolve().parents[2]),'Output inside source');out.mkdir(parents=True)
 c=GitHub(out)
 ref=c.json(PREFIX+'/git/ref/heads/main','main');need(ref.get('object',{}).get('sha')==R,'Main moved')
 commit=c.json(PREFIX+'/git/commits/'+R,'commit');need(commit.get('sha')==R,'Commit differs');tree=commit['tree']['sha'];source_files(c,R,tree,out)
 run=c.json(PREFIX+f'/actions/runs/{run_id}','run');actual_run(run,run_id,R,tree,WORKFLOW)
 rows=collection(c,PREFIX+f'/actions/runs/{run_id}/artifacts','artifacts','artifacts');admitted={}
 for label,prefix,job_name,manifest_name in [('source','current-source-0457','source-archives','source-manifest.json'),('wasm','current-temporal-wasm-0457','temporal-wasm','wasm-manifest.json')]:
  pattern=re.compile(re.escape(f'{prefix}-{R}-{run_id}-attempt-')+'([1-9][0-9]*)');successes=[]
  for item in rows:
   match=pattern.fullmatch(item.get('name',''))
   if not match:continue
   attempt=int(match[1]);need(attempt<=run['run_attempt'],'Future attempt')
   original=c.json(PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}',label+f'-attempt-{attempt}')
   actual_run(original,run_id,R,tree,WORKFLOW,final_success=False);need(original['run_attempt']==attempt and original['workflow_id']==run['workflow_id'],'Original attempt differs')
   need(item.get('expired') is False and item.get('workflow_run',{}).get('id')==run_id and item['workflow_run'].get('head_sha')==R,'Artifact binding differs')
   need(re.fullmatch('sha256:[0-9a-f]{64}',item.get('digest','')) is not None,'Actual digest absent')
   raw=c.download(PREFIX+f'/actions/artifacts/{item["id"]}/zip',str(item['id'])+'.zip');need(sha(raw)==item['digest'][7:],'Actual artifact digest differs');files=archive_files(raw)
   m=load_json(files[manifest_name]);need(m.get('sourceCommit')==R and m.get('sourceTree')==tree and m.get('workflowCommit')==R,'Source/tree/controller differs')
   jobs=collection(c,PREFIX+f'/actions/runs/{run_id}/attempts/{attempt}/jobs','jobs',label+f'-jobs-{attempt}')
   observed=actual_job(jobs,m,item,run_id,R,job_name,attempt)
   if observed is None:continue
   need(m.get('status')=='passed' and m.get('cargoLockSha256')==LOCK,'Component failed/frozen lock differs')
   if label=='source':
    need(m.get('archiveCorrespondenceVerified') is True,'Archive correspondence absent')
    for role in ['sdist','sourceArchive']:
     record=m[role];base=Path(record['archive'] if role=='sdist' else record['path']).name
     payloads=[v for k,v in files.items() if PurePosixPath(k).name==base];need(len(payloads)==1 and sha(payloads[0])==record['sha256'],'Source archive bytes differ')
   else:
    need(str(m.get('runID'))==str(run_id) and m.get('NodeCases')==m.get('ChromiumCases')==14 and m.get('carrierRefusalsPerBoundary')==3,'WASM boundary inventory differs')
    need(m.get('target')=='wasm32-unknown-unknown' and m.get('features','').startswith('raw-postgresql-reads,') and len(m['artifacts'])==5,'WASM profile differs')
    for row in m['artifacts']:
     p=PurePosixPath(row['path']);rel='raw-weft_wasm.wasm' if p.name=='weft_wasm.wasm' else '/'.join(p.parts[-2:])
     need(rel in files and sha(files[rel])==row['sha256'] and len(files[rel])==row['bytes'],'WASM bytes differ')
   save(out/(label+'-manifest.json'),m);successes.append(dict(**observed,artifactID=item['id'],artifactDigest=item['digest'],manifestSha256=sha(files[manifest_name])))
  need(len(successes)==1,'Original successful source component ambiguous/missing');admitted[label]=successes[0]
 again=c.json(PREFIX+f'/actions/runs/{run_id}','run-after');actual_run(again,run_id,R,tree,WORKFLOW);need(again['run_attempt']==run['run_attempt'],'Attempt changed');need(c.json(PREFIX+'/git/ref/heads/main','main-after')==ref,'Main changed')
 result=dict(status='checked',sourceCommit=R,sourceTree=tree,runID=run_id,components=admitted,nativeDatabaseExecution=False,publicationAdmission=False,attemptReceiptsRewritten=False);save(out/'current-source-admission.json',result);return result
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--source-sha',required=True);p.add_argument('--run-id',required=True,type=int);p.add_argument('--out',required=True,type=Path);a=p.parse_args();authenticate(a.source_sha,a.run_id,a.out)
