"""Fresh pinned CLI/native-wheel/Chromium qualification with input reconciliation."""
import argparse,hashlib,json,os,pathlib,sys,zipfile
if __package__ in (None,''):sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[1]))
from reliability.config import load_config
from reliability.process import run
ROOT=pathlib.Path(__file__).resolve().parents[2]
def validate_reports(out,wheel,built=None):
 summaries={name:json.loads((out/name).read_bytes()) for name in ('cli-summary.json','python-summary.json','browser-summary.json','resource-summary.json','security-summary.json','browser-resource-security-summary.json')}
 for name in ('cli-summary.json','python-summary.json','browser-summary.json'):
  value=summaries[name]
  if value.get('cases')!=2181 or (name!='browser-summary.json' and value.get('status')!='passed'):raise RuntimeError()
 for name in ('python-summary.json','browser-summary.json','browser-resource-security-summary.json'):
  if summaries[name].get('byteParity') is not True:raise RuntimeError()
 security=summaries['security-summary.json']
 if security.get('libraryByteParity') is not True or security.get('cliResponseParityCases')!=12 or security.get('cliInputLimitRefusals')!=1:raise RuntimeError()
 for name in ('resource-summary.json','security-summary.json','browser-resource-security-summary.json'):
  if summaries[name].get('status')!='passed' or summaries[name].get('cases')!=(7 if name=='resource-summary.json' else 13):raise RuntimeError()
 if summaries['security-summary.json'].get('securityCases')!=6 or summaries['security-summary.json'].get('resourceCases')!=7 or summaries['python-summary.json'].get('subprocessDisabled') is not True:raise RuntimeError()
 cli=json.loads((out/'cli-reports.json').read_bytes());expected={r['id']:hashlib.sha256(r['raw'].encode()).hexdigest() for r in cli}
 if len(cli)!=2181 or len(expected)!=2181:raise RuntimeError()
 for name in ('python-receipts.json','browser-receipts.json'):
  records=json.loads((out/name).read_bytes())['cases']
  if len(records)!=2181 or len({r['id'] for r in records})!=2181 or {r['id'] for r in records}!=set(expected):raise RuntimeError()
  for r in records:
   if r['actualSha256']!=expected[r['id']] or r['expectedSha256']!=expected[r['id']]:raise RuntimeError()
 native=pathlib.Path(summaries['python-summary.json']['nativeModule']).resolve()
 if not native.is_relative_to(pathlib.Path(sys.prefix).resolve()):raise RuntimeError()
 with zipfile.ZipFile(wheel) as package:
  libraries=[n for n in package.namelist() if n.endswith(('.so','.pyd','.dylib'))]
  if len(libraries)!=1 or hashlib.sha256(package.read(libraries[0])).hexdigest()!=summaries['python-summary.json']['extensionSha256'] or hashlib.sha256(native.read_bytes()).hexdigest()!=summaries['python-summary.json']['extensionSha256']:raise RuntimeError()
 if summaries['security-summary.json']['extensionSha256']!=summaries['python-summary.json']['extensionSha256'] or summaries['resource-summary.json']['extensionSha256']!=summaries['python-summary.json']['extensionSha256']:raise RuntimeError()
 if summaries['browser-resource-security-summary.json']['wasmSha256']!=summaries['browser-summary.json']['wasmSha256']:raise RuntimeError()
 if built is not None:
  if summaries['cli-summary.json']['compilerSha256']!=built['batch'] or summaries['resource-summary.json']['binarySha256']!=built['cli'] or summaries['browser-summary.json']['wasmSha256']!=built['wasm'] or summaries['browser-resource-security-summary.json']['wrapperSha256']!=built['wrapper']:raise RuntimeError()
 return summaries

