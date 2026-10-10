"""Explicit same-binary corpus producer; never index, native or host authority.

Expected case bytes and capability/refusal mapping are independently reviewed
inputs. The checker never asks its tested executable to generate expectations.
"""
from dataclasses import dataclass
from pathlib import Path
import argparse, hashlib, json, os, signal, subprocess, selectors, time, stat

LIMIT = 16 * 1024 * 1024
CONTROL_IDS = ('candidate-opt-out','unknown-backend','wrong-backend-version','wrong-profile',
 'unknown-envelope-member','mismatched-interface-dialect','binding-digest-mismatch',
 'model-digest-mismatch','sql-byte-limit','binding-byte-limit','duplicate-envelope-member',
 'missing-publication','negative-table-version','duplicate-table-uuid','missing-table-mapping',
 'inconsistent-model-pin','fresh-binding-0','fresh-binding-1','fresh-binding-2')

class Refusal(ValueError):
    pass

def sha(raw): return hashlib.sha256(raw).hexdigest()
def read(path, maximum):
    if not isinstance(path, Path) or path.is_symlink() or not path.is_file(): raise Refusal('regular-input-required')
    with path.open('rb') as stream: raw=stream.read(maximum+1)
    if len(raw)>maximum: raise Refusal('input-limit')
    return raw

def document(raw):
    def pairs(items):
        out={}
        for key,value in items:
            if key in out: raise Refusal('duplicate-json-key')
            out[key]=value
        return out
    def integer(token):
        if len(token.lstrip('-'))>20: raise Refusal('numeric-limit')
        return int(token)
    def forbidden(token): raise Refusal('numeric-shape')
    try:return json.loads(raw.decode('utf-8'),object_pairs_hook=pairs,parse_int=integer,parse_float=forbidden,parse_constant=forbidden)
    except (UnicodeError,json.JSONDecodeError):raise Refusal('invalid-json') from None

def encoded(value):return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()

@dataclass(frozen=True)
class Config:
    source: Path
    binary: Path
    cases: Path
    backend: Path
    output: Path
    source_commit: str
    binary_sha256: str
    maximum_input_bytes: int
    maximum_response_bytes: int
    timeout_seconds: int
    source_inventory: Path
    source_inventory_sha256: str
    schema_checker: Path
    schema_checker_sha256: str
    maximum_source_files: int
    maximum_source_file_bytes: int
    maximum_source_total_bytes: int
    maximum_cases: int
    maximum_receipt_bytes: int
    cases_sha256: str
    backend_sha256: str
    declared_capability_count: int
    def __post_init__(self):
        if type(self.declared_capability_count)is not int or not 1<=self.declared_capability_count<=512:raise Refusal('declared-capability-count')
        if any(not isinstance(p,Path) or not p.is_absolute() for p in (self.source,self.binary,self.cases,self.backend,self.output,self.source_inventory,self.schema_checker)):raise Refusal('absolute-settings-required')
        if any(len(d)!=64 or any(c not in '0123456789abcdef' for c in d) for d in (self.source_inventory_sha256,self.schema_checker_sha256,self.cases_sha256,self.backend_sha256)):raise Refusal('digest-setting')
        if any(type(n)is not int or n<=0 or n>2*1024*1024*1024 for n in (self.maximum_source_files,self.maximum_source_file_bytes,self.maximum_source_total_bytes,self.maximum_cases,self.maximum_receipt_bytes)):raise Refusal('source-bound-setting')
        if len(self.source_commit)!=40 or len(self.binary_sha256)!=64 or any(c not in '0123456789abcdef' for c in self.source_commit+self.binary_sha256):raise Refusal('digest-setting')
        if any(type(n)is not int or n<=0 or n>128*1024*1024 for n in (self.maximum_input_bytes,self.maximum_response_bytes,self.timeout_seconds)):raise Refusal('finite-settings-required')

