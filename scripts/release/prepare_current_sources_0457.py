"""Guarded source-only archive production. Never repairs mutated source/lock."""
from pathlib import Path,PurePosixPath
import argparse,base64,copy,email.parser,gzip,hashlib,io,json,os,stat,subprocess,tarfile,tomllib,time
from verify_current_inputs_0457 import verify,sha,need
from current_sdist_policy_0457 import validate_sdist_configs,sdist_command,regular_mode_permissions,expected_maturin196_workspace

def save(p,v):Path(p).write_text(json.dumps(v,indent=2)+'\n')
def run(root,args,out,name):
 row=dict(name=name,argv=list(map(str,args)),cwd=str(root),started=time.time())
 with (out/(name+'.stdout')).open('wb') as a,(out/(name+'.stderr')).open('wb') as b:r=subprocess.run(row['argv'],cwd=root,stdout=a,stderr=b)
 row.update(exitCode=r.returncode,finished=time.time())
 with (out/'commands.jsonl').open('a') as log:log.write(json.dumps(row)+'\n')
 need(r.returncode==0,'Source-only command failed: '+name)
def roster(root,R):
 rows={}
 for line in subprocess.check_output(['git','ls-tree','-r','-z',R],cwd=root).split(b'\0'):
  if not line:continue
  meta,path=line.split(b'\t');mode,kind,blob=meta.decode().split();rows[path.decode()]=dict(mode=mode,kind=kind,blob=blob)
 return rows
def actual_inputs(root,R):
 rows=roster(root,R);actual={}
 for name,row in rows.items():
  p=root/name;st=p.lstat();need(row['kind']=='blob' and stat.S_ISREG(st.st_mode),'Unsupported tracked source type: '+name)
  raw=p.read_bytes();need(hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==row['blob'],'Tracked source bytes differ: '+name)
  need(bool(st.st_mode&0o111)==(row['mode']=='100755'),'Tracked source mode differs: '+name)
  actual[name]=dict(sha256=hashlib.sha256(raw).hexdigest(),bytes=len(raw),mode=row['mode'])
 return actual

def gitbytes(root,R,path):return subprocess.check_output(['git','show',R+':'+path],cwd=root)
def members(path,expected_root=None,with_modes=False,with_headers=False):
 data={};modes={};headers={};roots=set()
 # Inflate once for header capture; no per-member backward gzip seeks.
 tar_raw=gzip.decompress(Path(path).read_bytes()) if with_headers else None
 with (tarfile.open(fileobj=io.BytesIO(tar_raw),mode='r:') if with_headers else tarfile.open(path,'r:gz')) as t:
  for m in t:
   p=PurePosixPath(m.name);roots.add(p.parts[0]);need(expected_root is None or p.parts[0]==expected_root,'Archive root identity differs');need(not p.is_absolute() and '..' not in p.parts and '\\' not in m.name,'Unsafe archive member')
   if m.isdir():continue
   need(m.isfile() and m.type in (tarfile.REGTYPE,tarfile.AREGTYPE),'Unsupported archive source type')
   name='/'.join(p.parts[1:]);need(name not in data,'Duplicate archive source')
   data[name]=t.extractfile(m).read();modes[name]=m.mode
   if with_headers:headers[name]=dict(entryTypeHex=m.type.hex(),rawMode=oct(m.mode),bytes=m.size,rawHeaderChainBase64=base64.b64encode(tar_raw[m.offset:m.offset_data]).decode('ascii'))
 need(len(roots)==1,'Archive has multiple roots')
 return (data,modes,headers) if with_headers else ((data,modes) if with_modes else data)

REVIEWED_LOCAL_ROOTS={
 'crates/weft-core':('weft-core','0.1.0'),
 'crates/weft-databricks':('weft-databricks','0.1.0'),
 'crates/weft-postgresql':('weft-postgresql','0.1.0'),
 'crates/weft-python':('weft-python','0.4.57'),
 'crates/weft-runtime':('weft-runtime','0.1.0'),
 'vendor/ahash':('ahash','0.8.12'),
}

