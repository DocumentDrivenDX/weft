"""Cheap actual CLI/lock coupling, before any expensive current WASM build."""
from pathlib import Path
import json,subprocess,tomllib
from verify_current_inputs_0457 import need

def inspect(root,out,*,wasm):
 out.mkdir(parents=True,exist_ok=True);rows={}
 commands={'rust':['rustc','--version','--verbose'],'cargo':['cargo','--version','--verbose'],'maturin':['maturin','--version']}
 if wasm:commands.update(bindgen=['wasm-bindgen','--version'],bun=['bun','--version'],node=['node','--version'])
 for name,argv in commands.items():
  r=subprocess.run(argv,cwd=root,capture_output=True,text=True);(out/(name+'.stdout')).write_text(r.stdout);(out/(name+'.stderr')).write_text(r.stderr)
  need(r.returncode==0,'Tool preflight failed: '+name);rows[name]=dict(argv=argv,exitCode=r.returncode,version=r.stdout.strip())
 need(rows['rust']['version'].startswith('rustc 1.90.0 '),'Rust version differs')
 need(rows['cargo']['version'].startswith('cargo 1.90.0 '),'Cargo version differs')
 need(rows['maturin']['version']=='maturin 1.9.6','Maturin version differs')
 if wasm:
  lock=tomllib.loads((root/'Cargo.lock').read_text());crates=[p['version'] for p in lock['package'] if p['name']=='wasm-bindgen']
  need(crates==['0.2.105'],'Locked wasm-bindgen crate differs')
  need(rows['bindgen']['version']=='wasm-bindgen '+crates[0],'CLI/locked wasm-bindgen crate mismatch')
  need(rows['bun']['version']=='1.4.2','Bun version differs')
  need(rows['node']['version'].startswith('v24.'),'Pinned Node24 major differs')
  need(json.loads((root/'package.json').read_text())['devDependencies']['playwright']=='1.62.1','Declared Playwright pin differs')
 rows['scope']='Observed actual CLI versions; wasm CLI matches frozen lock crate before build'
 (out/'tools.json').write_text(json.dumps(rows,indent=2)+'\n');return rows