def verify_source(config):
    if config.source.is_symlink() or not config.source.is_dir():raise Refusal('source-root')
    raw=read(config.source_inventory,config.maximum_input_bytes)
    if sha(raw)!=config.source_inventory_sha256:raise Refusal('source-inventory-pin')
    inventory=document(raw)
    if set(inventory)!= {'sourceCommit','files'} or inventory['sourceCommit']!=config.source_commit:raise Refusal('source-inventory-shape')
    files=inventory['files']
    if type(files)is not list or not files or len(files)>config.maximum_source_files:raise Refusal('source-count')
    paths=[];directories={''};total=0
    for desc in files:
        if set(desc)!= {'path','mode','gitBlob','sha256','bytes'}:raise Refusal('source-entry')
        name=desc['path']
        if type(name)is not str or name.startswith('/') or '\\' in name or any(x in ('','.', '..') for x in name.split('/')):raise Refusal('source-path')
        if desc['mode'] not in ('100644','100755') or type(desc['bytes'])is not int or not 0<=desc['bytes']<=config.maximum_source_file_bytes:raise Refusal('source-file-bound')
        total+=desc['bytes']
        if total>config.maximum_source_total_bytes:raise Refusal('source-total-bound')
        paths.append(name)
        parent=Path(name).parent
        while str(parent)!='.':directories.add(parent.as_posix());parent=parent.parent
        p=config.source/name
        parent=p.parent
        while parent!=config.source:
            if parent.is_symlink():raise Refusal('source-parent-symlink')
            parent=parent.parent
        if p.is_symlink() or not p.is_file():raise Refusal('source-regular-required')
        if bool(p.stat().st_mode&0o111)!=(desc['mode']=='100755'):raise Refusal('source-mode')
        blob=hashlib.sha1(b'blob '+str(desc['bytes']).encode()+b'\0');digest=hashlib.sha256();remaining=desc['bytes']
        with p.open('rb') as stream:
            while remaining:
                chunk=stream.read(min(65536,remaining))
                if not chunk:raise Refusal('source-length')
                remaining-=len(chunk);blob.update(chunk);digest.update(chunk)
            if stream.read(1):raise Refusal('source-length')
        if blob.hexdigest()!=desc['gitBlob'] or digest.hexdigest()!=desc['sha256']:raise Refusal('source-file-pin')
    if paths!=sorted(set(paths)):raise Refusal('source-inventory-order')
    expected=set(paths);pending=[config.source];seen=set()
    while pending:
        directory=pending.pop()
        with os.scandir(directory) as entries:
            for entry in entries:
                relative=Path(entry.path).relative_to(config.source).as_posix();mode=entry.stat(follow_symlinks=False).st_mode
                if stat.S_ISDIR(mode) and relative in directories:pending.append(Path(entry.path))
                elif stat.S_ISREG(mode) and relative in expected:seen.add(relative)
                else:raise Refusal('source-extra-or-nonregular')
    if seen!=expected:raise Refusal('source-inventory-missing')
    return raw

# Existing reviewed migrations only; originals remain in each report.
def migrate_legacy(identity,response):
    result=document(encoded(response));changes=[]
    if identity=='unsigned-columns:unsigned-64':
        expected={'diagnostics':[{'code':'WFT-BINDING','message':'Native column carrier cannot establish the original logical scalar domain','phase':'binding','recoverability':'correct-input','severity':'error'}],'interfaceVersion':'weft-compile/0.1.0','status':'blocked'}
        if result!=expected:raise Refusal('migration-precondition')
        result['diagnostics'][0]['phase']='lower';changes.append({'path':'/diagnostics/0/phase','old':'binding','new':'lower'})
    for obligation in result.get('obligations',[]):
        if obligation['id']=='ashlar.candidate.publication':
            profile=obligation['parameters']['nativeProfile']
            if 'versionReported' in profile:
                if profile['versionReported']!='4.2.0 zero build hash':raise Refusal('migration-precondition')
                del profile['versionReported'];changes.append({'path':'/obligations/ashlar.candidate.publication/parameters/nativeProfile/versionReported','old':'4.2.0 zero build hash','new':None})
    for operation in result.get('qualification',{}).get('operations',[]):
        declaration=operation['declaration']
        if declaration['id']=='value.presence' and declaration['logicalDomain']=={'subset':'optional scalar envelopes; absent or exact value; explicit native null refuses'}:
            declaration['logicalDomain']={'subset':'optional scalar or compound envelopes; absent or exact value; explicit native null refuses'}
            changes.append({'path':'/qualification/operations/value.presence/declaration/logicalDomain','old':'optional scalar','new':'optional scalar or compound'})
    return result,changes