def selected_local_roots(root,R,metadata):
 # Actual --locked Cargo metadata is the graph input, NOT a caller root list.
 root=Path(root).resolve();rows=roster(root,R)
 need(metadata.get('workspace_root')==str(root),'Selected Cargo workspace root differs from frozen source')
 packages={p['id']:p for p in metadata['packages']};nodes={n['id']:n for n in metadata['resolve']['nodes']}
 need(len(packages)==len(metadata['packages']) and len(nodes)==len(metadata['resolve']['nodes']),'Duplicate selected Cargo graph identity')
 selected=metadata['resolve'].get('root')
 need(selected in packages and selected in nodes and packages[selected]['name']=='weft-python','Actual selected Cargo root is not weft-python')
 workspace=tomllib.loads(gitbytes(root,R,'Cargo.toml').decode())
 need(workspace.get('patch',{}).get('crates-io',{}).get('ahash')=={'path':'vendor/ahash'},'Frozen R ahash patch is not the exact reviewed root')
 todo=[selected];seen=set();local={}
 while todo:
  key=todo.pop()
  if key in seen:continue
  need(key in packages and key in nodes,'Selected Cargo graph edge is missing')
  seen.add(key);package=packages[key]
  if package.get('source') is None:
   raw_path=package.get('manifest_path');need(type(raw_path) is str,'Local manifest path absent')
   manifest=Path(raw_path)
   need(manifest.is_absolute() and manifest.as_posix()==raw_path and '..' not in manifest.parts and manifest.is_relative_to(root),'External/noncanonical local manifest root')
   relative=manifest.relative_to(root);native=relative.parent.as_posix()
   need(relative.name=='Cargo.toml' and native in REVIEWED_LOCAL_ROOTS,'Unreviewed selected local root: '+native)
   need(native not in local,'Substituted/duplicate selected local root: '+native)
   cursor=root
   for part in relative.parts:
    cursor=cursor/part;need(not cursor.is_symlink(),'Symlink selected local root: '+native)
   path=relative.as_posix()
   need(path in rows and rows[path]['kind']=='blob' and rows[path]['mode'] in ['100644','100755'],'Local manifest is not a regular frozen R entry')
   need(manifest.is_file(),'Missing selected local manifest')
   raw=manifest.read_bytes()
   need(hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==rows[path]['blob'],'Local manifest bytes differ from frozen R')
   need(manifest.read_bytes()==raw and bool(manifest.stat().st_mode&0o111)==(rows[path]['mode']=='100755'),'Local manifest differs from frozen R')
   declared=tomllib.loads(raw.decode())['package'];version=declared.get('version')
   if type(version) is dict:
    need(version=={'workspace':True},'Unsupported inherited local version')
    version=workspace['workspace']['package']['version']
   expected=REVIEWED_LOCAL_ROOTS[native]
   need((package.get('name'),package.get('version'))==expected==(declared.get('name'),version),'Selected local package identity differs from frozen R/review: '+native)
   prefix='path+'+manifest.parent.as_uri()+'#'
   need(key in [prefix+version,prefix+declared['name']+'@'+version],'Substituted Cargo local source ID: '+native)
   if native=='vendor/ahash':need(declared.get('build')=='build.rs','Frozen ahash build script changed')
   local[native]=key
  for edge in nodes[key]['deps']:
   kinds=edge.get('dep_kinds')
   need(type(kinds) is list and bool(kinds) and all(type(k) is dict and 'kind' in k and k['kind'] in [None,'build','dev'] for k in kinds),'Unknown selected Cargo dependency kind')
   if any(k['kind']!='dev' for k in kinds):todo.append(edge['pkg'])
 need(set(local)==set(REVIEWED_LOCAL_ROOTS),'Selected local root roster missing/extra reviewed roots')
 need(local['crates/weft-python']==selected,'Actual root package identity differs')
 return sorted(local)

AHASH_GIT_R_FILES=('.github/workflows/rust.yml', '.gitignore', 'Cargo.toml', 'Cargo.toml.orig', 'FAQ.md', 'LICENSE-APACHE', 'LICENSE-MIT', 'README.md', 'build.rs', 'rustfmt.toml', 'src/aes_hash.rs', 'src/convert.rs', 'src/fallback_hash.rs', 'src/hash_map.rs', 'src/hash_quality_test.rs', 'src/hash_set.rs', 'src/lib.rs', 'src/operations.rs', 'src/random_state.rs', 'src/specialize.rs', 'tests/bench.rs', 'tests/map_tests.rs', 'tests/nopanic.rs')

def source_configs(root,R):
 nested=tomllib.loads(gitbytes(root,R,"crates/weft-python/pyproject.toml").decode())
 original=tomllib.loads(gitbytes(root,R,"pyproject.toml").decode())
 validate_sdist_configs(original,nested)
 return original

