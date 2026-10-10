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
RESOURCE={'service.name':'weft-reliability','service.version':'0.1.0','deployment.environment.name':'development'}
SCOPE={'name':'weft.reliability','version':'1'}
EVENTS={'run.started':('Run started',9),'operation.started':('Operation started',9),'operation.completed':('Operation completed',9),'operation.failed':('Operation failed',17),'capture.loss':('Capture incomplete',13),'run.completed':('Run completed',9),'run.failed':('Run failed',17)}
OPERATIONS=frozenset(('source-mutations','cargo','module-check','formal-analysis','fresh-hosts','duplicate-key-guard','node-limit-guard','scan-type-filter','projection-distinct','decimal-through-double','presence-null-to-absence','module-pin-guard','pilot'))
OUTCOMES=frozenset(('pending','passed','failed','incomplete'))
UUID=re.compile(r'[0-9a-f]{32}')
class DiagnosticError(Exception):pass

def unique(pairs):
 result={}
 for key,value in pairs:
  if key in result:raise DiagnosticError()
  result[key]=value
 return result

def parsed(raw):return json.loads(raw,object_pairs_hook=unique)
def packed(value):return json.dumps(value,separators=(',',':'),sort_keys=True,ensure_ascii=True).encode()
def regular(path,limit):
 fd=os.open(path,os.O_RDONLY|os.O_NONBLOCK|getattr(os,'O_NOFOLLOW',0))
 with os.fdopen(fd,'rb') as f:
  info=os.fstat(f.fileno())
  if not stat.S_ISREG(info.st_mode) or info.st_uid!=os.getuid():raise DiagnosticError()
  data=f.read(limit+1)
  after=os.fstat(f.fileno())
  if info.st_size!=after.st_size or info.st_mtime_ns!=after.st_mtime_ns:raise DiagnosticError()
  if len(data)>limit:raise DiagnosticError()
 return data,info

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

RECORD_KEYS={'schema_version','timestamp_ns','observed_timestamp_ns','severity_number','severity_text','event_name','body','resource','scope','attributes'}
ATTRS={'weft.run.id','weft.attempt.id','weft.sequence','weft.operation','weft.outcome','weft.duration_ms'}
def validate_record(value):
 if type(value)!=dict or set(value) not in (RECORD_KEYS,RECORD_KEYS|{'trace_id','span_id','trace_flags'}):raise DiagnosticError()
 if type(value['schema_version'])!=int or value['schema_version']!=1 or value['resource']!=RESOURCE or value['scope']!=SCOPE:raise DiagnosticError()
 if type(value['event_name'])!=str or value['event_name'] not in EVENTS or type(value['severity_number'])!=int or value['severity_number'] not in (9,13,17) or (value['body'],value['severity_number'])!=EVENTS[value['event_name']] or value['severity_text']!={9:'INFO',13:'WARN',17:'ERROR'}[value['severity_number']]:raise DiagnosticError()
 for key in ('timestamp_ns','observed_timestamp_ns'):
  if type(value[key])!=str or not re.fullmatch('[0-9]{1,20}',value[key]):raise DiagnosticError()
 attrs=value['attributes']
 if type(attrs)!=dict or set(attrs)!=(ATTRS|{'weft.dropped'} if value['event_name']=='capture.loss' else ATTRS):raise DiagnosticError()
 if 'weft.dropped' in attrs and (type(attrs['weft.dropped'])!=int or not 0<=attrs['weft.dropped']<=1_000_000):raise DiagnosticError()
 for key in ('weft.run.id','weft.attempt.id'):
  if type(attrs[key])!=str or not UUID.fullmatch(attrs[key]):raise DiagnosticError()
 if type(attrs['weft.operation'])!=str or attrs['weft.operation'] not in OPERATIONS or type(attrs['weft.outcome'])!=str or attrs['weft.outcome'] not in OUTCOMES:raise DiagnosticError()
 if type(attrs['weft.sequence'])!=int or not 1<=attrs['weft.sequence']<=1_000_000 or type(attrs['weft.duration_ms'])!=int or not 0<=attrs['weft.duration_ms']<=3_600_000:raise DiagnosticError()
 if 'trace_id' in value:
  if type(value['trace_id'])!=str or not re.fullmatch('[0-9a-f]{32}',value['trace_id']) or int(value['trace_id'],16)==0 or type(value['span_id'])!=str or not re.fullmatch('[0-9a-f]{16}',value['span_id']) or int(value['span_id'],16)==0 or type(value['trace_flags'])!=int or not 0<=value['trace_flags']<=255:raise DiagnosticError()

