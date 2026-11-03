"""Closed hosted retention-only projector; never executes retained consumers."""
from pathlib import Path
from datetime import datetime
import argparse,json,hashlib,zipfile,stat,subprocess,sys,os
ROOT=Path(__file__).resolve().parents[2]
DATA=Path(__file__).with_name('postflight_projection_0457')
CONFIG_SHA='6cb310ce4bd990ddad2591d97bef98a5a0304bb27a4766cc953e3c6359edc0a5'
WORKFLOW='.github/workflows/retained-public-postflight-0457.yml'
def need(ok,message):
 if not ok:raise ValueError(message)
def sha(raw):return hashlib.sha256(raw).hexdigest()
def file_sha(path):
 h=hashlib.sha256()
 with Path(path).open('rb') as f:
  while b:=f.read(4*1024*1024):h.update(b)
 return h.hexdigest()
def save(path,value):Path(path).write_text(json.dumps(value,indent=2,sort_keys=True)+'\n')
def inventory(z):
 rows=[];seen=set()
 for i in sorted(z.infolist(),key=lambda i:i.filename):
  n=i.filename;mode=i.external_attr>>16
  need(n not in seen and not n.startswith('/') and '..' not in Path(n).parts and '\\' not in n and all(x not in ['','.'] for x in n.split('/')),'Unsafe/duplicate original path');seen.add(n)
  need(not i.flag_bits&1 and not i.is_dir() and stat.S_ISREG(mode) and mode in [0o100644,0o100755,0o100777],'Original type/mode/encryption differs')
  rows.append(dict(path=n,bytes=i.file_size,compressedBytes=i.compress_size,crc32=i.CRC,mode=mode,externalAttr=i.external_attr,createSystem=i.create_system,compression=i.compress_type,flagBits=i.flag_bits,headerOffset=i.header_offset,isDirectory=i.is_dir()))
 return rows

def verified_members(z,sealed_inventory,whitelist):
 rows=inventory(z);need(rows==sealed_inventory['entries'],'Sealed original complete inventory differs')
 need(len(rows)==sealed_inventory['count'] and sum(r['bytes'] for r in rows)==sealed_inventory['expandedBytes'],'Original inventory count/bytes differs')
 selected=whitelist['members'];names=[r['path'] for r in selected];need(len(names)==len(set(names))==whitelist['count'],'Whitelist duplicated/missing')
 actual={r['path']:r for r in rows};need(set(names)=={n for n in actual if not n.startswith('proof-venv/')},'Only exact selected proof subset allowed')
 need(all(n in ['initial.json','operation.log','pure-controls.log'] or n.startswith('proof/') for n in names),'Unselected proof path')
 need(sum(r['bytes'] for r in selected)==whitelist['expandedBytes'],'Selected size differs');payloads={}
 for row in selected:
  n=row['path'];need({k:v for k,v in row.items() if k!='sha256'}==actual[n],'Selected original metadata differs')
  need(row['mode']==0o100644 and row['bytes']<=(64*1024*1024 if n.endswith('.whl') else 2*1024*1024),'Selected regular proof mode/size differs')
  raw=z.read(n);need(len(raw)==row['bytes'] and sha(raw)==row['sha256'],'Selected original bytes differ');payloads[n]=raw
 name=whitelist['originalProofManifestPath'];need(name in payloads and sha(payloads[name])==whitelist['originalProofManifestSha256'],'Original proof manifest absent/changed')
 manifest=json.loads(payloads[name]);expected={n[6:] for n in names if n.startswith('proof/') and n!=name};need(set(manifest['files'])==expected,'Original manifest roster differs')
 for n,row in manifest['files'].items():need(len(payloads['proof/'+n])==row['bytes'] and sha(payloads['proof/'+n])==row['sha256'],'Original manifest member differs')
 return payloads

def contained_paths(quarantine,state,out,root):
 paths=[p.resolve() for p in [quarantine,state,out]];root=root.resolve()
 need(all(not p.is_relative_to(root) for p in paths),'Controller input/output must be outside checkout')
 need(all(not a.is_relative_to(b) and not b.is_relative_to(a) for i,a in enumerate(paths) for b in paths[i+1:]),'Quarantine/state/upload must be disjoint')
 return paths

