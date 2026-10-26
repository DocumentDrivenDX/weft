# SPDX-License-Identifier: MIT OR Apache-2.0
"""Collect native dependency license notices from locked Cargo metadata."""
import json, sys
from pathlib import Path
metadata = json.loads(Path(sys.argv[1]).read_text())
packages = {p['id']: p for p in metadata['packages']}
nodes = {n['id']: n for n in metadata['resolve']['nodes']}
root = next(k for k,p in packages.items() if p['name']=='weft-python')
seen=set()
def visit(key):
    if key in seen:return
    seen.add(key)
    for dep in nodes[key]['deps']:
        if any(k['kind'] in (None,'build') for k in dep['dep_kinds']):visit(dep['pkg'])
visit(root)
text=['Third-party notices for umf-weft. Dependencies retain their original licenses.\n']
missing=[]
for key in sorted(seen):
    p=packages[key]
    if p['source'] is None and p['name']!='ahash':continue
    directory=Path(p['manifest_path']).parent
    files=sorted({f for pattern in ['LICENSE*','LICENCE*','COPYING*','NOTICE*','license*'] for f in directory.glob(pattern) if f.is_file()})
    if p.get('license_file'):
        f=directory/p['license_file']
        if f not in files:files.append(f)
    if p['name']=='r-efi':files=[directory/'AUTHORS']
    if p['name'] in ('uuid-simd','vsimd'):files=[Path('scripts/python-release/upstream-licenses/simd-LICENSE')]
    if not files:
        vcs=json.loads((directory/'.cargo_vcs_info.json').read_text())
        candidates=[q for q in packages.values() if q.get('repository')==p.get('repository')]
        for q in candidates:
            other=Path(q['manifest_path']).parent
            info=other/'.cargo_vcs_info.json'
            if info.exists() and json.loads(info.read_text())['git']==vcs['git']:
                files.extend(f for pattern in ['LICENSE*','license*','COPYING*'] for f in other.glob(pattern) if f.is_file())
                if files:break
    if not p.get('license') or not files:missing.append(p['name'])
    text.append('\n=== '+p['name']+' '+p['version']+' | '+str(p.get('license'))+' ===\n')
    for f in files:text.extend([f.name+'\n',f.read_text()+'\n'])
if missing:raise RuntimeError('Missing license evidence: '+str(missing))
Path('crates/weft-python/THIRD-PARTY-LICENSES.txt').write_text(''.join(text))
print(json.dumps({'dependencies':len(seen),'status':'passed'}))
