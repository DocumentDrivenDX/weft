"""Host diagnostics: safe logger → real SDK → local capture / OTLP HTTP JSON."""
from __future__ import annotations
import fcntl, socket, http.client, urllib.parse, io, base64, contextlib, importlib.metadata, sys, dataclasses, hashlib, hmac, json, logging, os, pathlib, queue, re, shutil, stat, threading, time, urllib.request, uuid
from opentelemetry._logs import LogRecord,SeverityNumber
from opentelemetry.sdk._logs import LoggerProvider,LogRecordProcessor,ReadableLogRecord,LogRecordLimits
from opentelemetry.sdk.resources import Resource
from opentelemetry.sdk.util.instrumentation import InstrumentationScope
from opentelemetry.sdk.trace import TracerProvider,SpanProcessor,ReadableSpan
from opentelemetry.sdk.trace.sampling import ALWAYS_ON
from opentelemetry.trace import SpanContext,TraceState
from opentelemetry.exporter.otlp.proto.common._internal._log_encoder import encode_logs
from opentelemetry.exporter.otlp.proto.common._internal.trace_encoder import encode_spans
from google.protobuf.json_format import MessageToDict
from reliability.diagnostic_records import (RESOURCE, SCOPE, EVENTS, OPERATIONS, OUTCOMES, UUID, DiagnosticError, unique, parsed, packed, regular, RECORD_KEYS, ATTRS, validate_record, validate_revision, MANIFEST_KEYS, COUNT_KEYS, LOSS, read_manifest, retrieve)

def wire(message):
 value=MessageToDict(message,preserving_proto_field_name=False,use_integers_for_enums=True)
 def walk(obj):
  if isinstance(obj,dict):
   for k,v in list(obj.items()):
    if k in ('traceId','spanId','parentSpanId'):obj[k]=base64.b64decode(v,validate=True).hex()
    else:walk(v)
  elif isinstance(obj,list):
   for v in obj:walk(v)
 walk(value)
 return packed(value)

class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs):return None

class Bridge(logging.Handler):
 def __init__(self,owner):super().__init__();self.owner=owner
 def emit(self,record):
  # Arbitrary message and extras are never interpolated or projected.
  event=getattr(record,'weft_event',None);operation=getattr(record,'weft_operation',None)
  if type(event)!=str or event not in EVENTS or type(operation)!=str or operation not in OPERATIONS:self.owner.loss('rejected');return
  outcome=getattr(record,'weft_outcome','pending');duration=getattr(record,'weft_duration_ms',0)
  attempt=getattr(record,'weft_attempt',self.owner.attempt)
  if type(outcome)!=str or outcome not in OUTCOMES or type(duration)!=int or not 0<=duration<=3_600_000 or type(attempt)!=str or not UUID.fullmatch(attempt):self.owner.loss('rejected');return
  owner=self.owner;owner.sequence+=1;owner.counts['emitted']+=1
  if event=='operation.started':owner.counts['operations']+=1
  if event=='operation.failed':owner.counts['operations_failed']+=1
  body,number=EVENTS[event];attrs={'weft.run.id':owner.id,'weft.attempt.id':attempt,'weft.sequence':owner.sequence,'weft.operation':operation,'weft.outcome':outcome,'weft.duration_ms':duration}
  if event=='capture.loss':attrs['weft.dropped']=owner.counts['dropped']
  owner.sdk.emit(LogRecord(timestamp=time.time_ns(),observed_timestamp=time.time_ns(),severity_number=SeverityNumber(number),severity_text={9:'INFO',13:'WARN',17:'ERROR'}[number],body=body,attributes=attrs,event_name=event))
  try:owner.manifest()
  except OSError:owner.loss('capture_failed')