def checked_source_snapshot(root):
 result={}
 for path in sorted(root.rglob('*')):
  mode=path.lstat().st_mode
  need(not path.is_symlink() and (stat.S_ISDIR(mode) or stat.S_ISREG(mode)),'Archive resolver introduced unsafe source entry')
  if stat.S_ISREG(mode):result[path.relative_to(root).as_posix()]=dict(sha256=sha(path),mode=oct(stat.S_IMODE(mode)))
 return result

def selected_metadata_command(features):
 return ["cargo","metadata","--locked","--format-version","1","--manifest-path","crates/weft-python/Cargo.toml","--filter-platform","x86_64-unknown-linux-gnu","--features",features]

def declared_sdist_inventory(root,R,native_roots):
 # Maturin1.9.6 uses cargo package --list for each native crate, then explicit
 # pyproject includes/metadata. Derive the roster BEFORE inspecting the archive.
 need(set(native_roots)==set(REVIEWED_LOCAL_ROOTS) and len(native_roots)==6,'Exact six selected local roots required')
 before=roster(root,R);frozen=actual_inputs(root,R)
 status=subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=root)
 selected={};package_lists={}
 def add(target,source):
  need(source in before and before[source]['kind']=='blob' and before[source]['mode'] in ['100644','100755'],'Declared source missing/unsafe in R: '+source)
  need(target not in selected or selected[target]==source,'Conflicting declared archive mapping: '+target)
  selected[target]=source
 try:
  for native in sorted(native_roots):
   need(native in REVIEWED_LOCAL_ROOTS and native+'/Cargo.toml' in before,'Unreviewed selected local root: '+native)
   if native=='vendor/ahash':
    actual={name.removeprefix('vendor/ahash/') for name in before if name.startswith('vendor/ahash/')}
    need(actual==set(AHASH_GIT_R_FILES),'Exact Git-derived ahash23 roster differs')
    for name in AHASH_GIT_R_FILES:add('vendor/ahash/'+name,'vendor/ahash/'+name)
    continue # Git-R declared inventory, NOT Cargo package-list output.
   raw=subprocess.check_output(['cargo','package','--list','--allow-dirty','--manifest-path',native+'/Cargo.toml'],cwd=root)
   listed=raw.decode('utf-8').splitlines();need(len(listed)==len(set(listed)),'Duplicate Cargo package entry');package_lists[native]=listed
   add(native+'/Cargo.toml',native+'/Cargo.toml')
   for name in listed:
    path=PurePosixPath(name)
    need(bool(name) and not path.is_absolute() and '..' not in path.parts and '\\' not in name and path.as_posix()==name,'Unsafe Cargo package entry')
    # Generated Cargo entries have no original source and are not copied by
    # Maturin. The main crate pyproject is relocated to the sdist root.
    if name in ['Cargo.toml.orig','Cargo.toml'] or native=='crates/weft-python' and name=='pyproject.toml':continue
    if path.suffix in ['.pyc','.pyd','.so']:continue
    source=native+'/'+name
    if source not in before:
     need(not (root/source).exists(),'Untracked Cargo package input: '+source)
     continue
    add(source,source)
 finally:
  need(actual_inputs(root,R)==frozen and roster(root,R)==before and subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=root)==status,'Cargo inventory source/lock mutation; refuse without restoring')
 py=source_configs(root,R)
 policy=py['tool']['maturin']
 need(policy.get('sdist-generator','cargo')=='cargo' and not policy.get('exclude') and not policy.get('python-source') and not policy.get('python-packages'),'Review unsupported sdist generator/exclusion/Python-source policy')
 for source in before:
  if source.startswith(('tests/','spec/','.cargo/')):add(source,source)
 for name in ['Cargo.toml','Cargo.lock','pyproject.toml']:add(name,name)
 project=py['project']
 for name in [project['readme']]+project['license-files']:add(name,name)
 # Retain the original mandatory native source/resource requirement as well.
 for source in before:
  if any(source.startswith(v+'/src/') or source.startswith(v+'/resources/') or source==v+'/build.rs' for v in native_roots):add(source,source)
 for name in ['Cargo.toml','build.rs','src/lib.rs','README.md','LICENSE-MIT','LICENSE-APACHE']:
  add('vendor/ahash/'+name,'vendor/ahash/'+name)
 selected['PKG-INFO']=None
 return selected,package_lists

