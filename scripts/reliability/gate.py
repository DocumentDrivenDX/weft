"""Scoped formal gate with actual Rust correspondence; no native authority."""
import dataclasses,hashlib,json,os,pathlib,sys
if __package__ in (None,''):sys.path.insert(0,str(pathlib.Path(__file__).resolve().parents[1]))
from reliability.config import load_config
from reliability.process import run
ROOT=pathlib.Path(__file__).resolve().parents[2]
CORRESPONDENCE={
 'WFT-FM-001':[('security_admission','core08_source_custody_never_activates_uninterpreted_security'),('security_admission','unsupported_security_does_not_invoke_backend_factory')],
 'WFT-FM-002':[('lib','security_composition::tests::all_permit_require_forbid_truths_and_rule_order'),('lib','security_composition::tests::portable_typescript_truth_corpus_correspondence')],
 'WFT-FM-003':[('lib','security_composition::tests::all_permit_require_forbid_truths_and_rule_order')],
 'WFT-FM-004':[('lib','security_composition::tests::bounded_disposition_oracle_correspondence'),('lib','security_composition::tests::missing_protected_disposition_false_permit_and_mask_conflict'),('lib','security_composition::tests::transform_identity_uses_exact_value_and_qualified_output_domain')],
 'WFT-FM-005':[('security_admission','source_packet_cannot_be_reused_with_changed_model_bytes'),('security_admission','rehashed_catalog_input_cannot_substitute_for_prepared_definitions'),('security_admission','invalid_source_digest_refuses_before_security_gate')],
}
FILES=['crates/weft-core/src/compile.rs','crates/weft-core/src/model.rs','crates/weft-core/src/security_source.rs','crates/weft-core/src/security_ir.rs','crates/weft-core/src/security_composition.rs','crates/weft-core/tests/security_admission.rs','crates/weft-core/tests/security-source-fixture.json','crates/weft-core/tests/security-composition-oracle.json','crates/weft-core/tests/security-disposition-oracle.json','scripts/reliability/formal.py','scripts/reliability/gate.py','scripts/reliability/requirements.txt','docs/helix/02-design/contracts/CONTRACT-007-security-compilation.md','docs/helix/03-test/fixtures/cases.json','docs/helix/02-design/contracts/compile-response-v0.5.schema.json','docs/helix/02-design/technical-designs/TD-009-security-compilation.md','Cargo.toml','Cargo.lock','rust-toolchain.toml']
def snapshot(root):
 result={}
 for name in FILES:
  path=root/name
  if path.is_symlink() or not path.is_file():raise RuntimeError()
  result[name]=hashlib.sha256(path.read_bytes()).hexdigest()
 return result

def correspondence(config,root):
 results={}
 for target,name in sorted({pair for values in CORRESPONDENCE.values() for pair in values}):
  command=[str(config.cargo),'test','-p','weft-core','--lib'] if target=='lib' else [str(config.cargo),'test','-p','weft-core','--test',target]
  command+=['--locked','--offline',name,'--','--exact']
  env=config.command_environment();env['CARGO_TARGET_DIR']=str(config.temp_root/'formal-target')
  result=run(command,root,env,config.timeout_seconds,(),(name,))
  if result.timed_out or result.exit_code!=0 or result.selected_tests!=1 or result.passed!=1 or result.failed!=0 or result.ignored!=0:raise RuntimeError()
  results[target+':'+name]={'exitCode':result.exit_code,'selected':result.selected_tests,'passed':result.passed,'durationMs':result.duration_ms}
 return results

def main():
 try:
  config=load_config(sys.argv[1:]);identity=config.tool_identity(ROOT)
  from reliability.formal import check
  from reliability.diagnostics import Run,source_revision
  frozen=snapshot(ROOT);revision=source_revision(ROOT)
  diagnostics=Run(config,'formal-analysis',revision)
  try:
   with diagnostics.operation_context('formal-analysis'):
    result=check(ROOT);result['actualRustCorrespondence']=correspondence(config,ROOT)
    if snapshot(ROOT)!=frozen or source_revision(ROOT)!=revision:raise RuntimeError()
  except Exception:diagnostics.close(failed=True);raise
  if diagnostics.close()!='passed':raise RuntimeError()
  result['propertyTests']=CORRESPONDENCE;result['toolIdentity']=identity;result['diagnosticRunId']=diagnostics.id
  result['inputs']=frozen;result['compilerAndHostRevision']=revision
  path=diagnostics.directory/'qualification.json'
  with open(path,'x',opener=lambda name,flags:os.open(name,flags,0o600)) as stream:json.dump(result,stream,indent=2);stream.write('\n')
  print(json.dumps({'version':'weft-formal/1','status':'passed','properties':len(result['properties']),'rustTests':len(result['actualRustCorrespondence'])}));return 0
 except Exception:print('weft-runner: formal qualification failed',file=sys.stderr);return 1
if __name__=='__main__':sys.exit(main())