def extract_legacy(source,maximum):
    evidence=source/'docs/helix/04-build/evidence';cases=[];sources=[]
    def load(path):
        raw=read(path,maximum);sources.append({'path':str(path.relative_to(source)),'sha256':sha(raw),'bytes':len(raw)});return raw
    for scope in ('columns-native','application-native','key-refusal','unsigned-columns','optional-native','relationship-native','compound-native','compound-boundaries-native','compound-application-native'):
        for line in load(evidence/('B-006-'+scope)/'compile-artifacts.jsonl').splitlines():
            case=document(line);case['id']=scope+':'+str(case['id']);cases.append(case)
    for scope in ('scalar-native','global-native'):
        for path in sorted((evidence/('B-006-'+scope)).glob('*-compile.json')):
            case=document(load(path));case['id']=scope+':'+path.stem;cases.append(case)
    case=document(load(evidence/'B-006-cross-module-native/compile.json'));case['id']='cross-module';cases.append(case)
    if len(cases)!=463 or len({x['id'] for x in cases})!=463:raise Refusal('legacy-inventory')
    return cases,sources

def transport(binary,data,mode,config):
    """One deadline covers process, descendants and every pipe; no threads."""
    if mode not in ('normal','directory-input','closed-output'):raise Refusal('transport-mode')
    owned=None;process=None;primary=None;cleanup_failed=False
    selector=selectors.DefaultSelector();streams=[];out=bytearray();err=bytearray()
    deadline=time.monotonic()+config.timeout_seconds
    try:
        stdin=subprocess.PIPE
        if mode=='directory-input':owned=os.open(config.source,os.O_RDONLY);stdin=owned
        process=subprocess.Popen([str(binary)],stdin=stdin,stdout=subprocess.PIPE,stderr=subprocess.PIPE,start_new_session=True)
        streams=[p for p in (process.stdout,process.stderr,process.stdin) if p is not None]
        if owned is not None:os.close(owned);owned=None
        for stream,label in ((process.stdout,'out'),(process.stderr,'err')):
            if label=='out' and mode=='closed-output':stream.close();continue
            os.set_blocking(stream.fileno(),False);selector.register(stream,selectors.EVENT_READ,label)
        remaining=memoryview(data)
        if process.stdin is not None:
            os.set_blocking(process.stdin.fileno(),False)
            if remaining:selector.register(process.stdin,selectors.EVENT_WRITE,'in')
            else:process.stdin.close()
        while selector.get_map() or process.poll() is None:
            available=deadline-time.monotonic()
            if available<=0:raise Refusal('transport-deadline')
            for key,event in selector.select(min(available,0.05)):
                if key.data=='in':
                    try:written=os.write(key.fd,remaining[:65536]);remaining=remaining[written:]
                    except BrokenPipeError:remaining=memoryview(b'')
                    if not remaining:selector.unregister(key.fileobj);key.fileobj.close()
                else:
                    dest,maximum=(out,config.maximum_response_bytes) if key.data=='out' else (err,4096)
                    chunk=os.read(key.fd,min(65536,maximum+1-len(dest)))
                    if not chunk:selector.unregister(key.fileobj);key.fileobj.close();continue
                    dest.extend(chunk)
                    if len(dest)>maximum:raise Refusal('capture-limit')
        return process.returncode,bytes(out),bytes(err)
    except BaseException as error:
        primary=error
        raise
    finally:
        if owned is not None:
            try:os.close(owned)
            except BaseException:cleanup_failed=True
        # Kill the whole group even if the leader exited but descendants hold pipes.
        if process is not None:
            try:os.killpg(process.pid,signal.SIGKILL)
            except ProcessLookupError:pass
            except BaseException:cleanup_failed=True
        for stream in streams:
            try:stream.close()
            except BaseException:cleanup_failed=True
        try:selector.close()
        except BaseException:cleanup_failed=True
        if process is not None:
            try:process.wait(timeout=1)
            except BaseException:cleanup_failed=True
        if cleanup_failed:
            if primary is not None:primary.cleanup_failed=True
            else:raise Refusal('transport-cleanup')