def verify_sdist(root,R,archive,metadata):
 native_roots=selected_local_roots(root,R,metadata)
 declared,package_lists=declared_sdist_inventory(root,R,native_roots)
 data,modes,headers=members(archive,expected_root='umf_weft-0.4.57',with_headers=True);before=roster(root,R)
 need(set(data)==set(declared),'SDist declared inclusion policy misses required entries or has substituted/extra source: missing='+repr(sorted(set(declared)-set(data)))+' extra='+repr(sorted(set(data)-set(declared))))
 for name,source in declared.items():
  try:regular_mode_permissions(modes[name],bytes.fromhex(headers[name]['entryTypeHex']),before[source]['mode'] if source is not None else None)
  except ValueError as error:raise ValueError(str(error)+': '+name) from error
 need(data.get('Cargo.lock')==(root/'Cargo.lock').read_bytes(),'SDist Cargo.lock is not byte-identical')
 needed=set()
 for p in before:
  if any(p.startswith(v+'/src/') or p.startswith(v+'/resources/') or p in [v+'/Cargo.toml',v+'/build.rs'] for v in native_roots):needed.add(p)
 need(needed<=data.keys(),'SDist misses native source/manifests/resources')
 original_py=source_configs(root,R)
 for name,raw in data.items():
  if name=='PKG-INFO':
   meta=email.parser.BytesParser().parsebytes(raw)
   need(meta.get_all('Name')==['umf-weft'] and meta.get_all('Version')==['0.4.57'],'SDist metadata identity differs')
   need(meta.get('License-Expression')=='MIT OR Apache-2.0','SDist license metadata differs')
   need(meta.get_all('License-File')==original_py['project']['license-files'],'SDist license path metadata differs')
   need(meta.get('Requires-Python')=='>=3.9' and not meta.get_all('Requires-Dist'),'SDist Python/dependency metadata differs')
   continue
  if name=='pyproject.toml':
   need(raw==gitbytes(root,R,'pyproject.toml'),'SDist root pyproject bytes differ from frozen production config')
   continue
  if name=='Cargo.toml':
   original_workspace=tomllib.loads(gitbytes(root,R,'Cargo.toml').decode())
   expected=expected_maturin196_workspace(original_workspace,metadata);actual=tomllib.loads(raw.decode())
   need(actual==expected,'SDist workspace/pins changed beyond native-member inclusion policy')
   continue
  if name=='crates/weft-python/Cargo.toml':
   expected=tomllib.loads(gitbytes(root,R,name).decode());actual=tomllib.loads(raw.decode());expected['package']['readme']='README.md'
   need(actual==expected,'SDist Python crate manifest changes beyond readme metadata')
   continue
  mapped=declared[name]
  need(mapped in before,'SDist substituted/extra source: '+name)
  need(any(mapped.startswith(v+'/') for v in native_roots) or mapped.startswith(('tests/','spec/','.cargo/')) or mapped in ['Cargo.lock','pyproject.toml'],'SDist source outside declared inclusion policy: '+name)
  need(hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==before[mapped]['blob'],'SDist source/dependency manifest/resource bytes differ: '+name)
 # Dependency manifests, including all canonical UMF pins, MUST remain literal bytes.
 for native in native_roots:
  if native!='crates/weft-python':need(data[native+'/Cargo.toml']==gitbytes(root,R,native+'/Cargo.toml'),'Dependency manifest was rewritten')
 return dict(archive=str(archive),sha256=sha(archive),sourceFiles=len(data),requiredNativeFiles=len(needed),cargoLockSha256=sha(root/'Cargo.lock'),inclusion='Five Cargo-listed workspace crates plus independent Git-R ahash23 and root sdist-only tests/spec/.cargo. Root config byte-identical; workspace/Python-readme metadata transforms pinned to Maturin1.9.6; PKG-INFO explicitly validated',allOtherBytesMatchR=True,selectedLocalRoots=native_roots,completeDeclaredInventory=True,mappedGitModesMatchR=True,declaredInventory={name:dict(source=source,mode=oct(modes[name])) for name,source in sorted(declared.items())},cargoPackageLists=package_lists,gitDerivedLocalDependency=dict(root='vendor/ahash',name='ahash',version='0.8.12',policy='Independent declared Git-R23; NOT Cargo package output',R=R,files=list(AHASH_GIT_R_FILES)),rawTarHeaders=headers)

