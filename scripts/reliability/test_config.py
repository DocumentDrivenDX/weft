"""@covers US-008-AC4 Exercise actual host configuration and optimized failures."""
import dataclasses,json,os,pathlib,subprocess,sys,tempfile,unittest
from reliability.config import load_config,ConfigurationError
from reliability.mutations import apply_mutation,detected,execute,MutationError
from reliability.process import run,Result
class ConfigurationTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.root=pathlib.Path(self.tmp.name).resolve();self.file=self.root/'config.json';
  self.values={'cargo':sys.executable,'cargo_home':str(self.root),'rustup_home':str(self.root),'temp_root':str(self.root/'tmp'),'output_root':str(self.root/'out')};self.file.write_text(json.dumps(self.values))
 def tearDown(self):self.tmp.cleanup()
 def test_precedence(self):
  self.file.write_text(json.dumps(self.values|{'record_limit':10}));c=load_config(['--config',str(self.file),'--record-limit','12'],{'WEFT_RECORD_LIMIT':'11'});self.assertEqual(c.record_limit,12);self.assertEqual(c.queue_limit,16)
 def test_unknown_duplicate_oversize_malformed_and_missing_refuse_before_side_effects(self):
  for raw in [json.dumps(self.values|{'secret':'sentinel'}),'{}','{"cargo":"x","cargo":"y"}',json.dumps(self.values|{'record_limit':True}),json.dumps(self.values|{'timeout_seconds':float('nan')}),' '*65537]:
   self.file.write_text(raw)
   with self.assertRaises(ConfigurationError):load_config(['--config',str(self.file)],{})
   self.assertFalse((self.root/'out').exists());self.assertFalse((self.root/'tmp').exists())
 def test_nonregular_configuration_refuses_without_waiting(self):
  if os.name!='posix':self.skipTest('Unix host boundary')
  fifo=self.root/'fifo';os.mkfifo(fifo)
  with self.assertRaises(ConfigurationError):load_config(['--config',str(fifo)],{})
 def test_bad_env_and_endpoint_refuse(self):
  for env in [{'WEFT_UNKNOWN':'sentinel'},{'WEFT_RECORD_LIMIT':'0'},{'WEFT_ENDPOINT':'https://secret:credential@example.test/v1/logs'},{'WEFT_ENDPOINT':'https://example.test/v1/logs?secret=credential'},{'WEFT_CARGO':''}]:
   with self.assertRaises(ConfigurationError):load_config(['--config',str(self.file)],env)
 def test_actual_optimized_cli_refuses_invalid_configuration_without_leak(self):
  entry=pathlib.Path(__file__).resolve().parents[2]/'tests/qualify-and-evolve/source-mutations.py'
  self.file.write_text(json.dumps(self.values|{'unknown':'private-configuration-sentinel'}))
  environment={k:v for k,v in os.environ.items() if not k.startswith('WEFT_')}
  result=subprocess.run([sys.executable,'-O',str(entry),'--config',str(self.file)],env=environment,capture_output=True,timeout=5)
  self.assertEqual(result.returncode,1);self.assertEqual(result.stdout,b'')
  self.assertEqual(result.stderr,b'weft-runner: source mutation qualification failed\n')
  self.assertFalse((self.root/'out').exists());self.assertFalse((self.root/'tmp').exists())
 def test_actual_cli_nul_path_is_fixed_safe_failure(self):
  entry=pathlib.Path(__file__).resolve().parents[2]/'tests/qualify-and-evolve/source-mutations.py'
  self.file.write_text(json.dumps(self.values|{'temp_root':str(self.root)+'/private-sentinel\x00'}))
  result=subprocess.run([sys.executable,'-O',str(entry),'--config',str(self.file)],env={k:v for k,v in os.environ.items() if not k.startswith('WEFT_')},capture_output=True,timeout=5)
  self.assertEqual(result.returncode,1);self.assertEqual(result.stdout,b'');self.assertEqual(result.stderr,b'weft-runner: source mutation qualification failed\n')
 def test_tool_identity_refuses_unpinned_tool(self):
  c=load_config(['--config',str(self.file)],{})
  source=self.root/"source";source.mkdir()
  with self.assertRaises(ConfigurationError):c.tool_identity(source)
 def test_actual_execute_rejects_source_overlap_and_symlink_alias_before_copy(self):
  self.file.write_text(json.dumps(self.values))
  config=load_config(['--config',str(self.file)],{})
  with self.assertRaises(ConfigurationError):execute(config,self.root)
  self.assertFalse((self.root/'tmp').exists());self.assertFalse((self.root/'out').exists())
  if os.name=='posix':
   alias=self.root/'alias';alias.symlink_to(self.root,target_is_directory=True)
   config=dataclasses.replace(config,temp_root=alias/'tmp')
   with self.assertRaises(ConfigurationError):execute(config,self.root)
   self.assertFalse((self.root/'tmp').exists())
 def test_identity_probe_rejects_oversize_suffix(self):
  source=self.root/'source';source.mkdir();(source/'rust-toolchain.toml').write_text('[toolchain]\nchannel="1.90.0"\n')
  bin=self.root/'bin';bin.mkdir();cargo=bin/'cargo';rustc=bin/'rustc'
  config=dataclasses.replace(load_config(['--config',str(self.file)],{}),cargo=cargo)
  for tool,name in [(cargo,'cargo'),(rustc,'rustc')]:
   tool.write_text('#!'+sys.executable+'\nprint("'+name+' 1.90.0 (123456789 2025-09-14)")\n');tool.chmod(0o700)
  self.assertEqual(config.tool_identity(source)['rustcVersion'],'rustc 1.90.0 (123456789 2025-09-14)')
  cargo.write_text('#!'+sys.executable+'\nprint("cargo 1.90.0 (123456789 2025-09-14)"+"private-sentinel"*100000)\n')
  with self.assertRaises(ConfigurationError):config.tool_identity(source)
  self.assertEqual(config.command_environment()['RUSTC'],str(rustc))
  self.assertEqual(config.command_environment()['RUSTC_WRAPPER'],'')
  self.assertEqual(config.command_environment()['RUSTC_WORKSPACE_WRAPPER'],'')
 def test_actual_execute_refuses_already_failing_baseline(self):
  from unittest.mock import patch
  source=self.root/'source';source.mkdir();(source/'guard.rs').write_text('guard')
  script=self.root/'failing';script.write_text('#!'+sys.executable+'\nprint("running 1 test")\nprint("test selected ... FAILED")\nprint("panicked at owner type selection must survive lowering")\nprint("test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out")\nraise SystemExit(101)\n');script.chmod(0o700)
  config=dataclasses.replace(load_config(['--config',str(self.file)],{}),cargo=script)
  with patch('reliability.config.Config.tool_identity',return_value={}),patch('reliability.mutations.cases',[('scan-type-filter','guard.rs','guard','broken','postgresql-candidate','selected')]):
   with self.assertRaises(MutationError):execute(config,source)
  self.assertEqual((source/'guard.rs').read_text(),'guard')
 def test_unapplied_and_duplicate_mutations_refuse(self):
  for original in ['absent','guard guard']:
   with self.assertRaises(MutationError):apply_mutation(original,'guard','broken')
  self.assertEqual(apply_mutation('guard','guard','broken'),'broken')
 def test_real_process_missing_test_wrong_signature_and_compilation_failure_refuse(self):
  name='scan-type-filter';selected='selected';needles=('owner type selection must survive lowering','test result: FAILED.','panicked at')
  good='running 1 test\ntest selected ... FAILED\npanicked at owner type selection must survive lowering\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n'
  for text,exit_code,expected in [(good,101,True),(good.replace('running 1 test','running 0 tests'),101,False),(good.replace('owner type selection must survive lowering','different signature'),101,False),(good,1,False),('error: compilation failed',101,False)]:
   result=run([sys.executable,'-c','import sys;sys.stdout.write('+repr(text)+');sys.exit('+str(exit_code)+')'],self.root,os.environ.copy(),2,needles,(selected,));self.assertEqual(detected(result,name,selected),expected)
 def test_large_raw_output_drains_without_capture(self):
  result=run([sys.executable,'-c','import sys;sys.stdout.write("private-sentinel"*1000000)'],self.root,os.environ.copy(),5)
  self.assertEqual(result.exit_code,0);self.assertGreater(result.bytes_read,10000000);self.assertNotIn('private-sentinel',repr(result))
 def test_timeout_terminates_boundedly(self):
  result=run([sys.executable,'-c','import time;time.sleep(20)'],self.root,os.environ.copy(),.1)
  self.assertTrue(result.timed_out);self.assertLess(result.duration_ms,3100)
if __name__=='__main__':unittest.main()