def validate_revision(revision):
 if type(revision)!=dict or set(revision)!={'kind','value','scope'}:raise DiagnosticError()
 if revision!={'kind':'unknown','value':None,'scope':'not-supplied'} and not (revision['kind']=='sha256' and revision['scope']=='compiler-and-host-source' and type(revision['value'])==str and re.fullmatch('[0-9a-f]{64}',revision['value'])):raise DiagnosticError()

MANIFEST_KEYS={'schema_version','owner','run_id','attempt_id','operation','revision','started_ns','ended_ns','outcome','counts','loss','access','retention_runs','sources','export_enabled','input_revision','flush_state'}
COUNT_KEYS={'emitted','captured','dropped','export_queued','exported','export_failed','export_dropped','operations','operations_failed','export_pending'}
LOSS=frozenset(('rejected','capture_failed','record_limit','queue_limit','export_failed','shutdown_timeout','retention_failed'))
def read_manifest(directory):
 if directory.is_symlink() or not directory.is_dir() or directory.stat().st_uid!=os.getuid():raise DiagnosticError()
 raw,_=regular(directory/'manifest.json',65536);m=parsed(raw)
 if type(m)!=dict or set(m)!=MANIFEST_KEYS or type(m['schema_version'])!=int or m['schema_version']!=1 or m['owner']!='weft-reliability':raise DiagnosticError()
 if type(m['run_id'])!=str or not UUID.fullmatch(m['run_id']) or directory.name!='weft-run-'+m['run_id']:raise DiagnosticError()
 if type(m['attempt_id'])!=str or not UUID.fullmatch(m['attempt_id']) or type(m['operation'])!=str or m['operation'] not in OPERATIONS or type(m['outcome'])!=str or m['outcome'] not in OUTCOMES or m['access']!='owner-only' or m['sources']!=['events.jsonl'] or type(m['export_enabled'])!=bool:raise DiagnosticError()
 if type(m['counts'])!=dict or set(m['counts'])!=COUNT_KEYS or any(type(v)!=int or not 0<=v<=1_000_000 for v in m['counts'].values()):raise DiagnosticError()
 if type(m['loss'])!=list or any(type(v)!=str or v not in LOSS for v in m['loss']) or m['loss']!=sorted(set(m['loss'])):raise DiagnosticError()
 for key in ('started_ns','ended_ns'):
  if key=='ended_ns' and m[key] is None:continue
  if type(m[key])!=str or not re.fullmatch('[0-9]{1,20}',m[key]):raise DiagnosticError()
 if type(m['revision'])!=int or not 0<=m['revision']<=1_000_000 or type(m['retention_runs'])!=int or not 1<=m['retention_runs']<=32:raise DiagnosticError()
 if m['counts']['captured']>m['counts']['emitted'] or m['counts']['operations_failed']>m['counts']['operations'] or m['counts']['exported']+m['counts']['export_failed']+m['counts']['export_pending']!=m['counts']['export_queued']:raise DiagnosticError()
 if (m['outcome']=='pending')!=(m['ended_ns'] is None):raise DiagnosticError()
 if m['flush_state'] not in ('pending','complete','failed','timeout'):raise DiagnosticError()
 validate_revision(m['input_revision'])
 return m

