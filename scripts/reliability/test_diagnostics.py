"""@covers US-008-AC5 Real SDK/receiver, privacy, loss and snapshot controls."""
import subprocess,base64,contextlib,dataclasses,http.server,io,json,os,pathlib,sys,tempfile,threading,time,unittest,uuid
from unittest.mock import patch
from reliability.diagnostics import Run,retrieve,DiagnosticError,packed
from reliability.config import load_config,ConfigurationError
from google.protobuf.json_format import ParseDict
from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import ExportLogsServiceRequest
from opentelemetry.proto.collector.trace.v1.trace_service_pb2 import ExportTraceServiceRequest
class Receiver(http.server.BaseHTTPRequestHandler):
 def log_message(self,*args):pass
 def do_POST(self):
  data=self.rfile.read(int(self.headers['Content-Length']));self.server.wire.append((self.path,data));value=json.loads(data)
  def walk(v):
   if isinstance(v,dict):
    for k,x in list(v.items()):
     if k in ('traceId','spanId','parentSpanId'):v[k]=base64.b64encode(bytes.fromhex(x)).decode()
     else:walk(x)
   elif isinstance(v,list):
    for x in v:walk(x)
  walk(value)
  message=ExportLogsServiceRequest() if self.path=='/v1/logs' else ExportTraceServiceRequest()
  ParseDict(value,message);self.server.messages.append((self.path,message))
  if self.server.delay:time.sleep(self.server.delay)
  self.send_response(200);self.send_header('Content-Type',getattr(self.server,'response_type','application/json'));self.end_headers()
  try:
   if getattr(self.server,'drip',False):
    for _ in range(48):self.wfile.write(b' ');self.wfile.flush();time.sleep(.035)
   self.wfile.write(getattr(self.server,'response_body',b'{}'))
  except OSError:pass
