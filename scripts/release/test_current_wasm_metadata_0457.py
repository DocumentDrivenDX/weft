"""Captured actual admission serialization; pure mocks, no tools/Cargo/browser/API."""
import copy,json,os,sys,tempfile,unittest
from pathlib import Path
from unittest.mock import patch
import qualify_current_wasm_0457 as subject
CAPTURE={"sourceCommit":"21ea4b4315e7dbf32ebebde0609c1f586ce40f97","sourceTree":"8319c5d350f9b32d8f119c03a8b8dcb54d4d322e","canonicalUmf":"git+https://github.com/DocumentDrivenDX/umf.git?rev=e3f38ef6f05ae636d2837f9d69f8d6805222664b#e3f38ef6f05ae636d2837f9d69f8d6805222664b","cargoLockSha256":"92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d","resources":9,"features":["ashlar-sdk-ordered-page-candidate","raw-postgresql-reads","ashlar-databricks-paths-keys","ashlar-databricks-qualified","truss-postgresql-pg18","current-postgresql-scalar-candidate","current-databricks-scalar-candidate","current-query-refinement-candidate","current-presence-graph-requirements-candidate"]}
class Metadata(unittest.TestCase):
 def invoke(self,preflight):
  with tempfile.TemporaryDirectory() as td:
   out=Path(td);inputs=copy.deepcopy(CAPTURE)
   args=['qualify_current_wasm_0457.py','--out',td]+(['--preflight-only'] if preflight else [])
   with patch.object(sys,'argv',args),patch.dict(os.environ,{'GITHUB_SHA':CAPTURE['sourceCommit'],'GITHUB_RUN_ID':'41674740287'}),patch.object(subject,'verify',return_value=inputs),patch('verify_current_tools_0457.inspect') as tools,patch.object(subject.subprocess,'run',side_effect=RuntimeError('STOP-BEFORE-CARGO')) as run:
    if preflight:subject.main();tools.assert_not_called();run.assert_not_called()
    else:
     with self.assertRaisesRegex(RuntimeError,'STOP-BEFORE-CARGO'):subject.main()
     tools.assert_called_once();self.assertEqual(run.call_args.args[0][0:3],['cargo','metadata','--locked'])
     self.assertEqual(run.call_args.args[0][-1],json.loads((out/'inputs.json').read_text())['wasmBuild']['features'])
   value=json.loads((out/'inputs.json').read_text());self.assertEqual(inputs,CAPTURE);return value
 def test_actual_captured_admission_default_serializes_before_cargo(self):self.check(self.invoke(False))
 def test_cheap_preflight_serializes_without_any_builder(self):self.check(self.invoke(True))
 def test_admission_refusal_blocks_preflight(self):
  with tempfile.TemporaryDirectory() as td,patch.object(sys,'argv',['qualify_current_wasm_0457.py','--out',td,'--preflight-only']),patch.object(subject,'verify',side_effect=ValueError('ADMISSION-REFUSAL')),patch('verify_current_tools_0457.inspect') as tools,patch.object(subject.subprocess,'run') as run:
   with self.assertRaisesRegex(ValueError,'ADMISSION-REFUSAL'):subject.main()
   tools.assert_not_called();run.assert_not_called();self.assertFalse((Path(td)/'inputs.json').exists())
 def check(self,value):
  self.assertEqual(value['features'],CAPTURE['features']);self.assertEqual(value['wasmBuild'],dict(target='wasm32-unknown-unknown',features='raw-postgresql-reads,'+','.join('weft-runtime/'+x for x in CAPTURE['features'])))
  for k,v in CAPTURE.items():self.assertEqual(value[k],v)
 def test_workflow_serialization_before_expensive_builders(self):
  workflow=(Path(__file__).resolve().parents[2]/'.github/workflows/temporal-source-0457.yml').read_text().split('  temporal-wasm:',1)[1]
  preflight='python scripts/release/qualify_current_wasm_0457.py --preflight-only --out "$RUNNER_TEMP/current-wasm"'
  self.assertEqual(workflow.count(preflight),1)
  self.assertLess(workflow.index('Exact-main R input admission before tools'),workflow.index(preflight))
  for step in ['Pinned WASM builders','oven-sh/setup-bun@v2','actions/setup-node@v4','Pinned browser dependencies','Build corrected raw-temporal WASM and execute Node and real Chromium']:self.assertLess(workflow.index(preflight),workflow.index(step))
if __name__=='__main__':unittest.main()