def retrieve(directory:pathlib.Path,cursor=None):
 """Read-only pages; bounded whole-file hashing binds immutable snapshot prefixes."""
 started=time.monotonic();m=read_manifest(directory)
 secret,_=regular(directory/'cursor.key',32)
 if len(secret)!=32:raise DiagnosticError()
 raw,info=regular(directory/'events.jsonl',1048576)
 complete=raw[:raw.rfind(b'\n')+1] if raw else b''
 if m['outcome']!='pending' and raw!=complete and 'capture_failed' not in m['loss']:raise DiagnosticError()
 lines=complete.splitlines(keepends=True)
 if len(lines)>256 or m['counts']['captured']>256:raise DiagnosticError()
 if m['outcome']=='pending':
  if len(lines)<m['counts']['captured']:raise DiagnosticError()
  lines=lines[:m['counts']['captured']]
 else:
  if len(lines)!=m['counts']['captured']:raise DiagnosticError()
 sequence=0;validated=[];boundary=0
 for number,line in enumerate(lines,1):
  if time.monotonic()-started>=1 or len(line)>4096:raise DiagnosticError()
  value=parsed(line);validate_record(value);attrs=value['attributes']
  if attrs['weft.run.id']!=m['run_id'] or attrs['weft.sequence']<=sequence or (attrs['weft.sequence']!=sequence+1 and not m['loss']):raise DiagnosticError()
  sequence=attrs['weft.sequence'];boundary+=len(line);validated.append((number,value,len(line)))
 if sequence>m['revision'] or (m['outcome']!='pending' and not m['loss'] and sequence!=m['revision']):raise DiagnosticError()
 identity=[info.st_dev,info.st_ino]
 if cursor is None:
  state={'run':m['run_id'],'identity':identity,'snapshot':boundary,'offset':0,'line':1,'sequence':0,'end_sequence':sequence,'digest':hashlib.sha256(raw[:boundary]).hexdigest(),'expiry':time.time()+60}
 else:
  if type(cursor)!=str or len(cursor)>2048:raise DiagnosticError()
  try:
   decoded=parsed(base64.urlsafe_b64decode(cursor));state=decoded['state']
   if set(decoded)!={'state','sha256'} or not hmac.compare_digest(decoded['sha256'],hmac.new(secret,packed(state),hashlib.sha256).hexdigest()):raise DiagnosticError()
  except (ValueError,KeyError,TypeError):raise DiagnosticError() from None
  if type(state)!=dict or set(state)!={'run','identity','snapshot','offset','line','sequence','end_sequence','digest','expiry'} or state['run']!=m['run_id'] or state['identity']!=identity or type(state['expiry']) not in (float,int) or not time.time()<=state['expiry']<=time.time()+60:raise DiagnosticError()
  for key in ('snapshot','offset','line','sequence','end_sequence'):
   if type(state[key])!=int or not 0<=state[key]<=1048576:raise DiagnosticError()
  if not state['offset']<=state['snapshot']<=len(raw) or state['line']<1 or state['digest']!=hashlib.sha256(raw[:state['snapshot']]).hexdigest():raise DiagnosticError()
 records=[];output=0;position=0
 for number,value,size in validated:
  if position>=state['snapshot']:break
  end=position+size
  if end>state['snapshot']:raise DiagnosticError()
  if position>=state['offset']:
   if len(records)>=50 or time.monotonic()-started>=1:break
   entry={'source':'events.jsonl','line':number,'record':value};length=len(packed(entry))
   if output+length>60000:break
   if number!=state['line'] or value['attributes']['weft.sequence']<=state['sequence']:raise DiagnosticError()
   records.append(entry);output+=length;state['line']+=1;state['sequence']=value['attributes']['weft.sequence'];state['offset']=end
  position=end
 continuation=None
 if state['offset']<state['snapshot']:continuation=base64.urlsafe_b64encode(packed({'state':state,'sha256':hmac.new(secret,packed(state),hashlib.sha256).hexdigest()})).decode()
 result={'schema_version':1,'run_id':m['run_id'],'input_revision':m['input_revision'],'flush_state':m['flush_state'],'records':records,'cursor':continuation,'coverage':{'snapshot_bytes':state['snapshot'],'snapshot_end_sequence':state['end_sequence'],'through_sequence':state['sequence'],'scanned_bytes':len(raw),'captured':m['counts']['captured'],'emitted':m['counts']['emitted']},'outcome':m['outcome'],'loss':m['loss'],'truncated':continuation is not None}
 if len(packed(result))>65536 or time.monotonic()-started>=1:raise DiagnosticError()
 return result

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