def write_capsule(out,payloads,record,inventory_bytes,provenance_payloads,caps):
 need(record.get('originalArtifactAccepted') is False and record.get('consumerReexecuted') is False and record.get('newConsumerClaim') is False,'Projection must not accept original or claim a new consumer')
 need(set(provenance_payloads)=={'projection-provenance/original-terminal-run.json','projection-provenance/original-terminal-jobs.json','projection-provenance/original-artifact-api.json'},'New provenance metadata roster differs')
 generated=dict(provenance_payloads);generated.update({'retention-projection.json':json.dumps(record,indent=2,sort_keys=True).encode()+b'\n','original-container-inventory.json':inventory_bytes});need(not set(generated)&set(payloads),'Metadata collision')
 allbytes=dict(payloads,**generated);manifest=dict(schema='weft-retention-projection-output/1',originalArtifactAccepted=False,files={n:dict(bytes=len(b),sha256=sha(b),mode=0o100644) for n,b in sorted(allbytes.items())});allbytes['projection-output-manifest.json']=json.dumps(manifest,indent=2,sort_keys=True).encode()+b'\n'
 need(len(allbytes)<=caps['entries'] and sum(len(b) for b in allbytes.values())<=caps['expandedBytes'] and sum(len(b) for b in allbytes.values())<caps['zipBytes']-1024*1024,'New capsule exceeds unchanged bounds')
 need(not out.exists(),'Projection output exists; never overwrite');out.mkdir()
 for n,b in sorted(allbytes.items()):
  need(not n.startswith('/') and '..' not in Path(n).parts and '\\' not in n,'Unsafe projected path');p=out/n;p.parent.mkdir(parents=True,exist_ok=True)
  with p.open('xb') as f:f.write(b)
  p.chmod(0o644)
 for n,b in allbytes.items():need((out/n).read_bytes()==b,'Projected member bytes changed')
 return manifest

