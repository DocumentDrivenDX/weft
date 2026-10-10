"""@covers US-008-AC6 Bounded formal gate and fail-closed controls."""
import pathlib,types,unittest,sys
from unittest.mock import patch
import z3
from reliability import formal,gate
ROOT=pathlib.Path(__file__).resolve().parents[2]
class FormalTests(unittest.TestCase):
 def test_actual_bounded_model_and_witnesses(self):
  report=formal.check(ROOT);self.assertEqual(report['status'],'passed');self.assertEqual(report['oracleTuples'],27)
  self.assertTrue(all(v['result']=='unsat' for v in report['properties'].values()));self.assertTrue(all(v['result']=='sat' for v in report['witnesses'].values()))
 def test_solver_unknown_and_wrong_expected_fail(self):
  with patch.object(z3,'Solver') as factory:
   factory.return_value.check.return_value=z3.unknown
   with self.assertRaises(formal.FormalError):formal.solve(z3.BoolVal(True),z3.sat,())
  with self.assertRaises(formal.FormalError):formal.solve(z3.BoolVal(True),z3.unsat,())
 def test_wrong_solver_version_refuses(self):
  with patch.object(z3,'get_version_string',return_value='4.15.3'):
   with self.assertRaises(formal.FormalError):formal.check(ROOT)
 def test_wrong_distribution_version_refuses(self):
  with patch.object(formal.importlib.metadata,'version',return_value='4.15.4.1'):
   with self.assertRaises(formal.FormalError):formal.check(ROOT)
 def test_missing_or_malformed_correspondence_refuses(self):
  config=types.SimpleNamespace(cargo=pathlib.Path('/unused'),temp_root=pathlib.Path('/private/tmp/formal-control'),timeout_seconds=1,command_environment=lambda:{})
  for result in [types.SimpleNamespace(timed_out=True,exit_code=0),types.SimpleNamespace(timed_out=False,exit_code=1),types.SimpleNamespace(timed_out=False,exit_code=0,selected_tests=0),types.SimpleNamespace(timed_out=False,exit_code=0,selected_tests=1,passed=0),types.SimpleNamespace(timed_out=False,exit_code=0,selected_tests=1,passed=1,failed=1),types.SimpleNamespace(timed_out=False,exit_code=0,selected_tests=1,passed=1,failed=0,ignored=1)]:
   with patch.object(gate,'run',return_value=result):
    with self.assertRaises(RuntimeError):gate.correspondence(config,ROOT)
 def test_oracle_substitution_refuses(self):
  import json,tempfile
  with tempfile.TemporaryDirectory() as work:
   root=pathlib.Path(work);path=root/'crates/weft-core/tests/security-composition-oracle.json';path.parent.mkdir(parents=True)
   oracle=json.loads((ROOT/'crates/weft-core/tests/security-composition-oracle.json').read_bytes());oracle['decisions'][0]['decision']='conflict';path.write_text(json.dumps(oracle));(path.parent/'security-disposition-oracle.json').write_bytes((ROOT/'crates/weft-core/tests/security-disposition-oracle.json').read_bytes())
   with self.assertRaises(formal.FormalError):formal.check(root)
 def test_actual_gate_rejects_changed_and_deleted_inputs(self):
  import contextlib,io,json,shutil,tempfile
  from reliability.config import load_config,Config
  for mode in ('changed','deleted','compiler_changed'):
   with tempfile.TemporaryDirectory() as work:
    work=pathlib.Path(work);root=work/'repo';root.mkdir()
    for name in gate.FILES:
     path=root/name;path.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,path)
    for name in ('tests/qualify-and-evolve/source-mutations.py','docs/helix/02-design/module-boundaries.json'):
     path=root/name;path.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(ROOT/name,path)
    config=load_config(['--cargo',sys.executable,'--rustup-home',str(work),'--cargo-home',str(work),'--temp-root',str(work/'tmp'),'--output-root',str(work/'out')],{})
    def mutated(*args):
     path=root/('crates/weft-core/src/security_composition.rs' if mode=='compiler_changed' else 'crates/weft-core/tests/security-disposition-oracle.json')
     if mode=='deleted':path.unlink()
     else:path.write_bytes(path.read_bytes()+b' ')
     return {}
    with patch.object(gate,'ROOT',root),patch.object(gate,'load_config',return_value=config),patch.object(Config,'tool_identity',return_value={}),patch.object(gate,'correspondence',side_effect=mutated),contextlib.redirect_stdout(io.StringIO()) as stdout,contextlib.redirect_stderr(io.StringIO()) as stderr:
     self.assertEqual(gate.main(),1)
    self.assertEqual(stdout.getvalue(),'');self.assertIn('weft-runner: formal qualification failed',stderr.getvalue())
    directory=next((work/'out').glob('weft-run-*'));self.assertFalse((directory/'qualification.json').exists());self.assertEqual(json.loads((directory/'manifest.json').read_bytes())['outcome'],'failed')
if __name__=='__main__':unittest.main()