def verify_canonical_git_inventory(metadata,expected,root,R):
 # Complete resolved/selected Git closure and manifest declarations have
 # different scopes. Validate BOTH against their own unchanged frozen records.
 packages={p['id']:p for p in metadata['packages']}
 need(len(packages)==len(metadata['packages']),'Duplicate canonical metadata package ID')
 git_rows=lambda values:sorted((p['name'],p['version'],p['source']) for p in values if (p.get('source') or '').startswith('git+'))
 locked=sorted((p['name'],p['version'],p['sourceID']) for p in expected['lockGitPackages'])
 actual=git_rows(packages.values())
 need(actual==locked,'Complete canonical locked Git package inventory differs')
 nodes={n['id']:n for n in metadata['resolve']['nodes']}
 need(len(nodes)==len(metadata['resolve']['nodes']),'Duplicate selected graph identity')
 selected=metadata['resolve'].get('root')
 need(selected in packages and selected in nodes and packages[selected]['name']=='weft-python','Actual selected inventory root is not weft-python')
 todo=[selected];seen=set()
 while todo:
  key=todo.pop()
  if key in seen:continue
  need(key in packages and key in nodes,'Selected inventory graph edge missing');seen.add(key)
  for edge in nodes[key]['deps']:
   kinds=edge.get('dep_kinds')
   need(type(kinds) is list and bool(kinds) and all(type(k) is dict and k.get('kind','invalid') in [None,'build','dev'] for k in kinds),'Unknown selected inventory dependency kind')
   if any(k['kind']!='dev' for k in kinds):todo.append(edge['pkg'])
 need(git_rows(packages[k] for k in seen)==locked,'Complete selected normal/build Git closure differs')
 declared=expected['packages']
 need(len(declared)==8 and len({p['packageID'] for p in declared})==8,'Exact two dependencies/six patches inventory differs')
 for item in declared:
  matches=[p for p in packages.values() if p['name']==item['name'] and p['version']==item['version']]
  need(len(matches)==1 and matches[0]['id']==item['packageID'] and matches[0].get('source')==item['sourceID'],'Declared canonical package/source ID differs: '+item['name'])
 core=tomllib.loads(gitbytes(root,R,'crates/weft-core/Cargo.toml').decode())
 workspace=tomllib.loads(gitbytes(root,R,'Cargo.toml').decode())
 direct=[(name,pin) for section in ['dependencies','build-dependencies'] for name,pin in core.get(section,{}).items() if isinstance(pin,dict) and ('git' in pin or name in ['umf-core','umf-codegen'])]
 patches=[(name,pin) for name,pin in workspace['patch']['crates-io'].items() if isinstance(pin,dict) and 'git' in pin]
 need(len(direct)==2 and {n for n,p in direct}=={'umf-core','umf-codegen'} and len(patches)==6,'Actual two dependency/six patch declarations differ')
 observed=[]
 for name,pin in direct+patches:
  package=pin.get('package',name);matches=[p for p in declared if p['name']==package and (pin.get('version') is None or pin['version']=='='+p['version'])]
  need(len(matches)==1 and 'path' not in pin and 'git' in pin and 'rev' in pin,'Declared package/version/source differs: '+name)
  item=matches[0]
  need((name,pin) in direct or pin.get('version')=='='+item['version'],'Patch version is not exact')
  source='git+'+pin['git']+'?rev='+pin['rev']+'#'+pin['rev']
  observed.append((package,item['version'],source))
 need(sorted(observed)==sorted((p['name'],p['version'],p['sourceID']) for p in declared),'Actual declaration inventory differs from frozen eight')
 return dict(completeLockedGitPackages=[dict(name=n,version=v,sourceID=s) for n,v,s in actual],selectedNormalBuildGitCount=len(locked),declaredPackageIDs=[p['packageID'] for p in declared],completeLockedGitCount=len(actual),declaredTwoDependenciesSixPatchesCount=len(declared))