def qualify(config,*,execute,validate_schema):
    """Trusted schema port must validate exact selected request/response schemas.

    Returning None is its closed verification contract, not an authority claim.
    """
    if config.output.exists():raise Refusal('fresh-output-required')
    inventory_raw=verify_source(config)
    harness=read(Path(__file__).resolve(),config.maximum_input_bytes)
    checker=read(config.schema_checker,32*1024*1024)
    if sha(checker)!=config.schema_checker_sha256:raise Refusal('schema-checker-pin')
    binary=read(config.binary,32*1024*1024)
    if sha(binary)!=config.binary_sha256:raise Refusal('binary-pin')
    bundle_raw=read(config.cases,config.maximum_input_bytes);backend_raw=read(config.backend,config.maximum_input_bytes)
    if sha(bundle_raw)!=config.cases_sha256 or sha(backend_raw)!=config.backend_sha256:raise Refusal('corpus-input-pin')
    bundle=document(bundle_raw);backend=document(backend_raw)
    if set(bundle)!= {'sourceCommit','paths','controls','coverage','sources','legacyNamespaceRefusal'} or bundle['sourceCommit']!=config.source_commit:raise Refusal('case-envelope')
    source_hashes=[]
    for desc in bundle['sources']:
        if set(desc)!= {'path','sha256','bytes'} or type(desc['path'])is not str or any(p in ('','.', '..') for p in desc['path'].split('/')) or '\\' in desc['path'] or desc['path'].startswith('/'):raise Refusal('source-descriptor')
        path=config.source/desc['path']
        if not path.resolve().is_relative_to(config.source.resolve()):raise Refusal('source-containment')
        raw=read(path,config.maximum_input_bytes)
        if len(raw)!=desc['bytes'] or sha(raw)!=desc['sha256']:raise Refusal('source-pin')
        source_hashes.append((path,sha(raw)))
    legacy,original_sources=extract_legacy(config.source,config.maximum_input_bytes)
    source_hashes.extend((config.source/x['path'],x['sha256']) for x in original_sources)
    if [c['id'] for c in bundle['controls']]!=list(CONTROL_IDS):raise Refusal('control-inventory')
    if 463+len(bundle['paths'])+len(bundle['controls'])>config.maximum_cases:raise Refusal('case-count-limit')
    retained_bytes=8*len(bundle_raw)+4*len(inventory_raw)+8192
    if retained_bytes>config.maximum_receipt_bytes:raise Refusal('receipt-limit')
    records=[];selected={};statuses={}
    def run(identity,request,expected,scope,role,migrations=(),original=None):
        nonlocal retained_bytes
        retained_bytes+=8*(len(request)+len(expected)+len(original or expected))+4096
        if retained_bytes>config.maximum_receipt_bytes:raise Refusal('receipt-limit')
        if len(request)>LIMIT:raise Refusal('case-request-limit')
        response=document(expected)
        if validate_schema(scope,role,bytes(request),bytes(expected)) is not None:raise Refusal('schema-port-return')
        for repeat in range(2):
            if sha(read(config.binary,32*1024*1024))!=config.binary_sha256:raise Refusal('binary-drift')
            actual=execute(config.binary,request,'normal',config)
            if actual!=(0,expected,b''):raise Refusal('response-parity')
        statuses[identity]=response['status'];selected[identity]=set(response.get('logicalPlan',{}).get('requiredCapabilities',[]))
        records.append({'id':identity,'scope':scope,'role':role,'requestHex':request.hex(),'responseHex':expected.hex(),'originalExpectedHex':(original or expected).hex(),'migrations':list(migrations),'exit':0,'stderrHex':''})
    fence={'interfaceVersion':'weft-compile/0.4.0','status':'blocked','diagnostics':[{'code':'WFT-VERSION','severity':'error','message':'This entrypoint requires the exact 0.4 compile and dialect pair','phase':'input','recoverability':'correct-input'}]}
    namespace_expected=bytes.fromhex(bundle['legacyNamespaceRefusal'])
    if namespace_expected!=encoded(fence)+b'\n':raise Refusal('namespace-expectation')
    for case in legacy:
        # No historical migration or request version rewriting at this entrypoint.
        run('legacy:'+case['id'],encoded(case['request']),namespace_expected,'legacy','namespace',(),encoded(case['response'])+b'\n')
    for scope in ('paths','controls'):
        for case in bundle[scope]:
            if set(case)!= {'id','requestHex','responseHex','role'}:raise Refusal('case-shape')
            if case['role'] not in ('valid','invalid','namespace'):raise Refusal('case-role')
            run(scope+':'+case['id'],bytes.fromhex(case['requestHex']),bytes.fromhex(case['responseHex']),scope,case['role'])
    if len(statuses)!=len(records):raise Refusal('duplicate-case')
    if backend.get('backendId')!='ashlar.databricks.paths' or backend.get('backendVersion')!='0.4.0-paths-candidate' or backend.get('interfaceVersion')!='weft-backend/0.3.0' or backend.get('languageProfiles')!=[{'dialectProfile':'weft-sql/0.4.0','irVersion':'weft-ir/0.4.0'}]:raise Refusal('backend-profile')
    for i,identity in enumerate(CONTROL_IDS):
        if statuses.get('controls:'+identity)!=('blocked' if i<16 else 'compiled'):raise Refusal('control-status')
    capabilities={x['id'] for x in backend['capabilities']}
    if len(capabilities)!=config.declared_capability_count or len(backend['capabilities'])!=config.declared_capability_count or set(bundle['coverage'])!=capabilities:raise Refusal('capability-inventory')
    for capability,item in bundle['coverage'].items():
        if set(item)!= {'accepted','refused','scope'} or not item['scope'] or not(item['accepted'] or item['refused']):raise Refusal('capability-coverage')
        for identity in item['accepted']:
            if statuses.get(identity)!='compiled' or capability not in selected[identity]:raise Refusal('capability-not-selected')
        for identity in item['refused']:
            if statuses.get(identity)!='blocked':raise Refusal('refusal-not-observed')
    controls=[]
    for name,data,mode,code in [('oversize',b' '*(LIMIT+1),'normal',b'WEFT_CLI_INPUT_LIMIT\n'),('invalid-utf8',b'\xff','normal',b'WEFT_CLI_UTF8\n'),('split-utf8-at-limit',b' '*(LIMIT-1)+b'\xc3','normal',b'WEFT_CLI_UTF8\n'),('exact-limit',b' '*LIMIT,'normal',None),('exact-limit-multibyte',b' '*(LIMIT-2)+'é'.encode(),'normal',None),('malformed-json',b'{','normal',None),('directory-input',b'','directory-input',b'WEFT_CLI_INPUT_IO\n'),('closed-output',b'{}','closed-output',b'WEFT_CLI_OUTPUT_IO\n')]:
        actual=execute(config.binary,data,mode,config)
        if code is not None:
            if actual!=(2,b'',code):raise Refusal('transport-control')
        elif actual[0]!=0 or actual[2]!=b'' or document(actual[1]).get('status')!='blocked':raise Refusal('transport-control')
        retained_bytes+=8*(len(actual[1])+len(actual[2]))+4096
        if retained_bytes>config.maximum_receipt_bytes:raise Refusal('receipt-limit')
        controls.append({'id':name,'inputBytes':len(data),'inputSha256':sha(data),'exit':actual[0],'stdoutHex':actual[1].hex(),'stderrHex':actual[2].hex()})
    if verify_source(config)!=inventory_raw:raise Refusal('source-inventory-drift')
    for path,digest in source_hashes+[(Path(__file__).resolve(),sha(harness)),(config.schema_checker,config.schema_checker_sha256)]+[(config.cases,sha(bundle_raw)),(config.backend,sha(backend_raw)),(config.binary,config.binary_sha256)]:
        if sha(read(path,max(config.maximum_input_bytes,32*1024*1024)))!=digest:raise Refusal('closing-drift')
    result={'casesInput':{'sha256':sha(bundle_raw),'bytes':len(bundle_raw)},'backendInput':{'sha256':sha(backend_raw),'bytes':len(backend_raw)},'declaredCapabilityCount':config.declared_capability_count,'sourceCommit':config.source_commit,'binarySha256':config.binary_sha256,'harnessSha256':sha(harness),'schemaCheckerSha256':config.schema_checker_sha256,'sourceInventorySha256':sha(inventory_raw),'cases':records,'coverage':bundle['coverage'],'transport':controls,'sources':original_sources+bundle['sources'],'scope':'Exact compiler producer bytes/schema/case correspondence; refusal mapping is reviewed input, not native support or index authority'}
    payload=encoded(result)+b'\n'
    if len(payload)>config.maximum_receipt_bytes:raise Refusal('receipt-limit')
    config.output.mkdir(exist_ok=False)
    with (config.output/'receipt.json').open('xb') as stream:stream.write(payload)
    return result

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    for key in ('source','binary','cases','backend','output','schema-checker','source-inventory'):parser.add_argument('--'+key,type=Path,required=True)
    for key in ('source-commit','binary-sha256','schema-checker-sha256','source-inventory-sha256','cases-sha256','backend-sha256'):parser.add_argument('--'+key,required=True)
    for key in ('maximum-input-bytes','maximum-response-bytes','timeout-seconds','maximum-source-files','maximum-source-file-bytes','maximum-source-total-bytes','maximum-cases','maximum-receipt-bytes','declared-capability-count'):parser.add_argument('--'+key,type=int,required=True)
    a=parser.parse_args();config=Config(a.source,a.binary,a.cases,a.backend,a.output,a.source_commit,a.binary_sha256,a.maximum_input_bytes,a.maximum_response_bytes,a.timeout_seconds,a.source_inventory,a.source_inventory_sha256,a.schema_checker,a.schema_checker_sha256,a.maximum_source_files,a.maximum_source_file_bytes,a.maximum_source_total_bytes,a.maximum_cases,a.maximum_receipt_bytes,a.cases_sha256,a.backend_sha256,a.declared_capability_count)
    # Explicit trusted schema program receives a bounded byte envelope; no retrieval.
    def schema(scope,role,request,response):
        body=encoded({'scope':scope,'role':role,'requestHex':request.hex(),'responseHex':response.hex()})
        if sha(read(config.schema_checker,32*1024*1024))!=config.schema_checker_sha256:raise Refusal('schema-checker-drift')
        code,out,err=transport(config.schema_checker,body,'normal',config)
        if sha(read(config.schema_checker,32*1024*1024))!=config.schema_checker_sha256:raise Refusal('schema-checker-drift')
        if (code,out,err)!=(0,b'',b''):raise Refusal('schema-refusal')
    qualify(config,execute=transport,validate_schema=schema)
if __name__=='__main__':main()
