#!/usr/bin/env python3
"""Enforce actual Cargo workspace dependency direction; no source regex proof.

Rust compilation owns symbol visibility; within-crate responsibility and external
integration semantics remain explicit semantic review obligations.
"""
import argparse,json,subprocess,sys
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]

def check(metadata,policy):
    if set(policy)!={'format','modules'} or policy['format']!='weft-cargo-boundaries/0.1':raise ValueError('Unknown boundary policy')
    allowed=policy['modules'];members=set(metadata['workspace_members'])
    packages={p['name']:p for p in metadata['packages']if p['id']in members}
    if set(packages)!=set(allowed):raise ValueError('Complete named workspace module inventory required')
    graph={name:set()for name in packages}
    for name,p in packages.items():
        for dependency in p['dependencies']:
            target=dependency['name']
            if target in packages:
                expected=Path(packages[target]['manifest_path']).resolve().parent
                if not dependency.get('path') or Path(dependency['path']).resolve()!=expected:
                    raise ValueError('Workspace dependency identity/path differs: '+name+' -> '+target)
                graph[name].add(target)
                if target not in allowed[name]:raise ValueError('Forbidden Cargo dependency: '+name+' -> '+target)
            elif dependency.get('path'):
                raise ValueError('Unmapped local dependency: '+name+' -> '+target)
    active=set();done=set()
    def visit(name):
        if name in active:raise ValueError('Cargo module cycle: '+name)
        if name in done:return
        active.add(name)
        for target in sorted(graph[name]):visit(target)
        active.remove(name);done.add(name)
    for name in sorted(graph):visit(name)
    return {'format':policy['format'],'workspaceModules':len(packages),'edges':[[n,t]for n in sorted(graph)for t in sorted(graph[n])],
            'qualification':'Actual Cargo manifest edges; Rust compiler enforces symbol visibility; within-crate/external meaning requires review'}

def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--manifest-path',type=Path,default=ROOT/'Cargo.toml');args=parser.parse_args()
    policy=json.loads((ROOT/'scripts/checks/module-boundaries.json').read_bytes())
    result=subprocess.run(['cargo','metadata','--manifest-path',str(args.manifest_path),'--no-deps','--format-version','1','--locked','--offline'],check=True,capture_output=True,text=True)
    print(json.dumps(check(json.loads(result.stdout),policy),sort_keys=True))
if __name__=='__main__':
    try:main()
    except (ValueError,subprocess.CalledProcessError)as error:print(str(error),file=sys.stderr);raise SystemExit(1)