class DiagnosticsTests(unittest.TestCase):
 def setUp(self):
  self.tmp=tempfile.TemporaryDirectory();self.root=pathlib.Path(self.tmp.name);self.config=load_config(['--cargo',sys.executable,'--rustup-home',str(self.root),'--cargo-home',str(self.root),'--temp-root',str(self.root/'tmp'),'--output-root',str(self.root/'out'),'--queue-limit','64'],{})
 def tearDown(self):self.tmp.cleanup()
 @contextlib.contextmanager
 def receiver(self,delay=0):
  server=http.server.ThreadingHTTPServer(('127.0.0.1',0),Receiver);server.wire=[];server.messages=[];server.delay=delay;server.drip=False
  thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start()
  try:yield server
  finally:server.shutdown();server.server_close();thread.join(1)
 def test_real_logger_sdk_receiver_context_privacy_and_interleaved_pilot(self):
  start=time.monotonic()
  with self.receiver() as receiver:
   config=dataclasses.replace(self.config,endpoint=f'http://127.0.0.1:{receiver.server_port}/v1/logs')
   run=Run(config,'pilot');attempts=[uuid.uuid4().hex for _ in range(3)]
   for attempt in attempts:run.event('operation.started','pilot',attempt=attempt)
   with run.tracer.start_as_current_span('pilot') as span:
    span.set_attribute('private','private-sentinel')
    run.logger.info('private-sentinel %s','private-sentinel',extra={'weft_event':'operation.failed','weft_operation':'pilot','weft_outcome':'failed','weft_attempt':attempts[1],'private':'private-sentinel'})
   run.event('operation.completed','pilot','passed',attempt=attempts[0]);run.event('operation.completed','pilot','passed',attempt=attempts[2])
   self.assertEqual(run.close(),'failed')
   page=retrieve(run.directory);logs=[r for path,m in receiver.messages if path=='/v1/logs' for resource in m.resource_logs for scope in resource.scope_logs for r in scope.log_records]
   spans=[s for path,m in receiver.messages if path=='/v1/traces' for resource in m.resource_spans for scope in resource.scope_spans for s in scope.spans]
   manifest=json.loads((run.directory/'manifest.json').read_bytes());self.assertEqual(manifest['counts']['operations'],3);self.assertEqual(manifest['counts']['operations_failed'],1)
   self.assertEqual(len(logs),8);self.assertEqual(len(spans),1);self.assertEqual(len(page['records']),8)
   self.assertEqual(len({r['attributes']['weft.sequence'] for r in [x['record'] for x in page['records']]}),8)
   failed=[x['record'] for x in page['records'] if x['record']['event_name']=='operation.failed'];self.assertEqual(len(failed),1);self.assertEqual(failed[0]['attributes']['weft.attempt.id'],attempts[1])
   self.assertEqual(failed[0]['trace_id'],spans[0].trace_id.hex());self.assertEqual(failed[0]['span_id'],spans[0].span_id.hex());self.assertNotIn('trace_id',page['records'][0]['record'])
   self.assertEqual(logs[4].severity_number,17)
   for path,data in receiver.wire:
    wire=json.loads(data)
    if path=='/v1/logs':
     record=wire['resourceLogs'][0]['scopeLogs'][0]['logRecords'][0]
     self.assertEqual(type(record['severityNumber']),int);self.assertEqual(type(record['timeUnixNano']),str)
     self.assertIn('eventName',record)
     for attribute in record['attributes']:
      if 'intValue' in attribute['value']:self.assertEqual(type(attribute['value']['intValue']),str)
   all_bytes=(run.directory/'events.jsonl').read_bytes()+(run.directory/'manifest.json').read_bytes()+b''.join(data for _,data in receiver.wire)
   self.assertNotIn(b'private-sentinel',all_bytes)
   self.assertIsNone(page['cursor']);self.assertLess(len(packed(page)),65536)
   self.assertEqual((run.directory.stat().st_mode&0o777),0o700)
   for p in run.directory.iterdir():self.assertEqual(p.stat().st_mode&0o777,0o600)
   self.metrics={'failedAttempts':1,'correctlyDiagnosed':1,'pages':1,'returnedBytes':len(packed(page)),'elapsedMs':round((time.monotonic()-start)*1000),'sdkLogRecords':len(logs),'sdkSpans':len(spans),'duplicateSequences':0}
 def test_logger_never_formats_raw_message_and_tampered_records_refuse(self):
  class PrivateMessage:
   def __str__(self):raise AssertionError('raw formatting forbidden')
  console=io.StringIO()
  with contextlib.redirect_stderr(console):
   run=Run(self.config,'pilot');run.logger.info(PrivateMessage(),extra={'weft_event':'operation.started','weft_operation':'pilot','private':'private-sentinel'});run.close()
  self.assertNotIn('private-sentinel',console.getvalue())
  path=run.directory/'events.jsonl';records=[json.loads(x) for x in path.read_bytes().splitlines()];records[0]['attributes']['private']='private-sentinel';path.write_bytes(b''.join(packed(v)+b'\n' for v in records))
  with self.assertRaises(DiagnosticError):retrieve(run.directory)
 def test_temporary_capture_failure_preserves_gap_and_loss(self):
  run=Run(self.config,'pilot');stream=run.stream
  class FailOnce:
   failed=False
   def write(self,data):
    if not self.failed:self.failed=True;raise OSError()
    return stream.write(data)
   def flush(self):return stream.flush()
   def close(self):return stream.close()
  run.stream=FailOnce();run.event('operation.started');run.event('operation.started');self.assertEqual(run.close(),'incomplete');page=retrieve(run.directory)
  sequences=[x['record']['attributes']['weft.sequence'] for x in page['records']];self.assertEqual(sequences,[1,3,4,5]);self.assertIn('capture_failed',page['loss']);self.assertEqual([x['line'] for x in page['records']],[1,2,3,4])
 def test_ambient_otel_override_refuses_before_outputs(self):
  with patch.dict(os.environ,{'OTEL_SDK_DISABLED':'true'}):
   with self.assertRaises(ConfigurationError):Run(self.config,'pilot')
  self.assertFalse(self.config.output_root.exists())
 def test_actual_readonly_cli_pages_and_malformed_safe_failure(self):
  run=Run(self.config,'pilot');run.close();entry=pathlib.Path(__file__).with_name('retrieve.py')
  before={p.name:(p.stat().st_size,p.stat().st_mtime_ns) for p in run.directory.iterdir()}
  result=subprocess.run([sys.executable,str(entry),'--run-directory',str(run.directory)],capture_output=True,timeout=5)
  self.assertEqual(result.returncode,0);self.assertEqual(result.stderr,b'');self.assertEqual(json.loads(result.stdout)['outcome'],'passed')
  self.assertEqual(before,{p.name:(p.stat().st_size,p.stat().st_mtime_ns) for p in run.directory.iterdir()})
  result=subprocess.run([sys.executable,'-O',str(entry),'--run-directory',str(run.directory),'--cursor','private-sentinel'],capture_output=True,timeout=5)
  self.assertEqual(result.returncode,1);self.assertEqual(result.stdout,b'');self.assertEqual(result.stderr,b'weft-runner: diagnostic retrieval refused\n')
 def test_crash_debris_pending_allowance_and_retention_do_not_delete_unknown_files(self):
  self.config.output_root.mkdir();debris=self.config.output_root/('weft-run-'+uuid.uuid4().hex);debris.mkdir();(debris/'private').write_text('owner content')
  with self.assertRaises(DiagnosticError):Run(self.config,'pilot')
  self.assertTrue((debris/'private').exists());(debris/'private').unlink();debris.rmdir()
  config=dataclasses.replace(self.config,retained_runs=1);pending=Run(config,'pilot')
  with self.assertRaises(DiagnosticError):Run(config,'pilot')
  pending.close();old=Run(config,'pilot');old.close();(old.directory/'unknown').write_text('owner content')
  with self.assertRaises(DiagnosticError):Run(config,'pilot')
  self.assertTrue((old.directory/'unknown').exists())
 def test_inherited_trace_state_is_removed_at_actual_receiver(self):
  from opentelemetry.trace import SpanContext,TraceState,TraceFlags,NonRecordingSpan,use_span
  with self.receiver() as receiver:
   config=dataclasses.replace(self.config,endpoint=f'http://127.0.0.1:{receiver.server_port}/v1/logs')
   run=Run(config,'pilot')
   parent=SpanContext(1,2,True,TraceFlags(1),TraceState([('vendor','private-tracestate-sentinel')]))
   with use_span(NonRecordingSpan(parent)):
    with run.operation_context('pilot'):pass
   self.assertEqual(run.close(),'passed');self.assertTrue(any(path=='/v1/traces' for path,_ in receiver.wire));self.assertNotIn(b'private-tracestate-sentinel',b''.join(data for _,data in receiver.wire))
 def test_closed_truncation_empty_capture_and_same_inode_rewrite_refuse(self):
  run=Run(self.config,'pilot')
  for _ in range(60):run.event('operation.started')
  run.close();path=run.directory/'events.jsonl';original=path.read_bytes();cursor=retrieve(run.directory)['cursor'];self.assertTrue(cursor)
  for replacement in (original[:-1],b''):
   path.write_bytes(replacement)
   with self.assertRaises(DiagnosticError):retrieve(run.directory)
  path.write_bytes(original);cursor=retrieve(run.directory)['cursor'];rewritten=original.replace(b'"weft.operation":"pilot"',b'"weft.operation":"cargo"',1);self.assertEqual(len(rewritten),len(original));path.write_bytes(rewritten)
  with self.assertRaises(DiagnosticError):retrieve(run.directory,cursor)
 def test_total_export_deadline_stops_slow_drip_response(self):
  with self.receiver() as receiver:
   receiver.drip=True;config=dataclasses.replace(self.config,endpoint=f'http://127.0.0.1:{receiver.server_port}/v1/logs',export_timeout_seconds=.2)
   start=time.monotonic();run=Run(config,'pilot');self.assertEqual(run.close(),'incomplete');elapsed=time.monotonic()-start
   self.assertLess(elapsed,1.2);self.assertGreaterEqual(len(receiver.wire),1);manifest=json.loads((run.directory/'manifest.json').read_bytes());self.assertEqual(manifest['counts']['exported'],0);self.assertEqual(manifest['flush_state'],'failed')
 def test_input_revision_ingress_and_all_zero_context_refuse(self):
  with self.assertRaises(DiagnosticError):Run(self.config,'pilot',{'kind':'sha256','value':'private-sentinel','scope':'compiler-and-host-source'})
  self.assertFalse(self.config.output_root.exists())
  run=Run(self.config,'pilot',{'kind':'sha256','value':'a'*64,'scope':'compiler-and-host-source'});run.close();self.assertEqual(retrieve(run.directory)['input_revision']['value'],'a'*64)
  path=run.directory/'events.jsonl';records=[json.loads(line) for line in path.read_bytes().splitlines()];records[0].update(trace_id='0'*32,span_id='0'*16,trace_flags=1);path.write_bytes(b''.join(packed(v)+b'\n' for v in records))
  with self.assertRaises(DiagnosticError):retrieve(run.directory)
 def test_actual_process_admission_is_single_writer(self):
  values={name:str(getattr(self.config,name)) for name in ('cargo','rustup_home','cargo_home','temp_root','output_root')};values['retained_runs']=1
  file=self.root/'config.json';file.write_text(json.dumps(values));environment=os.environ.copy();environment['PYTHONPATH']=str(pathlib.Path(__file__).resolve().parents[1])
  code="from reliability.config import load_config;from reliability.diagnostics import Run;import sys,time\ntry:\n r=Run(load_config(['--config',sys.argv[1]],{}),'pilot');print('admitted',flush=True);time.sleep(.3)\nexcept Exception:print('refused',flush=True)"
  processes=[subprocess.Popen([sys.executable,'-c',code,str(file)],stdout=subprocess.PIPE,stderr=subprocess.PIPE,env=environment) for _ in range(8)]
  outputs=[p.communicate(timeout=10) for p in processes]
  self.assertEqual(sum(out==b'admitted\n' for out,err in outputs),1);self.assertEqual(len(list(self.config.output_root.glob('weft-run-*'))),1)
  self.assertTrue(all(p.returncode==0 for p in processes));self.assertTrue(all(b'Traceback' not in err for out,err in outputs))
 def test_export_response_duplicates_and_falsey_wrong_types_refuse(self):
  for body in (b'{"partialSuccess":{"rejectedLogRecords":"1"},"partialSuccess":{}}',b'{"partialSuccess":false}',b'{"partialSuccess":{}}',b'',b'{"partialSuccess":null}',b'{"partialSuccess":[]}',b'{"unknown":"private-sentinel"}'):
   with self.receiver() as receiver:
    receiver.response_body=body;config=dataclasses.replace(self.config,endpoint=f'http://127.0.0.1:{receiver.server_port}/v1/logs');run=Run(config,'pilot');self.assertEqual(run.close(),'incomplete');manifest=json.loads((run.directory/'manifest.json').read_bytes());self.assertEqual(manifest['counts']['exported'],0)
 def test_wrong_otlp_response_content_type_refuses(self):
  with self.receiver() as receiver:
   receiver.response_type='text/plain';run=Run(dataclasses.replace(self.config,endpoint=f'http://127.0.0.1:{receiver.server_port}/v1/logs'),'pilot');self.assertEqual(run.close(),'incomplete');self.assertEqual(json.loads((run.directory/'manifest.json').read_bytes())['counts']['exported'],0)
 def test_record_overflow_is_incomplete_and_bounded(self):
  run=Run(dataclasses.replace(self.config,record_limit=4),'pilot')
  for _ in range(10):run.event('operation.started')
  self.assertEqual(run.close(),'incomplete');page=retrieve(run.directory);self.assertIn('record_limit',page['loss']);self.assertEqual(len(page['records']),4);self.assertLessEqual((run.directory/'events.jsonl').stat().st_size,4*4096)
 def test_pending_snapshot_append_rotation_expiry_and_readonly(self):
  run=Run(self.config,'pilot')
  for _ in range(60):run.event('operation.started')
  run.manifest();page=retrieve(run.directory);self.assertEqual(len(page['records']),50);cursor=page['cursor'];self.assertTrue(cursor)
  for _ in range(5):run.event('operation.started')
  second=retrieve(run.directory,cursor);self.assertEqual(len(second['records']),11);self.assertEqual(second['coverage']['snapshot_bytes'],page['coverage']['snapshot_bytes']);self.assertIsNone(second['cursor'])
  before=(run.directory/'manifest.json').stat().st_mtime_ns;retrieve(run.directory);self.assertEqual(before,(run.directory/'manifest.json').stat().st_mtime_ns)
  decoded=json.loads(base64.urlsafe_b64decode(cursor));decoded['state']['line']+=1
  forged=base64.urlsafe_b64encode(packed(decoded)).decode()
  with self.assertRaises(DiagnosticError):retrieve(run.directory,forged)
  with patch('reliability.diagnostics.time.time',return_value=time.time()+61):
   with self.assertRaises(DiagnosticError):retrieve(run.directory,cursor)
  run.close();path=run.directory/'events.jsonl';replacement=path.with_suffix('.replacement');replacement.write_bytes(path.read_bytes());os.replace(replacement,path)
  with self.assertRaises(DiagnosticError):retrieve(run.directory,cursor)
 def test_symlink_malformed_and_unknown_schema_refuse(self):
  run=Run(self.config,'pilot');run.close();path=run.directory/'events.jsonl';original=path.read_bytes();path.unlink();path.symlink_to(self.root/'outside')
  with self.assertRaises(OSError):retrieve(run.directory)
  path.unlink();path.write_bytes(original.replace(b'"schema_version":1',b'"schema_version":2',1))
  with self.assertRaises(DiagnosticError):retrieve(run.directory)
 def test_capture_denied_rejected_message_and_outage_are_visible(self):
  original=os.open
  def denied(path,*args,**kwargs):
   if pathlib.Path(path).name=='events.jsonl':raise PermissionError()
   return original(path,*args,**kwargs)
  with patch('reliability.diagnostics.os.open',side_effect=denied):
   run=Run(self.config,'pilot');self.assertEqual(run.close(),'incomplete');self.assertIn('capture_failed',json.loads((run.directory/'manifest.json').read_bytes())['loss'])
  run=Run(self.config,'pilot');run.logger.info('private-sentinel');self.assertEqual(run.close(),'incomplete');self.assertIn('rejected',retrieve(run.directory)['loss'])
  run=Run(dataclasses.replace(self.config,endpoint='http://127.0.0.1:1/v1/logs',export_timeout_seconds=.1),'pilot');self.assertEqual(run.close(),'incomplete');self.assertIn('export_failed',retrieve(run.directory)['loss'])
 def test_queue_overflow_shutdown_bound_and_retention(self):
  with self.receiver(delay=.5) as receiver:
   config=dataclasses.replace(self.config,endpoint=f'http://127.0.0.1:{receiver.server_port}/v1/logs',queue_limit=1,shutdown_timeout_seconds=.1,export_timeout_seconds=.2)
   run=Run(config,'pilot')
   for _ in range(20):run.event('operation.started')
   start=time.monotonic();self.assertEqual(run.close(),'incomplete');self.assertLess(time.monotonic()-start,.5);page=retrieve(run.directory);self.assertIn('queue_limit',page['loss'])
  config=dataclasses.replace(self.config,retained_runs=2)
  for _ in range(4):Run(config,'pilot').close()
  self.assertEqual(len(list(config.output_root.glob('weft-run-*'))),2)
if __name__=='__main__':unittest.main()
