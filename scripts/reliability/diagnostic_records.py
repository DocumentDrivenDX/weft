"""Read-only bounded diagnostic records; standard-library dependencies only."""
from __future__ import annotations
import base64,hashlib,hmac,json,os,pathlib,re,stat,time
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

