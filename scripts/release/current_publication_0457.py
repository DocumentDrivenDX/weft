"""Current3/42 publication binding before tag creation and normal OIDC upload.
Historical validators and all existing publisher guards remain independent.
"""
from pathlib import Path
import argparse,json,os,tempfile
from admit_current_sdk_0457 import authenticate as sdk,archive_files,sha,save,need,PROFILE
from admit_current_sources_0457 import authenticate as sources

def assemble(R,linux,mac,source_run,out):
 out=Path(out);need(not out.exists(),'Fresh qualification output required');out.mkdir(parents=True)
 a=sdk(R,linux,mac,out/'sdk-auth');b=sources(R,source_run,out/'source-auth')
 need(a['sourceTree']==b['sourceTree'],'Source/SDK tree differs')
 assets={};wheels={}
 for obs in a['actualGitHubObservations']:
  files=archive_files((out/'sdk-auth'/f'{obs["artifactID"]}.zip').read_bytes());c=json.loads(files['component.json']);name=c['wheel']
  need(name not in assets,'Duplicate wheel');assets[name]=files[name];wheels[name]=sha(files[name])
  for name0 in ['sdk-smoke.json','packaging.json','component.json']:
   assets[obs['target']+'-'+name0]=files[name0]
 for label,obs in b['components'].items():
  files=archive_files((out/'source-auth'/f'{obs["artifactID"]}.zip').read_bytes())
  manifest=json.loads(files['source-manifest.json' if label=='source' else 'wasm-manifest.json'])
  assets['current-'+label+'-manifest.json']=json.dumps(manifest,indent=2).encode()+b'\n'
  if label=='source':
   for role in ['sdist','sourceArchive']:
    v=manifest[role];name=Path(v['archive'] if role=='sdist' else v['path']).name
    matches=[raw for path,raw in files.items() if Path(path).name==name];need(len(matches)==1,'Source asset ambiguous');assets[name]=matches[0]
  else:
   for row in manifest['artifacts']:
    p=Path(row['path']);rel='raw-weft_wasm.wasm' if p.name=='weft_wasm.wasm' else '/'.join(p.parts[-2:]);assets['temporal-'+rel.replace('/','-')]=files[rel]
 for label,obj in [('sdk',a),('source',b)]:assets['current-'+label+'-admission.json']=json.dumps(obj,indent=2).encode()+b'\n'
 proof=dict(profile=PROFILE,sourceCommit=R,sourceTree=a['sourceTree'],linuxRun=linux,macosRun=mac,sourceRun=source_run,assetSha256={n:sha(raw) for n,raw in assets.items()},wheelSha256=wheels,publicationAdmission=False)
 save(out/'authenticated-assets.json',proof);return assets,proof

def stage(R,linux,mac,source_run,asset_dir,out):
 assets,proof=assemble(R,linux,mac,source_run,out);dest=Path(asset_dir)
 existing={p.name:sha(p.read_bytes()) for p in dest.glob('*.whl')}
 need(existing==proof['wheelSha256'],'Publisher wheels differ from authenticated SDK wheels')
 for name,raw in assets.items():
  p=dest/name
  if p.exists():need(p.read_bytes()==raw,'Existing publisher asset differs: '+name)
  else:p.write_bytes(raw)
 return proof

def verify_public(R,asset_dir,out):
 root=Path(asset_dir);pins=json.loads((root/'release-pins.json').read_bytes());proof=pins['currentQualification']
 from current_macos_pair_0457 import COHORT_PROFILE
 if proof.get('profile')==COHORT_PROFILE:
  from publish_macos_pair_0457 import verify_public as verify_exact_pairs
  return verify_exact_pairs(R,asset_dir,out)
 need(pins['version']=='0.4.57' and pins['sourceCommit']==pins['commit']==R and proof['profile']==PROFILE,'Current publication profile/source differs')
 assets,fresh=assemble(R,proof['linuxRun'],proof['macosRun'],proof['sourceRun'],out)
 # API observations may include fresh timestamps; bind immutable byte payloads,
 # not serialized transport snapshots, to their original successful jobs.
 for name,raw in assets.items():
  if name.endswith('-admission.json'):continue
  need((root/name).is_file() and (root/name).read_bytes()==raw,'Public asset differs from authenticated source: '+name)
 need(proof['wheelSha256']==fresh['wheelSha256'] and proof['sourceTree']==fresh['sourceTree'],'Qualified publication cohort differs')
 return fresh
if __name__=='__main__':
 p=argparse.ArgumentParser();p.add_argument('--source-sha',required=True);p.add_argument('--assets',required=True,type=Path);p.add_argument('--out',required=True,type=Path);a=p.parse_args();verify_public(a.source_sha,a.assets,a.out)