def main():
 p=argparse.ArgumentParser();p.add_argument('--out',type=Path,required=True);a=p.parse_args();out=a.out.resolve();out.mkdir(parents=True,exist_ok=True);root=Path(__file__).resolve().parents[2]
 need(not out.is_relative_to(root),'Source output must be external')
 need(all(p.name in ['phase.json','preflight.json'] for p in out.iterdir()),'Fresh source output required')
 before=verify(root);R=before['sourceCommit'];source_configs(root,R);save(out/'source-before.json',before);save(out/'tracked-before.json',roster(root,R));save(out/'actual-inputs-before.json',actual_inputs(root,R))
 run(root,['cargo','--version','--verbose'],out,'cargo-version');run(root,['rustc','--version','--verbose'],out,'rust-version');run(root,['maturin','--version'],out,'maturin-version')
 need((out/'maturin-version.stdout').read_text().strip()=='maturin 1.9.6','Pinned Maturin differs')
 features=','.join(before['features'])
 run(root,selected_metadata_command(features),out,'locked-metadata')
 expected=json.loads(Path(__file__).with_name('current-inputs-0457.json').read_text());m=json.loads((out/'locked-metadata.stdout').read_text())
 save(out/'source-git-inventory.json',verify_canonical_git_inventory(m,expected,root,R))
 save(out/'locked-selected-features.json',m['resolve'])
 selected_local_roots(root,R,m) # refuse unreviewed/substituted local roots before production
 need(verify(root)==before,'Preflight source/lock mutation')
 distribution=out/'sdist';distribution.mkdir(exist_ok=False)
 # Source-only CLI has NO --locked option. Never pretend that it does.
 try:
  run(root,sdist_command(distribution),out,'maturin-sdist')
 finally:
  save(out/'immediate-source-after-command.json',dict(head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),status=subprocess.check_output(['git','status','--porcelain','--untracked-files=all'],cwd=root,text=True),actualLockSha256=sha(root/'Cargo.lock') if (root/'Cargo.lock').is_file() else None,sourceRestoreAttempted=False))
  save(out/'actual-inputs-immediate-after.json',actual_inputs(root,R))
 after=verify(root);need(after==before and roster(root,R)==json.loads((out/'tracked-before.json').read_text()),'Source/lock changed; refuse without restoring')
 save(out/'source-after.json',after);save(out/'tracked-after.json',roster(root,R));save(out/'actual-inputs-after.json',actual_inputs(root,R));need((out/'actual-inputs-before.json').read_bytes()==(out/'actual-inputs-after.json').read_bytes(),'Tracked input hash mutation')
 sdists=list(distribution.glob('*.tar.gz'));need(len(sdists)==1,'SDist output roster differs')
 proof=verify_sdist(root,R,sdists[0],m);save(out/'sdist-correspondence.json',proof)
 checked=out/'checked-sdist';checked.mkdir(exist_ok=False)
 checked_data,checked_modes=members(sdists[0],with_modes=True)
 for name,raw in checked_data.items():
  p=checked/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw);p.chmod(stat.S_IMODE(checked_modes[name]))
 archive_lock=sha(checked/'Cargo.lock')
 archive_before=checked_source_snapshot(checked);save(out/'checked-source-before.json',archive_before)
 try:
  run(checked,selected_metadata_command(features),out,'archive-locked-metadata')
 finally:
  archive_after=checked_source_snapshot(checked);save(out/'checked-source-after.json',archive_after)
  need(archive_after==archive_before,'Archive resolver source/mode/roster mutation; refuse without restoring')
 need(sha(checked/'Cargo.lock')==archive_lock==before['cargoLockSha256'],'Archive resolver mutated/changed lock')
 archived=json.loads((out/'archive-locked-metadata.stdout').read_text())
 save(out/'archive-git-inventory.json',verify_canonical_git_inventory(archived,expected,root,R))
 from verify_archived_resolution_0457 import verify_archived_resolution
 save(out/'archive-selected-resolution.json',verify_archived_resolution(m,archived,root,checked))
 save(out/'archive-selected-features.json',archived['resolve'])
 # Full repository source archive is separate and always names final R.
 archive=out/('weft-source-'+R+'.tar.gz')
 with archive.open('wb') as f:subprocess.run(['git','archive','--format=tar.gz','--prefix=weft-'+R+'/',R],cwd=root,stdout=f,check=True)
 content,modes=members(archive,expected_root='weft-'+R,with_modes=True);rows=roster(root,R);need(set(content)==set(rows),'Full source archive roster differs from R')
 for name,raw in content.items():
  need(hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==rows[name]['blob'],'Full source archive bytes differ from R: '+name)
  need(bool(modes[name]&0o111)==(rows[name]['mode']=='100755'),'Full source archive executable mode differs')
 need(verify(root)==before,'Final source mutation; refuse without restoring')
 save(out/'source-manifest.json',dict(sourceCommit=R,sourceTree=before['sourceTree'],producerRunId=int(os.environ['GITHUB_RUN_ID']),producerRunAttempt=int(os.environ['GITHUB_RUN_ATTEMPT']),producerJobName=os.environ['GITHUB_JOB'],workflowCommit=os.environ['GITHUB_SHA'],status='passed',cargoLockSha256=before['cargoLockSha256'],sdist=proof,sourceArchive=dict(path=str(archive),sha256=sha(archive),files=len(content)),maturin='1.9.6',sourceOnly=True,archiveCorrespondenceVerified=True,publicationAdmission=False))
if __name__=='__main__':main()