def main():
 try:
  parser=argparse.ArgumentParser(add_help=False);parser.add_argument('--bun',required=True);parser.add_argument('--maturin',required=True);parser.add_argument('--wasm-bindgen',required=True);parser.add_argument('--chromium')
  options,args=parser.parse_known_args();config=load_config(args);identity=config.tool_identity(ROOT)
  supplied=[pathlib.Path(getattr(options,n)) for n in ('bun','maturin','wasm_bindgen')]
  if any(not p.is_absolute() for p in supplied):raise RuntimeError()
  handles=[p.resolve(strict=True) for p in supplied]
  if any(not p.is_absolute() or p.is_symlink() or not p.is_file() for p in handles):raise RuntimeError()
  bun,maturin,bindgen=map(str,handles)
  env=config.command_environment();env.update(PYO3_PYTHON=sys.executable,CARGO_TARGET_DIR=str(config.temp_root/'fresh-target'))
  for command,expected in [([bun,'--version'],'1.4.2'),([maturin,'--version'],'maturin 1.9.6'),([bindgen,'--version'],'wasm-bindgen 0.2.105'),([bun,'node_modules/typescript/bin/tsc','--version'],'Version 5.9.3')]:
   version=run(command,ROOT,env,5,capture_limit=256)
   if version.timed_out or version.exit_code or version.bytes_read>256 or version.captured.decode().strip()!=expected:raise RuntimeError()
  from reliability.custody import snapshot,verify,historical
  from reliability.diagnostics import Run,source_revision
  inputs=snapshot(ROOT);history=historical(ROOT);diagnostics=Run(config,'fresh-hosts',source_revision(ROOT))
  checks=[];out=config.temp_root/'fresh-host-evidence';target=config.temp_root/'fresh-target'
  def execute(command,extras=None):
   with diagnostics.operation_context('fresh-hosts'):
    result=run(command,ROOT,dict(env,**(extras or {})),config.timeout_seconds)
    if result.timed_out or result.exit_code:raise RuntimeError()
    checks.append({'argv':command,'exitCode':0,'durationMs':result.duration_ms})
  try:
   if target.exists():raise RuntimeError()
   out.mkdir(parents=True,exist_ok=False)
   features='truss-postgresql-qualified,ashlar-databricks-qualified'
   execute([str(config.cargo),'build','-p','weft-runtime','--bin','weft-runtime','--example','compile_public_batch','--features',features,'--locked','--offline'])
   execute([maturin,'build','--manifest-path','crates/weft-python/Cargo.toml','--features',features,'--locked','--offline','--out',str(out/'wheels')])
   wheels=list((out/'wheels').glob('*.whl'))
   if len(wheels)!=1:raise RuntimeError()
   execute([sys.executable,'-I','-m','pip','install','--force-reinstall','--no-deps',str(wheels[0])])
   execute([str(config.cargo),'build','-p','weft-wasm','--features',features,'--target','wasm32-unknown-unknown','--locked','--offline'])
   execute([bindgen,str(target/'wasm32-unknown-unknown/debug/weft_wasm.wasm'),'--target','web','--out-dir',str(out/'web')])
   execute([bun,'run','browser:build'])
   sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
   built_paths={'cli':target/'debug/weft-runtime','batch':target/'debug/examples/compile_public_batch','wheel':wheels[0],'wasm':out/'web/weft_wasm_bg.wasm','glue':out/'web/weft_wasm.js','wrapper':ROOT/'packages/weft-browser/dist/index.js'}
   built={name:sha(path) for name,path in built_paths.items()}
   common={'WEFT_QUALIFIED_HOST_OUT':str(out),'WEFT_QUALIFIED_PUBLIC_BINARY':str(target/'debug/examples/compile_public_batch')}
   execute([sys.executable,'-I','tests/qualify-and-evolve/prepare-qualified-host-corpus.py'],common)
   execute([sys.executable,'-I','tests/qualify-and-evolve/qualified-host-python.py'],common)
   web={'WEFT_PROBE_API':'compile_json','WEFT_PROBE_JS':str(out/'web/weft_wasm.js'),'WEFT_PROBE_WASM':str(out/'web/weft_wasm_bg.wasm'),'WEFT_BROWSER_WRAPPER':str(ROOT/'packages/weft-browser/dist/index.js'),'WEFT_FRONTEND_CORPUS':str(out/'cases.jsonl'),'WEFT_FRONTEND_REPORTS':str(out/'cli-reports.json'),'WEFT_FRONTEND_BROWSER_SUMMARY':str(out/'browser-summary.json'),'WEFT_BROWSER_RECEIPTS':str(out/'browser-receipts.json')}
   if options.chromium:web['WEFT_CHROMIUM_EXECUTABLE']=options.chromium
   execute([bun,'tests/compile/browser-check.mjs'],web)
   resource={'WEFT_RESOURCE_BINARY':str(target/'debug/weft-runtime'),'WEFT_RESOURCE_CASES':str(out/'resource-cases.json'),'WEFT_RESOURCE_SUMMARY':str(out/'resource-summary.json')}
   execute([sys.executable,'-I','tests/qualify-and-evolve/host-resources.py'],resource)
   execute([sys.executable,'-I','scripts/reliability/fresh_cases.py'],dict(resource,WEFT_FRESH_HOST_OUT=str(out)))
   browser_resource={'WEFT_RESOURCE_CASES':str(out/'resource-security-cases.json'),'WEFT_RESOURCE_JS':str(out/'web/weft_wasm.js'),'WEFT_RESOURCE_WASM':str(out/'web/weft_wasm_bg.wasm'),'WEFT_RESOURCE_WRAPPER':str(ROOT/'packages/weft-browser/dist/index.js'),'WEFT_RESOURCE_SUMMARY':str(out/'browser-resource-security-summary.json'),'WEFT_RESOURCE_SCOPE':'Fresh WASM/native library byte parity for7resource and6blocked-security controls; CLI parity covers6resource and6security responses, with1separate input-limit transport refusal. No database/security enforcement.'}
   if options.chromium:browser_resource['WEFT_CHROMIUM_EXECUTABLE']=options.chromium
   execute([bun,'tests/qualify-and-evolve/host-resources-browser.mjs'],browser_resource)
   summaries=validate_reports(out,wheels[0],built)
   if any(sha(path)!=built[name] for name,path in built_paths.items()):raise RuntimeError()
   verify(ROOT,inputs)
   artifact_hashes={str(p.relative_to(out)):sha(p) for p in out.rglob('*') if p.is_file()}
  except Exception:diagnostics.close(failed=True);raise
  if diagnostics.close()!='passed':raise RuntimeError()
  sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
  result={'version':'weft-fresh-hosts/1','status':'passed','toolIdentity':identity,'inputManifest':inputs,'historicalSourceCustody':history,'checks':checks,'summaries':summaries,'artifacts':artifact_hashes,'builtArtifacts':built,'cliExecutableSha256':built['cli'],'batchExecutableSha256':built['batch'],'diagnosticRunId':diagnostics.id,'scope':'Fresh local CLI, installed native wheel and actual Chromium WASM parity across2181ordinary native-qualified fixtures;12resource/security CLI response-parity controls,1CLI input-limit refusal and13Python/WASM library-parity controls. Retained native evidence custody only; no current database execution, released package or native security qualification.'}
  with open(diagnostics.directory/'qualification.json','x',opener=lambda name,flags:os.open(name,flags,0o600)) as f:json.dump(result,f,indent=2);f.write('\n')
  print(json.dumps({'version':result['version'],'status':'passed','ordinaryCases':2181,'refusalCases':13,'inputFiles':len(inputs['files'])}));return 0
 except Exception:print('weft-runner: fresh host qualification failed',file=sys.stderr);return 1
if __name__=='__main__':sys.exit(main())