class Capture(LogRecordProcessor):
 def __init__(self,owner):self.owner=owner
 def on_emit(self,record):
  owner=self.owner;r=record.log_record
  value={'schema_version':1,'timestamp_ns':str(r.timestamp),'observed_timestamp_ns':str(r.observed_timestamp),'severity_number':r.severity_number.value,'severity_text':r.severity_text,'event_name':r.event_name,'body':r.body,'resource':RESOURCE,'scope':SCOPE,'attributes':dict(r.attributes)}
  if r.trace_id and r.span_id:value.update(trace_id=f'{r.trace_id:032x}',span_id=f'{r.span_id:016x}',trace_flags=int(r.trace_flags))
  try:
   validate_record(value)
   data=packed(value)+b'\n'
   if len(data)>4096 or record.dropped_attributes:raise DiagnosticError()
   if owner.counts['captured']>=owner.config.record_limit:owner.loss('record_limit');return
   if owner.stream is None:owner.loss('capture_failed');return
   owner.stream.write(data);owner.stream.flush();owner.counts['captured']+=1
   readable=ReadableLogRecord(r,owner.resource,InstrumentationScope(**SCOPE))
   owner.enqueue('logs',wire(encode_logs((readable,))))
  except (OSError,ValueError,TypeError,DiagnosticError):owner.loss('capture_failed')
 def shutdown(self):pass
 def force_flush(self,timeout_millis=30000):return True

class TraceCapture(SpanProcessor):
 def __init__(self,owner):self.owner=owner
 def on_start(self,span,parent_context=None):pass
 def on_end(self,span):
  # Export actual SDK context/times with only repository-owned semantic fields.
  if span.name not in OPERATIONS:self.owner.loss('rejected');return
  context=SpanContext(span.context.trace_id,span.context.span_id,span.context.is_remote,span.context.trace_flags,TraceState())
  parent=SpanContext(span.parent.trace_id,span.parent.span_id,span.parent.is_remote,span.parent.trace_flags,TraceState()) if span.parent else None
  safe=ReadableSpan(name=span.name,context=context,parent=parent,resource=self.owner.resource,attributes={'weft.run.id':self.owner.id},start_time=span.start_time,end_time=span.end_time,instrumentation_scope=InstrumentationScope(**SCOPE))
  self.owner.enqueue('traces',wire(encode_spans((safe,))))
 def shutdown(self):pass
 def force_flush(self,timeout_millis=30000):return True

