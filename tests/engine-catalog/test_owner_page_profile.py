"""Static source/fixture custody controls; no native compilation or execution."""
import ast
import copy
import hashlib
import json
from pathlib import Path
import unittest

ROOT = Path(__file__).resolve().parents[2]
DIRECTORY = ROOT / 'tests/engine-catalog'

class OwnerPageProfile(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        tree = ast.parse((DIRECTORY / 'wheel-smoke.py').read_text())
        names = {'no_executable', 'diagnostic_codes', 'owner_page_request'}
        helpers = ast.Module(body=[n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name in names], type_ignores=[])
        cls.provenance = json.loads((DIRECTORY / 'owner-page-profile-provenance.json').read_text())
        cls.namespace = dict(root=ROOT, json=json, sha=lambda b: hashlib.sha256(b).hexdigest(), provenance=cls.provenance)
        exec(compile(helpers, 'wheel-smoke-helpers', 'exec'), cls.namespace)

    def test_owner_source_and_registry_match_provenance(self):
        for source, digest in [('ownerSourcePath', 'ownerSourceSHA256'), ('registrySource', 'registrySHA256')]:
            self.assertEqual(hashlib.sha256((ROOT / self.provenance[source]).read_bytes()).hexdigest(), self.provenance[digest])
        self.assertFalse(self.provenance['nativeAuthority'])

    def test_complete_positive_requests_equal_owner_construction(self):
        for item in self.provenance['cases']:
            with self.subTest(file=item['file']):
                retained = json.loads((DIRECTORY / item['retainedSource']).read_text())
                actual = self.namespace['owner_page_request'](item['file'], retained)
                expected = copy.deepcopy(retained)
                expected['interfaceVersion'] = 'weft-compile/0.2.7'
                expected['timestampComparisonProfile'] = 'umf-rfc3339-exact-instant/1.0.0'
                expected['target']['backendVersion'] = item['construction']['backendVersion']
                if 'spark4' in item['file']:
                    expected['target']['backendId'] = 'ashlar.databricks'
                    expected['target']['targetProfile'] = 'spark4.0.1-delta4.0.0-candidate'
                self.assertEqual(actual, expected)

    def test_parent_source_binding_and_parameter_drift_refuse(self):
        for item in self.provenance['cases']:
            for mode in ['source', 'binding', 'parameters']:
                with self.subTest(file=item['file'], mode=mode):
                    retained = json.loads((DIRECTORY / item['retainedSource']).read_text())
                    if mode == 'source':
                        retained['modules'][0]['documentJson'] += ' '
                    elif mode == 'binding':
                        retained['target']['bindingJson'] += ' '
                    else:
                        retained['parameters'] = {}
                    with self.assertRaises(AssertionError):
                        self.namespace['owner_page_request'](item['file'], retained)

    def test_nested_refusal_requires_exact_diagnostic_and_no_executable(self):
        negative = {'status': 'blocked', 'original': {'status': 'blocked', 'original': {
            'status': 'blocked', 'diagnostics': [{'code': 'WFT-BACKEND-VERSION'}]}}}
        self.assertEqual(self.namespace['diagnostic_codes'](negative), ['WFT-BACKEND-VERSION'])
        self.namespace['no_executable'](negative)
        for key in ['artifact', 'sql', 'parameters']:
            with self.subTest(key=key):
                forged = copy.deepcopy(negative)
                forged['original']['original'][key] = {} if key != 'sql' else 'SELECT 1'
                with self.assertRaises(AssertionError):
                    self.namespace['no_executable'](forged)

if __name__ == '__main__':
    unittest.main()
