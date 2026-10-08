"""@covers US-006-AC2 @covers US-006-AC4
Temporary source mutants. Never edits the active compiler checkout.
"""
import hashlib,json,os,pathlib,shutil,subprocess
ROOT=pathlib.Path(__file__).resolve().parents[2]
BASE=pathlib.Path('/private/tmp/weft-b007-mutants')
OUT=ROOT/'docs/helix/04-build/evidence/B-007-source-mutations';OUT.mkdir(exist_ok=True)
cases=[
 ('duplicate-key-guard','crates/weft-core/src/json.rs','if !keys.insert(key) {','if false && !keys.insert(key) {','qualification-resources','generated_truncated_json_and_nested_duplicates'),
 ('node-limit-guard','crates/weft-core/src/json.rs','if depth > 128 || *count >= 100_000 {','if depth > 128 || false {','qualification-resources','json_node_and_request_byte_limits'),
 ('module-pin-guard','crates/weft-core/src/model.rs','if input.pin.sha256 != sha256(input.document_json.as_bytes()) {','if false && input.pin.sha256 != sha256(input.document_json.as_bytes()) {','qualification-properties','generated_unicode_retention_pins_and_selected_meaning'),
]
reports=[]
for name,file,before,after,test,filter in cases:
 dest=BASE/name;dest.mkdir(parents=True,exist_ok=True)
 subprocess.run(['rsync','-a','--exclude=.git','--exclude=target','--exclude=node_modules','--exclude=__pycache__',str(ROOT)+'/',str(dest)+'/'],check=True)
 p=dest/file;original=p.read_text();assert original.count(before)==1
 mutant=original.replace(before,after);p.write_text(mutant)
 env=os.environ.copy();env['CARGO_TARGET_DIR']='/private/tmp/weft-b007-mutation-target'
 cmd=['/private/tmp/weft-toolchain/cargo/bin/cargo','test','-p','weft-core','--test',test,'--locked','--offline',filter,'--','--nocapture']
 run=subprocess.run(cmd,cwd=dest,env=env,capture_output=True,text=True)
 log=run.stdout+run.stderr;(OUT/(name+'.log')).write_text(log)
 assert run.returncode==101 and 'test result: FAILED.' in log and 'panicked at' in log,(name,run.returncode,log[-2000:])
 regression=dest/'tests/qualify-and-evolve/property-regressions.txt'
 if regression.exists():shutil.copyfile(regression,OUT/(name+'-mutant-regression.txt'))
 reports.append({'mutation':name,'source':file,'before':before,'after':after,'originalSha256':hashlib.sha256(original.encode()).hexdigest(),'mutantSha256':hashlib.sha256(mutant.encode()).hexdigest(),'test':test,'filter':filter,'exitCode':run.returncode,'status':'detected'})
 print(json.dumps(reports[-1]),flush=True)
(OUT/'summary.json').write_text(json.dumps({'status':'passed','detected':len(reports),'mutations':reports,'scope':'Three source guard mutations only; not all backend semantic mutations.'},indent=2)+'\n')