def main(args):
 need(os.environ.get('GITHUB_REPOSITORY')=='DocumentDrivenDX/weft' and os.environ.get('GITHUB_EVENT_NAME')=='workflow_dispatch' and os.environ.get('GITHUB_REF')=='refs/heads/main','Closed manual projector only')
 controller=os.environ.get('GITHUB_SHA');need(controller==args.controller_sha and len(controller)==40,'Actual reviewed projector controller differs')
 need(subprocess.check_output(['git','-C',str(ROOT),'rev-parse','HEAD']).decode().strip()==controller,'Projector checkout differs')
 need(not subprocess.check_output(['git','-C',str(ROOT),'status','--porcelain','--untracked-files=all']).strip(),'Projector checkout dirty')
 q,state,out=contained_paths(args.quarantine,args.state,args.out,ROOT);need(not q.exists() and not out.exists(),'Original projector operation already started; no retry');q.mkdir();state.mkdir(exist_ok=True)
 raw=(DATA/'config.json').read_bytes();need(sha(raw)==CONFIG_SHA,'Reviewed sealed config changed');cfg=json.loads(raw);need(cfg['originalArtifactAccepted'] is False and cfg['newCaps']==dict(entries=160,expandedBytes=104857600,zipBytes=83886080),'No original acceptance or generic cap increase')
 values={}
 for role in ['inventory','whitelist','provenance']:
  b=(DATA/cfg[role+'Path']).read_bytes();need(sha(b)==cfg[role+'Sha256'],'Sealed '+role+' changed');values[role]=json.loads(b)
 need(values['inventory']['count']==3885 and values['inventory']['expandedBytes']==171528490 and values['whitelist']['count']==62 and values['whitelist']['expandedBytes']==17321182,'Exact refused/selected source facts changed')
 def api(path,label):
  r=subprocess.run(['gh','api','--hostname','github.com','repos/DocumentDrivenDX/weft/'+path],capture_output=True,timeout=60);(state/(label+'.json')).write_bytes(r.stdout);(state/(label+'.stderr')).write_bytes(r.stderr);need(r.returncode==0,'Actual CI API read failed');return json.loads(r.stdout)
 ownid=int(os.environ['GITHUB_RUN_ID']);attempt=int(os.environ['GITHUB_RUN_ATTEMPT']);own=api(f'actions/runs/{ownid}','projector-run');need(own['id']==ownid and own['head_sha']==controller and own['path']==WORKFLOW and own['event']=='workflow_dispatch' and own['head_branch']=='main' and own['repository']['full_name']=='DocumentDrivenDX/weft' and own['run_attempt']==attempt,'Actual projector run differs')
 ownjobs=api(f'actions/runs/{ownid}/attempts/{attempt}/jobs','projector-jobs')['jobs'];need(len(ownjobs)==1 and ownjobs[0]['name']=='retention-projector' and ownjobs[0]['head_sha']==controller and ownjobs[0]['run_id']==ownid and ownjobs[0]['run_attempt']==attempt,'Actual projector job differs');tree=api('git/commits/'+controller,'projector-commit')['tree']['sha']
 run=api('actions/runs/41729261593','original-run');need(run['id']==41729261593 and run['head_sha']==cfg['originalControllerV'] and run['run_attempt']==1 and run['status']=='completed' and run['conclusion']=='success' and run['workflow_id']==406265581 and run['path']=='.github/workflows/public-postflight-0457.yml' and run['head_branch']=='main' and run['event']=='workflow_dispatch' and run['repository']['full_name']=='DocumentDrivenDX/weft','Original consumer run differs')
 jobs=api('actions/runs/41729261593/attempts/1/jobs','original-jobs')['jobs'];need(len(jobs)==1,'Original job ambiguous');job=jobs[0];need(job['id']==126242032126 and job['head_sha']==cfg['originalControllerV'] and job['run_id']==41729261593 and job['run_attempt']==1 and job['conclusion']=='success','Original successful job differs')
 a=api('actions/artifacts/13275468435','original-artifact');need(a['id']==13275468435 and a['name']==values['provenance']['originalArtifactName'] and a['workflow_run']['id']==41729261593 and a['workflow_run']['head_sha']==cfg['originalControllerV'] and not a['expired'] and a['digest']=='sha256:'+cfg['rawZIPSha256'] and a['size_in_bytes']==cfg['rawZIPBytes'],'Original quarantined artifact differs')
 uploads=[s for s in job['steps'] if s['name']=='Retain original public postflight evidence and failures'];need(len(uploads)==1 and uploads[0]['conclusion']=='success','Original upload absent');stamp=lambda s:datetime.fromisoformat(s.replace('Z','+00:00'));need(stamp(uploads[0]['started_at'])<=stamp(a['created_at'])<=stamp(uploads[0]['completed_at']),'Original upload window differs')
 need(uploads[0]['started_at']==values['provenance']['uploadStartedAt'] and uploads[0]['completed_at']==values['provenance']['uploadCompletedAt'] and a['created_at']==values['provenance']['artifactCreatedAt'],'Sealed provenance changed')
 source=q/'original-rejected-13275468435.zip'
 with source.open('xb') as stream:r=subprocess.run(['gh','api','--hostname','github.com','repos/DocumentDrivenDX/weft/actions/artifacts/13275468435/zip'],stdout=stream,stderr=subprocess.PIPE,timeout=240)
 (state/'quarantine-download.stderr').write_bytes(r.stderr);need(r.returncode==0 and source.stat().st_size==cfg['rawZIPBytes'] and file_sha(source)==cfg['rawZIPSha256'],'Quarantined raw ZIP bytes changed')
 with zipfile.ZipFile(source) as z:payloads=verified_members(z,values['inventory'],values['whitelist'])
 identity=dict(kind='github-actions-retention-only-projector',controllerCommit=controller,controllerTree=tree,workflowPath=WORKFLOW,workflowId=own['workflow_id'],runId=ownid,jobId=ownjobs[0]['id'],attempt=attempt,scriptSha256=file_sha(__file__),configurationSha256=CONFIG_SHA,python=sys.version)
 record=dict(schema='weft-public-postflight-retention-projection/1',status='bounded-capsule-prepared-final-new-job-upload-auth-pending',projectionController=identity,originalConsumerProvenance=values['provenance'],originalArtifactAccepted=False,originalRefusal=dict(entries=3885,expandedBytes=171528490,limits=dict(entries=160,expandedBytes=104857600)),sealedOriginalInventorySha256=cfg['inventorySha256'],selectedOriginalMembers=values['whitelist'],excluded=dict(prefix='proof-venv/',entries=3823,decompressed=False,imported=False,executed=False),consumerReexecuted=False,newConsumerClaim=False,sourceR=cfg['sourceR'])
 provenance={'projection-provenance/original-terminal-run.json':(state/'original-run.json').read_bytes(),'projection-provenance/original-terminal-jobs.json':(state/'original-jobs.json').read_bytes(),'projection-provenance/original-artifact-api.json':(state/'original-artifact.json').read_bytes()}
 manifest=write_capsule(out,payloads,record,(DATA/cfg['inventoryPath']).read_bytes(),provenance,cfg['newCaps']);need(len(manifest['files'])+1==cfg['expectedOutputEntries'],'Exact68 output member roster differs')
 print('Bounded retention-only capsule prepared; originalArtifactAccepted=false; newConsumerClaim=false',flush=True)
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--controller-sha',required=True);p.add_argument('--quarantine',required=True,type=Path);p.add_argument('--state',required=True,type=Path);p.add_argument('--out',required=True,type=Path);main(p.parse_args())
