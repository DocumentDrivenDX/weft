"""Focused migration controls; no compiler/runtime qualification."""
import importlib.util,json,unittest
from pathlib import Path
p=Path(__file__).resolve().parents[2]/'scripts/distribution/build-count-star-corpus.py'
s=importlib.util.spec_from_file_location('migration',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
class MigrationTests(unittest.TestCase):
 def test_only_version_tokens_change_and_duplicate_members_survive(self):
  raw=b'{"interfaceVersion":"weft-compile/0.4.0","interfaceVersion":"weft-compile/0.4.0","sql":"SELECT \\\"interfaceVersion\\\":\\\"weft-compile/0.4.0\\\"","target":{"backendVersion":"0.4.0-paths-keys-candidate","bindingJson":"{\\\"backendVersion\\\":\\\"0.4.0-paths-keys-candidate\\\"}"}}'
  result,changes=m.migrate(raw)
  self.assertEqual(result.count(b'"interfaceVersion":"weft-compile/0.4.1"'),2)
  self.assertEqual(len(changes),3)
  for key in ('sql',):self.assertEqual(json.loads(raw)[key],json.loads(result)[key])
  self.assertEqual(json.loads(raw)['target']['bindingJson'],json.loads(result)['target']['bindingJson'])
 def test_mismatch_and_wrong_backend_intent_survive(self):
  raw=b'{"interfaceVersion":"weft-compile/0.4.0","dialect":"weft-sql/0.3.0","target":{"backendVersion":"99.0.0"}}'
  result,changes=m.migrate(raw);self.assertEqual(len(changes),1)
  self.assertEqual(json.loads(result)['dialect'],'weft-sql/0.3.0');self.assertEqual(json.loads(result)['target']['backendVersion'],'99.0.0')
 def test_plain_response_sql_and_nonversion_diagnostics_are_untouched(self):
  raw=m.encoded({'interfaceVersion':'weft-compile/0.4.0','sql':'weft-ir/0.4.0','diagnostics':[{'message':'Backend03 version mismatch'}]})
  result,changes=m.migrate(raw,True);self.assertEqual(len(changes),1)
  self.assertEqual(json.loads(raw)['sql'],json.loads(result)['sql']);self.assertEqual(json.loads(raw)['diagnostics'],json.loads(result)['diagnostics'])
 def test_duplicate_control_keeps_original_preselection_response(self):
  raw=b'{"interfaceVersion":"weft-compile/0.4.0","status":"blocked","diagnostics":[{"code":"WFT-JSON-DUPLICATE"}]}\n'
  result,changes=m.migrate_response('controls:duplicate-envelope-member',raw)
  self.assertEqual(result,raw);self.assertEqual(changes,[])
  other,changes=m.migrate_response('controls:unknown-envelope-member',raw)
  self.assertIn(b'weft-compile/0.4.1',other);self.assertEqual(len(changes),1)
 def test_complete_observations_survive_later_comparison_failure(self):
  import ast,gzip,tempfile
  source=p.with_name('check-count-star-corpus.py').read_text()
  definition=next(n for n in ast.parse(source).body if isinstance(n,ast.FunctionDef) and n.name=='retain_observation')
  with tempfile.TemporaryDirectory() as directory:
   path=Path(directory)/'observations.gz';path.write_bytes(b'')
   env={'gzip':gzip,'json':json,'observation_path':path,'observation_bytes':0,'compressed_bytes':0}
   exec(compile(ast.Module(body=[definition],type_ignores=[]),'retention-control','exec'),env)
   for n in range(2):env['retain_observation']({'id':'case','repeat':n,'responseHex':'7b7d0a'})
   try:raise ValueError('controlled later expectation mismatch')
   except ValueError:pass
   rows=[json.loads(line) for line in gzip.decompress(path.read_bytes()).splitlines()]
   self.assertEqual([row['repeat'] for row in rows],[0,1]);self.assertEqual(rows[0]['responseHex'],'7b7d0a')
if __name__=='__main__':unittest.main()
