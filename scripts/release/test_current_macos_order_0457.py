"""Source-only Mac producer ordering; no workflow, API or build execution."""
from pathlib import Path
import os,unittest
WORKFLOW=Path(os.environ.get('WEFT_MACOS_WORKFLOW',Path(__file__).resolve().parents[2]/'.github/workflows/release-hosted-macos.yml'))
class MacProducerOrder(unittest.TestCase):
 def setUp(self):self.source=WORKFLOW.read_text()
 def position(self,name):
  marker='      - name: '+name+'\n';self.assertEqual(self.source.count(marker),1,name);return self.source.index(marker)
 def test_admission_then_tools_then_wheel_then_sdk_upload_then_parity(self):
  names=['Require exact reviewed main source and ARM64 runner','Current final-R source, S and frozen-lock preflight','Admit actual current R source/archive and temporal WASM producer','Install bounded Rust and Python build tools','Build and check exact installed ARM64 wheel','Current42 SDK and actual installed final-R correspondence','Preserve Mac producer partials at their original attempt','Emit current final-R Mac wheel and SDK evidence immediately','Freeze nine refusal-response compiler vectors','Build Rust comparator and browser WASM using exact wheel features']
  positions=[self.position(n) for n in names];self.assertEqual(positions,sorted(positions),'Admission must precede tools; SDK/upload must follow wheel/packaging and precede parity')
 def test_current_admission_precedes_node_and_bun_setup(self):
  admission=self.position('Admit actual current R source/archive and temporal WASM producer')
  for action in ['oven-sh/setup-bun@v2','actions/setup-node@v4']:self.assertLess(admission,self.source.index('      - uses: '+action))
 def test_sdk_requires_prior_wheel_success_and_packaging(self):
  start=self.position('Current42 SDK and actual installed final-R correspondence');end=self.position('Preserve Mac producer partials at their original attempt');sdk=self.source[start:end]
  self.assertIn('        if: inputs.current_final\n',sdk);self.assertNotIn('always()',sdk)
  wheel=self.source[self.position('Build and check exact installed ARM64 wheel'):start]
  self.assertIn('maturin build --release --locked',wheel);self.assertIn('verify-wheel-packaging.py',wheel);self.assertIn('packaging-macos-arm64.json',wheel)
if __name__=='__main__':unittest.main()
