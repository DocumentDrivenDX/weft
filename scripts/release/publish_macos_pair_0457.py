"""Explicit current0457 R/Q/A publication; normal immutable asset/OIDC gates."""
from pathlib import Path
import argparse,json,os,re,shutil,sys,tempfile
from current_macos_pair_0457 import R,RTREE,COHORT_PROFILE,REGISTRY,need,load_json
from admit_controller_pairs_0457 import authenticate
from admit_current_sdk_0457 import sha,save
def select_publication(event,ref,inputs,registered):
 result=dict(produce=False,publish=False)
 if event!='workflow_dispatch' or ref!='refs/heads/main':return result
 flag=inputs.get('current_pair_publish',False)
 if not (flag is True or flag=='true'):return result
 need(inputs.get('current_final') is False and inputs.get('tag')=='v0.4.57','Pair publication cannot schedule a producer or another release')
 need(inputs.get('source_sha')==inputs.get('macos_source_sha')==R,'Closed pair publication source differs')
 need(all(re.fullmatch('[1-9][0-9]*',str(inputs.get(k,''))) is not None for k in ['packaging_run_id','macos_run_id','current_arm64_run_id','current_source_run_id']),'Pair publication must reuse all completed producer/source runs')
 need(registered.get('status')=='reviewed' and registered.get('sourceCommit')==R and registered.get('sourceTree')==RTREE,'Exact registration not reviewed')
 return dict(produce=False,publish=True)

def assemble(registration_commit,linux,mac,arm,source_run,out):
 proof,assets=authenticate(registration_commit,linux,mac,arm,source_run,out)
 assets['current-controller-pair-admission.json']=json.dumps(proof,indent=2).encode()+b'\n'
 assets['current-controller-pair-registration.json']=json.dumps(proof['registration'],indent=2).encode()+b'\n'
 return assets,proof

def verify_public(source,asset_dir,out):
 need(source==R,'Only exact registered0457 product R admitted');root=Path(asset_dir);pins=load_json((root/'release-pins.json').read_bytes());p=pins['currentQualification']
 need(pins.get('tag')=='v0.4.57' and pins.get('commit')==pins.get('sourceCommit')==R and pins.get('version')=='0.4.57' and pins.get('distribution')=='umf-weft','Pair publication product identity differs')
 need(p.get('profile')==COHORT_PROFILE and p.get('sourceCommit')==R and p.get('sourceTree')==RTREE,'No general differing-controller fallback')
 # Registration is trusted tracked controller policy, not a public caller envelope.
 registered=load_json(Path(__file__).with_name(Path(REGISTRY).name).read_bytes())
 need(p.get('registration')==registered and registered.get('status')=='reviewed','Public pair differs from tracked exact registration')
 assets,fresh=assemble(p['registrationCommit'],p['linuxRun'],p['macosRun'],p['arm64Run'],p['sourceRun'],out)
 need(fresh['wheelSha256']==p['wheelSha256'],'Public qualified wheel cohort differs')
 for name,raw in assets.items():
  if name.endswith('-admission.json'):continue
  need((root/name).is_file() and (root/name).read_bytes()==raw,'Public original payload differs: '+name)
 return fresh

