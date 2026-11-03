"""Closed current sdist config/mode/transform rules; no Cargo or source mutation."""
from pathlib import PurePosixPath
import copy
MATURIN_SOURCE='https://github.com/PyO3/maturin/blob/v1.9.6/src/source_distribution.rs'
def need(value,message):
 if not value:raise ValueError(message)
def validate_sdist_configs(root,nested):
 expected=copy.deepcopy(nested)
 need(nested['project']['readme']=='README.md','Nested readme policy differs')
 need(nested['project']['license-files']==['LICENSE.md','LICENSE-MIT','LICENSE-APACHE','THIRD-PARTY-LICENSES.txt'],'Nested notice policy differs')
 need(nested['tool']['maturin'].get('include')==[dict(path='../../'+p+'/**',format='sdist') for p in ['tests','spec','.cargo']],'Nested inclusion policy differs')
 expected['project']['readme']='crates/weft-python/README.md'
 expected['project']['license-files']=['crates/weft-python/'+p for p in nested['project']['license-files']]
 expected['tool']['maturin']['manifest-path']='crates/weft-python/Cargo.toml'
 expected['tool']['maturin']['include']=[dict(path=p+'/**',format='sdist') for p in ['tests','spec','.cargo','vendor/ahash']]
 need(root==expected,'Root/nested sdist config metadata/features/discovery parity differs')
 need(root['build-system']==dict(requires=['maturin==1.9.6'],**{'build-backend':'maturin'}),'Pinned sdist backend differs')
 return dict(manifestPath='crates/weft-python/Cargo.toml',includes=expected['tool']['maturin']['include'],metadataFeaturesMatchNested=True)
def sdist_command(out):return ['maturin','sdist','--out',str(out)]
def regular_mode_permissions(raw_mode,entry_type,git_mode):
 # Keep the raw header separately. These are the ONLY numeric representations
 # admitted; permission or type-bit drift is never hidden by a broad mask.
 need(entry_type in (b'0',b'\0'),'Nonregular archive type')
 need(type(raw_mode) is int and git_mode in (None,'100644','100755'),'Invalid mapped Git/archive mode')
 expected=0o755 if git_mode=='100755' else 0o644
 need(raw_mode in (expected,0o100000|expected),'SDist mapped Git/generated mode differs')
 return expected

def expected_maturin196_workspace(original,metadata):
 # Independent reproduction of v1.9.6 find_path_deps + rewrite_cargo_toml,
 # scoped to this profile's explicit workspace members (no glob inference).
 packages={p['id']:p for p in metadata['packages']};nodes={n['id']:n for n in metadata['resolve']['nodes']}
 need(len(packages)==len(metadata['packages']) and len(nodes)==len(metadata['resolve']['nodes']),'Duplicate Maturin graph identity')
 selected=metadata['resolve']['root'];need(selected in packages and selected in nodes and packages[selected]['name']=='weft-python','Maturin metadata root differs')
 workspace=PurePosixPath(metadata['workspace_root']);manifest=PurePosixPath(packages[selected]['manifest_path'])
 need(workspace.is_absolute() and manifest.is_relative_to(workspace) and '..' not in manifest.parts,'External Maturin main manifest')
 main_member=manifest.parent.relative_to(workspace).as_posix();need(main_member=='crates/weft-python','Maturin main member differs')
 known={};todo=[selected]
 while todo:
  key=todo.pop();need(key in packages and key in nodes,'Missing Maturin path graph node')
  for dependency_id in nodes[key]['dependencies']:
   need(dependency_id in packages,'Missing Maturin dependency package')
   matches=[d for d in packages[key]['dependencies'] if d['name']==packages[dependency_id]['name']]
   need(bool(matches),'Missing Maturin dependency declaration');dependency=matches[0]
   if dependency.get('path') is None:continue
   name=dependency.get('rename') or dependency['name']
   if name in known:continue
   path=PurePosixPath(dependency['path']);need(path.is_absolute() and path.is_relative_to(workspace) and '..' not in path.parts,'External Maturin dependency path')
   known[name]=str(path/'Cargo.toml')
   found=[p['id'] for p in packages.values() if p['manifest_path']==known[name]]
   need(len(found)==1,'Missing/duplicate Maturin path package');todo.append(found[0])
 # Maturin adds this relative-path key before pruning the workspace manifest.
 known[main_member]=str(manifest)
 expected=copy.deepcopy(original);members=expected['workspace']['members']
 need(all(type(m) is str and not any(c in m for c in '*?[]') for m in members),'Review workspace glob packaging before execution')
 kept=[member for member in members if member in known]
 need(kept==['crates/weft-python'],'Pinned Maturin workspace result is not this closed Python-only profile')
 expected['workspace']['members']=kept
 return expected
