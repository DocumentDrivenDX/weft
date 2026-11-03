"""Actual workflow/assignment containment regression; no venv/import/network."""
from pathlib import Path
from unittest.mock import patch
import ast,os,re,tempfile,unittest
HERE=Path(__file__).resolve().parent
SOURCE=Path(os.environ.get('WEFT_PLACEMENT_SOURCE',str(HERE/'verify_public_postflight_0457.py')))
WORKFLOW=HERE.parents[1]/'.github/workflows/public-postflight-0457.yml'
def actual_venv_assignment(out,runner,checkout):
 tree=ast.parse(SOURCE.read_bytes());main=next(n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name=='main');assignment=next(n for n in ast.walk(main) if isinstance(n,ast.Assign) and any(isinstance(t,ast.Name) and t.id=='venv' for t in n.targets))
 nodes=[n for n in tree.body if isinstance(n,ast.FunctionDef) and n.name in ['need','public_consumer_venv']]+[assignment];env=dict(Path=Path,os=os,ROOT=checkout,out=out)
 with patch.dict(os.environ,{'RUNNER_TEMP':str(runner)}):exec(compile(ast.fix_missing_locations(ast.Module(body=nodes,type_ignores=[])),'actual-placement-only','exec'),env)
 return env['venv']
class UploadContainment(unittest.TestCase):
 def test_actual_workflow_upload_never_contains_venv_marker(self):
  text=WORKFLOW.read_text();upload=re.search(r'path: \$\{\{ runner.temp \}\}(/[^\n]+)',text).group(1);proof=re.search(r'--out "\$RUNNER_TEMP([^\"]+)"',text).group(1)
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);runner=d/'runner';runner.mkdir();root=d/'checkout';out=runner/proof.lstrip('/');out.mkdir(parents=True);artifact=runner/upload.strip('/')
   venv=actual_venv_assignment(out,runner,root);venv.mkdir(parents=True);marker=venv/'SYNTHETIC_MUST_NOT_UPLOAD';marker.write_text('fixture only, no venv execution');(out/'retained-proof.json').write_text('{}')
   selected=[p for p in artifact.rglob('*') if p.is_file()];self.assertNotIn(marker,selected);self.assertIn(out/'retained-proof.json',selected)
 def test_overbroad_upload_root_cannot_contain_chosen_venv(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);runner=d/'runner';runner.mkdir()
   with self.assertRaises(ValueError):actual_venv_assignment(runner/'proof',runner,d/'checkout')
 def test_outside_runner_proof_path_refuses(self):
  with tempfile.TemporaryDirectory() as t:
   d=Path(t);runner=d/'runner';runner.mkdir()
   with self.assertRaises(ValueError):actual_venv_assignment(d/'elsewhere/proof',runner,d/'checkout')
if __name__=='__main__':unittest.main(verbosity=2)
