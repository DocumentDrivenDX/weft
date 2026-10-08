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
 ('scan-type-filter','crates/weft-postgresql/src/candidate.rs','WHERE type_id={slot}::int','WHERE {slot}::int IS NOT NULL','postgresql-candidate','relational_emission_keeps_owner_selection_and_bag_projection'),
 ('projection-distinct','crates/weft-postgresql/src/candidate.rs','let mut sql = format!("SELECT {} FROM {from}", projection.join(", "));','let mut sql = format!("SELECT DISTINCT {} FROM {from}", projection.join(", "));','postgresql-candidate','relational_emission_keeps_owner_selection_and_bag_projection'),
 ('decimal-through-double','crates/weft-postgresql/src/conformance_original.rs','value.clone(),\n                serde_json::json!({"literalSpan":span}),','value.parse::<f64>().unwrap().to_string(),\n                serde_json::json!({"literalSpan":span}),','postgresql-original','admitted_decimal_precision_scale_pairs_preserve_exact_operands'),
 ('presence-null-to-absence','crates/weft-postgresql/src/presence_definition.rs','Some(v) if v.is_null() && !authored_nullable => Err(Diagnostic::new(','Some(v) if v.is_null() => Ok(Presence::Absent),\n            Some(v) if v.is_null() && !authored_nullable => Err(Diagnostic::new(','postgresql-lib','original_definition_drives_presence_without_coercion'),
 ('module-pin-guard','crates/weft-core/src/model.rs','if input.pin.sha256 != sha256(input.document_json.as_bytes()) {','if false && input.pin.sha256 != sha256(input.document_json.as_bytes()) {','qualification-properties','generated_unicode_retention_pins_and_selected_meaning'),
]
reports=[]
for name,file,before,after,test,filter in cases:
 dest=BASE/name;dest.mkdir(parents=True,exist_ok=True)
 subprocess.run(['rsync','-a','--exclude=.git','--exclude=target','--exclude=node_modules','--exclude=__pycache__',str(ROOT)+'/',str(dest)+'/'],check=True)
 p=dest/file;original=p.read_text();assert original.count(before)==1
 mutant=original.replace(before,after);p.write_text(mutant)
 env=os.environ.copy();env['CARGO_TARGET_DIR']='/private/tmp/weft-b007-mutation-target'
 cmd=['/private/tmp/weft-toolchain/cargo/bin/cargo','test','-p']
 if test=='postgresql-original':cmd+=['weft-postgresql','--lib','--features','conformance-original']
 elif test=='postgresql-lib':cmd+=['weft-postgresql','--lib']
 elif test=='postgresql-candidate':cmd+=['weft-postgresql','--test','candidate-compiler']
 else:cmd+=['weft-core','--test',test]
 cmd+=['--locked','--offline',filter,'--','--nocapture']
 run=subprocess.run(cmd,cwd=dest,env=env,capture_output=True,text=True)
 log=run.stdout+run.stderr;(OUT/(name+'.log')).write_text(log)
 assert run.returncode==101 and 'test result: FAILED.' in log and 'panicked at' in log,(name,run.returncode,log[-2000:])
 regression=dest/'tests/qualify-and-evolve/property-regressions.txt'
 if regression.exists():shutil.copyfile(regression,OUT/(name+'-mutant-regression.txt'))
 reports.append({'mutation':name,'source':file,'before':before,'after':after,'originalSha256':hashlib.sha256(original.encode()).hexdigest(),'mutantSha256':hashlib.sha256(mutant.encode()).hexdigest(),'test':test,'filter':filter,'exitCode':run.returncode,'status':'detected'})
 print(json.dumps(reports[-1]),flush=True)
(OUT/'summary.json').write_text(json.dumps({'status':'passed','detected':len(reports),'mutations':reports,'scope':'Source guards, SQL emission, numeric token and presence mutations; no native mutated-result qualification.'},indent=2)+'\n')