class Run:
 def __init__(self,config,operation,input_revision=None):
  if operation not in OPERATIONS:raise DiagnosticError()
  config.validate_diagnostics_environment()
  for package in ('opentelemetry-sdk','opentelemetry-proto','opentelemetry-exporter-otlp-proto-common'):
   if importlib.metadata.version(package)!='1.45.1':raise DiagnosticError()
  self.input_revision={'kind':'unknown','value':None,'scope':'not-supplied'} if input_revision is None else input_revision
  validate_revision(self.input_revision)
  self.input_revision=dict(self.input_revision);self.flush_state='pending';self.guard=None
  self.config=config;self.operation=operation;self.id=uuid.uuid4().hex;self.attempt=uuid.uuid4().hex;self.sequence=0;self.closed=False;self.started=time.time_ns();self.ended=None;self.outcome='pending';self.losses=set();self.lock=threading.RLock()
  self.counts={k:0 for k in ('emitted','captured','dropped','export_queued','exported','export_failed','export_dropped','operations','operations_failed','export_pending')}
  config.output_root.mkdir(parents=True,exist_ok=True)
  if config.output_root.is_symlink():raise DiagnosticError()
  self.guard=os.open(config.output_root/'.weft-run.lock',os.O_RDWR|os.O_CREAT|getattr(os,'O_NOFOLLOW',0),0o600)
  try:
   info=os.fstat(self.guard)
   if not stat.S_ISREG(info.st_mode) or info.st_uid!=os.getuid():raise DiagnosticError()
   fcntl.flock(self.guard,fcntl.LOCK_EX|fcntl.LOCK_NB)
   self.prune()
   self.directory=config.output_root/('weft-run-'+self.id);self.directory.mkdir(mode=0o700)
   with os.fdopen(os.open(self.directory/'cursor.key',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'wb') as stream:stream.write(os.urandom(32))
   self.stream=None
   try:self.stream=os.fdopen(os.open(self.directory/'events.jsonl',os.O_WRONLY|os.O_CREAT|os.O_EXCL,0o600),'wb')
   except OSError:self.loss('capture_failed')
   self.queue=queue.Queue(config.queue_limit);self.stop=threading.Event();self.export_cancel=threading.Event();self.worker=None
   self.resource=Resource(RESOURCE)
   self.provider=LoggerProvider(resource=self.resource,shutdown_on_exit=False,log_record_limits=LogRecordLimits(max_log_record_attributes=32,max_log_record_attribute_length=64));self.provider.add_log_record_processor(Capture(self));self.sdk=self.provider.get_logger(**SCOPE)
   self.traces=TracerProvider(resource=self.resource,shutdown_on_exit=False,sampler=ALWAYS_ON);self.traces.add_span_processor(TraceCapture(self));self.tracer=self.traces.get_tracer(SCOPE["name"],SCOPE["version"])
   self.logger=logging.Logger('weft.reliability',logging.INFO);self.logger.propagate=False;self.logger.addHandler(Bridge(self))
   if config.endpoint:self.worker=threading.Thread(target=self.export_loop,daemon=True);self.worker.start()
   self.manifest();self.event('run.started');print('weft-runner: run started '+self.id,file=sys.stderr)
  except BaseException:
   os.close(self.guard);self.guard=None;raise DiagnosticError() from None
 def loss(self,reason):
  with self.lock:
   first=reason not in self.losses;self.losses.add(reason);self.counts['dropped']+=1
  if first:print('weft-runner: capture incomplete',file=sys.stderr)
 def event(self,event,operation=None,outcome='pending',duration_ms=0,attempt=None):
  if self.closed:raise DiagnosticError()
  self.logger.info('',extra={'weft_event':event,'weft_operation':operation or self.operation,'weft_outcome':outcome,'weft_duration_ms':duration_ms,'weft_attempt':attempt or self.attempt})
 @contextlib.contextmanager
 def operation_context(self,operation,attempt=None):
  start=time.monotonic()
  with self.tracer.start_as_current_span(operation):
   self.event('operation.started',operation,attempt=attempt)
   try:yield
   except BaseException:
    self.event('operation.failed',operation,'failed',min(3_600_000,round((time.monotonic()-start)*1000)),attempt);raise
   else:self.event('operation.completed',operation,'passed',min(3_600_000,round((time.monotonic()-start)*1000)),attempt)
 def enqueue(self,kind,data):
  if not self.config.endpoint:return
  with self.lock:
   try:self.queue.put_nowait((kind,data));self.counts['export_queued']+=1
   except queue.Full:self.counts['export_dropped']+=1;self.loss('queue_limit')
 def export_loop(self):
  from .process import run
  while not self.stop.is_set() or not self.queue.empty():
   try:kind,data=self.queue.get(timeout=.02)
   except queue.Empty:continue
   success=False
   if not self.export_cancel.is_set():
    try:
     endpoint=self.config.endpoint if kind=='logs' else self.config.endpoint[:-4]+'traces'
     result=run([sys.executable,str(pathlib.Path(__file__).with_name('transport.py')),endpoint],self.directory,self.config.command_environment(),self.config.export_timeout_seconds,input_data=data,cancel=self.export_cancel)
     success=not result.timed_out and result.exit_code==0 and result.bytes_read==0
    except Exception:pass
   with self.lock:
    self.counts['exported' if success else 'export_failed']+=1
    if not success:self.losses.add('export_failed')
   self.queue.task_done()
 def manifest(self):
  with self.lock:
   counts=dict(self.counts);counts['export_pending']=counts['export_queued']-counts['exported']-counts['export_failed']
  value={'schema_version':1,'owner':'weft-reliability','run_id':self.id,'attempt_id':self.attempt,'operation':self.operation,'revision':self.sequence,'started_ns':str(self.started),'ended_ns':str(self.ended) if self.ended else None,'outcome':self.outcome,'counts':counts,'loss':sorted(self.losses),'access':'owner-only','retention_runs':self.config.retained_runs,'sources':['events.jsonl'],'export_enabled':bool(self.config.endpoint),'input_revision':self.input_revision,'flush_state':self.flush_state}
  temp=self.directory/'manifest.tmp'
  fd=os.open(temp,os.O_WRONLY|os.O_CREAT|os.O_TRUNC|getattr(os,'O_NOFOLLOW',0),0o600)
  with os.fdopen(fd,'wb') as stream:stream.write(packed(value))
  os.replace(temp,self.directory/'manifest.json')
 def close(self,failed=False):
  if self.closed:return self.outcome
  if self.losses:self.event('capture.loss',outcome='incomplete')
  failed=failed or self.counts['operations_failed']>0
  self.event('run.failed' if failed else 'run.completed',outcome='failed' if failed else 'passed')
  self.closed=True;self.stop.set()
  if self.worker:
   self.worker.join(self.config.shutdown_timeout_seconds)
   if self.worker.is_alive():self.losses.add('shutdown_timeout');self.export_cancel.set()
  if self.stream:
   try:self.stream.close()
   except OSError:self.losses.add('capture_failed')
  if self.counts['emitted']!=self.counts['captured']:self.losses.add('capture_failed')
  self.ended=time.time_ns();self.outcome='incomplete' if self.losses else 'failed' if failed else 'passed'
  self.flush_state='timeout' if 'shutdown_timeout' in self.losses else 'failed' if self.losses else 'complete'
  try:
   self.manifest();self.provider.shutdown();self.traces.shutdown()
   try:self.prune()
   except (OSError,DiagnosticError):
    self.losses.add('retention_failed');self.outcome='incomplete';self.flush_state='failed';self.manifest()
   print('weft-runner: run '+self.outcome+' '+self.id,file=sys.stderr);return self.outcome
  finally:
   if self.guard is not None:os.close(self.guard);self.guard=None
 def prune(self):
  closed=[];pending=0
  for path in self.config.output_root.glob('weft-run-*'):
   if not UUID.fullmatch(path.name.removeprefix('weft-run-')):continue
   if path.is_symlink() or not path.is_dir():raise DiagnosticError()
   try:
    manifest=read_manifest(path)
    if manifest['outcome']=='pending':pending+=1
    else:
     entries=list(path.iterdir());names={entry.name for entry in entries}
     if not {'manifest.json','cursor.key'}<=names<={'events.jsonl','manifest.json','cursor.key','qualification.json'}:raise DiagnosticError()
     if any(not stat.S_ISREG(entry.lstat().st_mode) or entry.lstat().st_uid!=os.getuid() for entry in entries):raise DiagnosticError()
     closed.append((int(manifest['started_ns']),path,manifest))
   except (OSError,ValueError,DiagnosticError):raise DiagnosticError() from None
  if pending>=self.config.retained_runs:raise DiagnosticError()
  for _,path,manifest in sorted(closed)[:-self.config.retained_runs]:
   entries=list(path.iterdir());names={p.name for p in entries}
   if not {'manifest.json','cursor.key'}<=names<={'events.jsonl','manifest.json','cursor.key','qualification.json'}:raise DiagnosticError()
   if 'events.jsonl' not in names and not (manifest['counts']['captured']==0 and 'capture_failed' in manifest['loss']):raise DiagnosticError()
   for entry in entries:
    info=entry.lstat()
    if not stat.S_ISREG(info.st_mode) or info.st_uid!=os.getuid():raise DiagnosticError()
   shutil.rmtree(path)

def source_revision(root:pathlib.Path):
 """Scoped code identity; fixture/native evidence custody belongs to R6."""
 paths=set(root.glob('Cargo.*'))|{root/'rust-toolchain.toml',root/'tests/qualify-and-evolve/source-mutations.py',root/'docs/helix/02-design/module-boundaries.json'}
 paths.update(root.glob('crates/**/Cargo.toml'));paths.update(root.glob('crates/**/*.rs'));paths.update(root.glob('scripts/reliability/*.py'));paths.update(root.glob('scripts/reliability/requirements.txt'))
 digest=hashlib.sha256()
 for path in sorted(paths):
  if path.is_symlink() or not path.is_file():raise DiagnosticError()
  digest.update(str(path.relative_to(root)).encode());digest.update(b'\x00')
  with path.open('rb') as stream:
   for chunk in iter(lambda:stream.read(65536),b''):digest.update(chunk)
  digest.update(b'\x00')
 return {'kind':'sha256','value':digest.hexdigest(),'scope':'compiler-and-host-source'}
