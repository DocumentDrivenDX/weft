import importlib.util,json,tempfile,unittest,sys
from pathlib import Path
from unittest.mock import patch
SPEC=importlib.util.spec_from_file_location('paths_corpus',Path(__file__).resolve().parents[2]/'scripts/distribution/check-paths-cli.py')
m=importlib.util.module_from_spec(SPEC);sys.modules[SPEC.name]=m;SPEC.loader.exec_module(m)
class CorpusTests(unittest.TestCase):
 def setUp(self):
  self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup);self.root=Path(self.temp.name).resolve();self.binary=self.root/'binary';self.binary.write_bytes(b'fake-no-executable')
  self.caps=['c'+str(i) for i in range(48)];self.response={'status':'compiled','logicalPlan':{'requiredCapabilities':self.caps}}
  self.legacy=[{'id':'fixture','request':{},'response':self.response}]
  def row(name,status='compiled'):
   response=self.response if status=='compiled' else {'status':'blocked'}
   return {'id':name,'role':'invalid' if status=='blocked' else 'valid','requestHex':b'{}'.hex(),'responseHex':(m.encoded(response)+b'\n').hex()}
  self.fence={'interfaceVersion':'weft-compile/0.4.0','status':'blocked','diagnostics':[{'code':'WFT-VERSION','severity':'error','message':'This entrypoint requires the exact 0.4 compile and dialect pair','phase':'input','recoverability':'correct-input'}]}
  self.bundle={'legacyNamespaceRefusal':(m.encoded(self.fence)+b'\n').hex(),'sourceCommit':'a'*40,'paths':[row('positive')],'controls':[row(n,'blocked' if i<16 else 'compiled') for i,n in enumerate(m.CONTROL_IDS)],'sources':[], 'coverage':{c:{'accepted':['paths:positive'],'refused':[],'scope':'fixture selection only'} for c in self.caps}}
  self.cases=self.root/'cases';self.backend=self.root/'backend';self.backend.write_bytes(m.encoded({'backendId':'ashlar.databricks.paths','backendVersion':'0.4.0-paths-candidate','interfaceVersion':'weft-backend/0.3.0','languageProfiles':[{'dialectProfile':'weft-sql/0.4.0','irVersion':'weft-ir/0.4.0'}],'capabilities':[{'id':c} for c in self.caps]}));self.save()
  self.source=self.root/'source';self.source.mkdir();(self.source/'original').write_bytes(b'original')
  self.inventory=self.root/'inventory';self.inventory.write_bytes(m.encoded({'sourceCommit':'a'*40,'files':[{'path':'original','mode':'100644','gitBlob':m.hashlib.sha1(b'blob 8\0original').hexdigest(),'sha256':m.sha(b'original'),'bytes':8}]}))
  self.checker=self.root/'checker';self.checker.write_bytes(b'trusted fixture schema checker')
  self.config=m.Config(self.source,self.binary,self.cases,self.backend,self.root/'out','a'*40,m.sha(b'fake-no-executable'),4000000,4000000,10,self.inventory,m.sha(self.inventory.read_bytes()),self.checker,m.sha(self.checker.read_bytes()),100,1000000,10000000,1000,64000000,m.sha(self.cases.read_bytes()),m.sha(self.backend.read_bytes()))
 def save(self):
  self.cases.write_bytes(m.encoded(self.bundle))
  if hasattr(self,'config'):
   from dataclasses import replace
   self.config=replace(self.config,cases_sha256=m.sha(self.cases.read_bytes()))
 def execute(self,binary,data,mode,config):
  if mode=='directory-input':return 2,b'',b'WEFT_CLI_INPUT_IO\n'
  if mode=='closed-output':return 2,b'',b'WEFT_CLI_OUTPUT_IO\n'
  if len(data)>m.LIMIT:return 2,b'',b'WEFT_CLI_INPUT_LIMIT\n'
  if data.endswith(b'\xc3') or data==b'\xff':return 2,b'',b'WEFT_CLI_UTF8\n'
  if data!=b'{}':return 0,b'{"status":"blocked"}\n',b''
  # Cases intentionally distinguish expected blocked controls by ordered calls.
  self.calls+=1
  index=(self.calls-1)//2
  expected=self.fence if index==0 else self.response if index==1 or index>=18 else {'status':'blocked'}
  return 0,m.encoded(expected)+b'\n',b''
 def qualify(self,execute=None,validate=None):
  self.calls=0
  with patch.object(m,'extract_legacy',return_value=(self.legacy,[])):
   return m.qualify(self.config,execute=execute or self.execute,validate_schema=validate or (lambda *args:None))
 def test_full_fake_transport_receipts_and_no_authority(self):
  result=self.qualify();self.assertEqual(result['casesInput'],{'sha256':m.sha(self.cases.read_bytes()),'bytes':len(self.cases.read_bytes())});self.assertEqual(result['backendInput'],{'sha256':m.sha(self.backend.read_bytes()),'bytes':len(self.backend.read_bytes())});self.assertEqual(len(result['transport']),8);self.assertEqual(len(result['cases']),21);self.assertTrue((self.config.output/'receipt.json').exists())
 def test_mismatched_response_withholds(self):
  with self.assertRaises(m.Refusal):self.qualify(execute=lambda *a:(0,b'{}',b''))
  self.assertFalse(self.config.output.exists())
 def test_schema_false_refuses_before_execution(self):
  calls=[]
  with self.assertRaises(m.Refusal):self.qualify(execute=lambda *a:calls.append(a),validate=lambda *a:False)
  self.assertEqual(calls,[]);self.assertFalse(self.config.output.exists())
 def test_missing_or_unselected_matrix_refuses(self):
  del self.bundle['coverage']['c0'];self.save()
  with self.assertRaises(m.Refusal):self.qualify()
  self.assertFalse(self.config.output.exists())
 def test_binary_mutation_during_call_withholds(self):
  def execute(*args):self.binary.write_bytes(b'drift');return self.execute(*args)
  with self.assertRaises(m.Refusal):self.qualify(execute=execute)
  self.assertFalse(self.config.output.exists())
 def test_bounded_read_duplicate_and_numeric_controls(self):
  with self.assertRaises(m.Refusal):m.read(self.binary,1)
  with self.assertRaises(m.Refusal):m.document(b'{"a":1,"a":2}')
  with self.assertRaises(m.Refusal):m.document(b'['+b'9'*10000+b']')
  with self.assertRaises(m.Refusal):m.document(b'[1.0]')
 def test_unknown_migration_does_not_normalize(self):
  response={'status':'blocked','diagnostics':[{'phase':'other'}]}
  self.assertEqual(m.migrate_legacy('another',response),(response,[]))
  with self.assertRaises(m.Refusal):m.migrate_legacy('unsigned-columns:unsigned-64',response)
 def fake_program(self,body):
  p=self.root/'fake-program';p.write_text('#!'+sys.executable+'\n'+body);p.chmod(0o755);return p
 def test_actual_transport_capture_limit_kills_and_reaps_fake_process(self):
  from dataclasses import replace
  p=self.fake_program("import sys;sys.stdout.write('x'*1000);sys.stdout.flush()")
  with self.assertRaises(m.Refusal):m.transport(p,b'','normal',replace(self.config,maximum_response_bytes=4))
 def test_actual_transport_descendant_pipe_hold_is_deadline_bounded(self):
  import time
  from dataclasses import replace
  p=self.fake_program("import os,time\nif os.fork()==0:time.sleep(20)\nelse:os._exit(0)\n")
  started=time.monotonic()
  with self.assertRaises(m.Refusal):m.transport(p,b'','normal',replace(self.config,timeout_seconds=1))
  self.assertLess(time.monotonic()-started,3)
 def test_actual_transport_cancellation_preserves_primary(self):
  primary=KeyboardInterrupt();p=self.fake_program('import time;time.sleep(20)')
  with patch.object(m.selectors.BaseSelector,'select',side_effect=primary):
   # Patch concrete platform selector rather than its abstract base.
   with patch.object(type(m.selectors.DefaultSelector()),'select',side_effect=primary):
    with self.assertRaises(KeyboardInterrupt) as caught:m.transport(p,b'','normal',self.config)
  self.assertIs(caught.exception,primary)
 def test_actual_transport_closed_output_finishes_without_thread_join(self):
  p=self.fake_program("import os\ntry:os.write(1,b'x')\nexcept OSError:raise SystemExit(2)\n")
  code,out,err=m.transport(p,b'','closed-output',self.config)
  self.assertEqual(code,2);self.assertEqual(out,b'')
 def test_source_inventory_and_schema_pin_refuse_before_calls(self):
  self.checker.write_bytes(b'drift');calls=[]
  with self.assertRaises(m.Refusal):self.qualify(execute=lambda *a:calls.append(a))
  self.assertEqual(calls,[])
 def test_source_extra_and_changed_original_refuse(self):
  (self.source/'extra').write_bytes(b'x')
  with self.assertRaises(m.Refusal):m.verify_source(self.config)
  (self.source/'extra').unlink();(self.source/'original').write_bytes(b'drift')
  with self.assertRaises(m.Refusal):m.verify_source(self.config)
 def test_count_and_receipt_budget_refuse_before_transport(self):
  from dataclasses import replace
  for changes in ({'maximum_cases':1},{'maximum_receipt_bytes':1}):
   with self.subTest(changes=changes):
    calls=[]
    with patch.object(m,'extract_legacy',return_value=(self.legacy,[])):
     with self.assertRaises(m.Refusal):m.qualify(replace(self.config,**changes),execute=lambda *a:calls.append(a),validate_schema=lambda *a:None)
    self.assertEqual(calls,[]);self.assertFalse(self.config.output.exists())
 def test_namespace_retains_legacy_bytes_without_migration(self):
  result=self.qualify();row=result['cases'][0]
  self.assertEqual(bytes.fromhex(row['requestHex']),m.encoded(self.legacy[0]['request']))
  self.assertEqual(bytes.fromhex(row['originalExpectedHex']),m.encoded(self.response)+b'\n')
  self.assertEqual(bytes.fromhex(row['responseHex']),m.encoded(self.fence)+b'\n');self.assertEqual(row['migrations'],[])
 def test_arbitrary_legacy_blocked_expectation_refuses(self):
  self.bundle['legacyNamespaceRefusal']=b'{"status":"blocked"}\n'.hex();self.save()
  with self.assertRaises(m.Refusal):self.qualify()
 def test_cleanup_close_failure_does_not_mask_cancellation(self):
  primary=KeyboardInterrupt();p=self.fake_program('import time;time.sleep(20)')
  selector_type=type(m.selectors.DefaultSelector())
  with patch.object(selector_type,'select',side_effect=primary),patch.object(selector_type,'close',side_effect=OSError('cleanup')):
   with self.assertRaises(KeyboardInterrupt) as caught:m.transport(p,b'','normal',self.config)
  self.assertIs(caught.exception,primary);self.assertTrue(primary.cleanup_failed)
 def test_owned_fd_close_failure_does_not_mask_cancellation(self):
  primary=KeyboardInterrupt();p=self.fake_program('import time;time.sleep(20)');original=m.os.close;original_open=m.os.open;owned=[];calls=[]
  def opened(*args):
   fd=original_open(*args);owned.append(fd);return fd
  def close(fd):
   if fd not in owned:return original(fd)
   calls.append(fd)
   if len(calls)==1:raise primary
   original(fd);raise OSError('cleanup')
  with patch.object(m.os,'open',side_effect=opened),patch.object(m.os,'close',side_effect=close):
   with self.assertRaises(KeyboardInterrupt) as caught:m.transport(p,b'','directory-input',self.config)
  self.assertIs(caught.exception,primary);self.assertTrue(primary.cleanup_failed)
 def test_successful_leader_closed_stdio_descendant_group_is_cleaned(self):
  p=self.fake_program("import os,time\npid=os.fork()\nif pid==0:\n os.close(0);os.close(1);os.close(2);time.sleep(20)\nelse:\n print(pid,flush=True);os._exit(0)\n")
  original=m.os.killpg
  with patch.object(m.os,'killpg',wraps=original) as kill:
   code,out,err=m.transport(p,b'','normal',self.config)
  self.assertEqual(code,0);self.assertTrue(out.strip().isdigit());kill.assert_called_once()
 def test_large_control_receipt_is_refused_before_serialization(self):
  from dataclasses import replace
  self.config=replace(self.config,maximum_receipt_bytes=1000000)
  def execute(*args):
   if args[1]==b'{':return 0,b'{"status":"blocked","padding":"'+b'x'*200000+b'"}\n',b''
   return self.execute(*args)
  with self.assertRaises(m.Refusal):self.qualify(execute=execute)
  self.assertFalse(self.config.output.exists())
 def test_actual_checked_in_legacy_extraction_only(self):
  source=Path(__file__).resolve().parents[2]
  cases,files=m.extract_legacy(source,32000000)
  self.assertEqual(len(cases),463);self.assertEqual(len({x['id'] for x in cases}),463);self.assertTrue(files)
if __name__=='__main__':unittest.main()
