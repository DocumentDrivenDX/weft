"""@covers US-008-AC4 Fail-closed source mutation qualification."""
from __future__ import annotations
import contextlib, dataclasses, hashlib, json, pathlib, shutil, tempfile
from .process import run, Result
class MutationError(Exception):pass
cases=[
 ('duplicate-key-guard','crates/weft-core/src/json.rs','if !keys.insert(key) {','if false && !keys.insert(key) {','qualification-resources','generated_truncated_json_and_nested_duplicates'),
 ('node-limit-guard','crates/weft-core/src/json.rs','if depth > 128 || *count >= 100_000 {','if depth > 128 || false {','qualification-resources','json_node_and_request_byte_limits'),
 ('scan-type-filter','crates/weft-postgresql/src/candidate.rs','WHERE type_id={slot}::int','WHERE {slot}::int IS NOT NULL','postgresql-candidate','relational_emission_keeps_owner_selection_and_bag_projection'),
 ('projection-distinct','crates/weft-postgresql/src/candidate.rs','let mut sql = format!("SELECT {} FROM {from}", projection.join(", "));','let mut sql = format!("SELECT DISTINCT {} FROM {from}", projection.join(", "));','postgresql-candidate','relational_emission_keeps_owner_selection_and_bag_projection'),
 ('decimal-through-double','crates/weft-postgresql/src/conformance_original.rs','value.clone(),\n                serde_json::json!({"literalSpan":span}),','value.parse::<f64>().unwrap().to_string(),\n                serde_json::json!({"literalSpan":span}),','postgresql-original','admitted_decimal_precision_scale_pairs_preserve_exact_operands'),
 ('presence-null-to-absence','crates/weft-postgresql/src/presence_definition.rs','Some(v) if v.is_null() && !authored_nullable => Err(Diagnostic::new(','Some(v) if v.is_null() => Ok(Presence::Absent),\n            Some(v) if v.is_null() && !authored_nullable => Err(Diagnostic::new(','postgresql-lib','original_definition_drives_presence_without_coercion'),
 ('module-pin-guard','crates/weft-core/src/model.rs','if input.pin.sha256 != sha256(input.document_json.as_bytes()) {','if false && input.pin.sha256 != sha256(input.document_json.as_bytes()) {','qualification-properties','generated_unicode_retention_pins_and_selected_meaning'),
]
signatures={
 'duplicate-key-guard':['resources.rs:33:', 'on an `Ok` value: Object'],
 'node-limit-guard':['resources.rs:16:', 'on an `Ok` value: Array'],
 'scan-type-filter':['owner type selection must survive lowering'],
 'projection-distinct':['projection must retain duplicate names'],
 'decimal-through-double':['10000000000000000','9999999999999999'],
 'presence-null-to-absence':['on an `Ok` value: Absent'],
 'module-pin-guard':['WFT-PIN','left: `Null`'],
}

def apply_mutation(original:str,before:str,after:str) -> str:
    if original.count(before)!=1 or before==after:raise MutationError()
    mutated=original.replace(before,after)
    if mutated==original:raise MutationError()
    return mutated

def detected(result:Result, name:str, selected:str) -> bool:
    required=set(signatures[name]) | {'test result: FAILED.','panicked at'}
    return (not result.timed_out and result.exit_code==101 and result.selected_tests==1 and selected in result.failed_filters and required<=result.found and result.failed==1 and result.passed==0)

def baseline_clean(result:Result) -> bool:
    return not result.timed_out and result.exit_code==0 and result.selected_tests==1 and result.passed==1 and result.failed==0 and result.ignored==0

def execute(config,root:pathlib.Path,diagnostics=None) -> dict:
    config.validate_repository(root)
    identity=config.tool_identity(root) # before any output/temp side effect
    config.temp_root.mkdir(parents=True,exist_ok=True)
    config.output_root.mkdir(parents=True,exist_ok=True)
    reports=[]
    with tempfile.TemporaryDirectory(prefix='weft-mutants-',dir=config.temp_root) as temporary:
        work=pathlib.Path(temporary)
        for name,file,before,after,test,selected in cases:
            with diagnostics.operation_context(name) if diagnostics else contextlib.nullcontext():
                dest=work/name;target=work/(name+'-target')
                shutil.copytree(root,dest,ignore=shutil.ignore_patterns('.git','target','node_modules','dist','__pycache__'))
                path=dest/file;original=path.read_bytes().decode("utf8");mutated=apply_mutation(original,before,after);# mutation is written only after a clean selected-test baseline
                env=config.command_environment();env['CARGO_TARGET_DIR']=str(target)
                command=[str(config.cargo),'test','-p']
                if test=='postgresql-original':command+=['weft-postgresql','--lib','--features','conformance-original']
                elif test=='postgresql-lib':command+=['weft-postgresql','--lib']
                elif test=='postgresql-candidate':command+=['weft-postgresql','--test','candidate-compiler']
                else:command+=['weft-core','--test',test]
                command+=['--locked','--offline',selected,'--','--nocapture']
                needles=tuple(signatures[name]+['test result: FAILED.','panicked at'])
                baseline=run(command,dest,env,config.timeout_seconds,(),(selected,))
                if not baseline_clean(baseline):raise MutationError()
                path.write_text(mutated)
                result=run(command,dest,env,config.timeout_seconds,needles,(selected,))
                if not detected(result,name,selected):raise MutationError()
                reports.append({'mutation':name,'source':file,'originalSha256':hashlib.sha256(original.encode()).hexdigest(),'mutantSha256':hashlib.sha256(mutated.encode()).hexdigest(),'test':test,'filter':selected,'baseline':{'exitCode':baseline.exit_code,'selected':baseline.selected_tests,'passed':baseline.passed,'failed':baseline.failed,'ignored':baseline.ignored,'durationMs':baseline.duration_ms},'exitCode':result.exit_code,'status':'detected','durationMs':result.duration_ms})
    return {'version':'weft-runner/1','status':'passed','detected':len(reports),'toolIdentity':identity,'mutations':reports,'scope':'Exact source guards and selected test failure signatures. No native mutated-result qualification; raw subprocess output is omitted.'}