def publish(registration_commit,linux,mac,arm,source_run,dist,*,execute=False):
 import publish_hosted_release as legacy
 from publish_pypi_release import validate_archive
 need(os.environ.get('GITHUB_EVENT_NAME')=='workflow_dispatch' and os.environ.get('GITHUB_REF')=='refs/heads/main' and os.environ.get('GITHUB_SHA')==registration_commit,'Explicit exact-registration main publisher required')
 need(os.environ.get('GH_REPOSITORY')=='DocumentDrivenDX/weft' and os.environ.get('RELEASE_TAG')=='v0.4.57','Closed0457 publication repository/tag differs')
 root=Path(__file__).resolve().parents[2]
 head=legacy.command(['git','-C',str(root),'rev-parse','HEAD']).stdout.strip();need(head==registration_commit,'Publisher code is not registration commit')
 need(not legacy.command(['git','-C',str(root),'status','--porcelain','--untracked-files=all']).stdout.strip(),'Publisher controller checkout dirty')
 dist=Path(dist).resolve();need(not dist.is_relative_to(root),'Fresh publisher output outside controller checkout required');dist.mkdir(parents=True,exist_ok=False)
 assets,proof=assemble(registration_commit,linux,mac,arm,source_run,dist/'admission');asset_dir=dist/'release-assets';asset_dir.mkdir()
 for name,raw in assets.items():need(Path(name).name==name,'Unsafe publication payload name');(asset_dir/name).write_bytes(raw)
 wheels=sorted(asset_dir.glob('*.whl'));sdists=list(asset_dir.glob('umf_weft-0.4.57*.tar.gz'));source_archives=list(asset_dir.glob('weft-source-'+R+'.tar.gz'))
 need(len(wheels)==3 and len(sdists)==len(source_archives)==1,'Required3 wheels and original R source archives incomplete')
 for f in wheels+sdists:validate_archive(f,'umf-weft','0.4.57')
 note=root/'docs/helix/05-deploy/releases/v0.4.57.md';need(note.is_file(),'Governed qualification amendment missing');shutil.copyfile(note,asset_dir/note.name)
 sums=''.join(sha(f.read_bytes())+'  '+f.name+'\n' for f in sorted(wheels+sdists));(asset_dir/'SHA256SUMS').write_text(sums)
 pins=dict(schema='weft-hosted-release-pins/1',tag='v0.4.57',commit=R,sourceCommit=R,metadataCommit=registration_commit,workflowCommit=registration_commit,workflowRunId=int(os.environ['GITHUB_RUN_ID']),version='0.4.57',distribution='umf-weft',currentQualification=proof,wheelSha256=proof['wheelSha256'],qualification='Closed R/Q/A actual installed3 wheel SDK42 and original R source/WASM proof; Mac9 refusal parity. No native database/security authority.')
 save(asset_dir/'release-pins.json',pins);save(dist/'release-pins.json',pins);(dist/'SHA256SUMS').write_text(sums)
 manifest=dict(schema='weft-release-assets/1',tag='v0.4.57',version='0.4.57',sourceCommit=R,metadataCommit=registration_commit,status='verified-before-publication',expectedWheelCount=3,wheels=proof['wheelSha256'],assetSha256={f.name:sha(f.read_bytes()) for f in asset_dir.iterdir() if f.is_file()})
 save(asset_dir/'release-assets-0.4.57.json',manifest)
 if not execute:return proof
 # The existing normal immutable publisher's authenticated API/readback helpers
 # remain mandatory. No twine/direct-index bypass is introduced.
 repository='DocumentDrivenDX/weft';tag='v0.4.57';releases=legacy.list_tag_releases(repository,tag);need(len(releases)<=1,'Duplicate release tags');release=releases[0] if releases else None
 if release is not None:need(release.get('draft') is True,'Published release is immutable; no asset replacement')
 target=legacy.api_tag_commit(repository,tag)
 if target is None:
  created=legacy.command(['gh','api','--method','POST',f'repos/{repository}/git/refs','--field','ref=refs/tags/'+tag,'--field','sha='+R],check=False)
  need(legacy.api_tag_commit(repository,tag)==R,'Exact product R tag creation/readback failed')
 else:need(target==R,'Existing tag is not product R')
 if release is None:
  result=legacy.command(['gh','api','--method','POST',f'repos/{repository}/releases','--field','tag_name='+tag,'--field','target_commitish='+R,'--field','name='+tag,'--field','body='+note.read_text(),'-F','draft=true','-F','prerelease=false'],timeout=120);release=json.loads(result.stdout)
 release_id=release.get('id');need(type(release_id) is int,'Draft release ID absent')
 draft=json.loads(legacy.command(['gh','api',f'repos/{repository}/releases/{release_id}']).stdout);legacy.verify_release_api(draft,release_id=release_id,tag=tag,draft=True)
 legacy.command([sys.executable,str(root/'scripts/release/sync_release_assets.py'),'--tag',tag,'--asset-dir',str(asset_dir)])
 expected={f.name:sha(f.read_bytes()) for f in asset_dir.iterdir() if f.is_file()};draft=json.loads(legacy.command(['gh','api',f'repos/{repository}/releases/{release_id}']).stdout);legacy.verify_release_api(draft,release_id=release_id,tag=tag,draft=True,expected_digests=expected)
 need(legacy.api_tag_commit(repository,tag)==R,'Product tag changed before publication')
 legacy.command(['gh','api','--method','PATCH',f'repos/{repository}/releases/{release_id}','-F','draft=false','--raw-field','make_latest=true'],timeout=120)
 public=json.loads(legacy.command(['gh','api',f'repos/{repository}/releases/{release_id}']).stdout);rows=legacy.verify_release_api(public,release_id=release_id,tag=tag,draft=False,expected_digests=expected,immutable=True)
 save(dist/'release-api-assets.json',dict(status='passed',tag=tag,sourceCommit=R,metadataCommit=registration_commit,assets=rows));return proof

if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--registration-sha',required=True);p.add_argument('--linux-run',required=True,type=int);p.add_argument('--macos-run',required=True,type=int);p.add_argument('--arm64-run',required=True,type=int);p.add_argument('--source-run',required=True,type=int);p.add_argument('--dist',required=True,type=Path);p.add_argument('--publish-reviewed',action='store_true');a=p.parse_args();publish(a.registration_sha,a.linux_run,a.macos_run,a.arm64_run,a.source_run,a.dist,execute=a.publish_reviewed)
