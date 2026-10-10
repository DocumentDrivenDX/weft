"""Inert assembly for the separately qualified Paths530 realization.

Original native c6 evidence is not transferred. This producer never executes a
compiler, installs a component, grants index trust, or invokes a native engine.
"""
import argparse
from dataclasses import dataclass
import gzip
import hashlib
import types
import json
import os
from pathlib import Path
import shutil
import struct
import stat
import sys
import tempfile
import zlib

SOURCE = '530ae3511a4a50364d3d7e26195d3883952601df'
BINARY = '71ba88a07c27661413830d5a42e2ea5c4a8b071df965bb98dbbe8f3615a6c911'
BACKEND = 'fc36ddbb6ca41309d25dadf1969bb27efb72debe0c6669b76a8a6a425ca35ec5'
RECEIPT = '7f80a87ac41d868a32d220fd5a5ec7f8998c264ec0eecc4455029ce3ba7caf68'
CASES = '1cf69c0dfbde7cb8f82e6653c67637157342374bd1ab4114050d93246f21000d'
INVENTORY = '2a1eed46946a4d623f2f8abc7e7592e6d74e3964585ce1c500600a7ea62d6896'
BUILD = 'd7bec02e48c77842db160d3c34081a432e9a00b867d8547df11dd0de0dc45fa0'
FILE_LIMIT = 32 * 1024 * 1024
DECODED_LIMIT = 64 * 1024 * 1024
TOTAL_LIMIT = 96 * 1024 * 1024
CONTROL_IDS = ('candidate-opt-out','unknown-backend','wrong-backend-version','wrong-profile',
 'unknown-envelope-member','mismatched-interface-dialect','binding-digest-mismatch',
 'model-digest-mismatch','sql-byte-limit','binding-byte-limit','duplicate-envelope-member',
 'missing-publication','negative-table-version','duplicate-table-uuid','missing-table-mapping',
 'inconsistent-model-pin','fresh-binding-0','fresh-binding-1','fresh-binding-2')
TRANSPORT_IDS = ('oversize','invalid-utf8','split-utf8-at-limit','exact-limit',
 'exact-limit-multibyte','malformed-json','directory-input','closed-output')
FENCE = {'interfaceVersion':'weft-compile/0.4.0','status':'blocked','diagnostics':[{
 'code':'WFT-VERSION','severity':'error','message':'This entrypoint requires the exact 0.4 compile and dialect pair',
 'phase':'input','recoverability':'correct-input'}]}


def sha(raw): return hashlib.sha256(raw).hexdigest()
def encoded(value): return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
def relative(name):
    if type(name)is not str or not name or ':' in name or '\\' in name or name.startswith('/') or any(p in ('','.', '..')for p in name.split('/')):raise ValueError('relative-path')
    return name

def read(path, limit=FILE_LIMIT):
    path=Path(path)
    if any(p.is_symlink()for p in (path,*path.parents)):raise ValueError('regular-contained-input')
    fd=None;primary=None;raw=None
    try:
        fd=os.open(path,os.O_RDONLY|os.O_NONBLOCK|os.O_NOFOLLOW)
        if not stat.S_ISREG(os.fstat(fd).st_mode):raise ValueError('regular-contained-input')
        chunks=[];remaining=limit+1
        while remaining:
            chunk=os.read(fd,min(remaining,65536))
            if not chunk:break
            chunks.append(chunk);remaining-=len(chunk)
        raw=b''.join(chunks)
        if len(raw)>limit:raise ValueError('input-bound')
    except BaseException as exc:primary=exc
    finally:
        if fd is not None:
            try:os.close(fd)
            except BaseException as exc:
                if primary is None:primary=exc
                else:
                    try:setattr(primary,'cleanup_failed',True)
                    except BaseException:pass
    if primary is not None:raise primary
    return raw

def document(raw):
    def pairs(items):
        result={}
        for key,value in items:
            if key in result:raise ValueError('duplicate-json')
            result[key]=value
        return result
    def integer(token):
        if len(token.lstrip('-'))>20:raise ValueError('integer-bound')
        return int(token)
    def forbidden(_):raise ValueError('numeric-shape')
    return json.loads(raw.decode('utf8'),object_pairs_hook=pairs,parse_int=integer,parse_float=forbidden,parse_constant=forbidden)

def inflate(raw, limit=DECODED_LIMIT):
    decoder=zlib.decompressobj(16+zlib.MAX_WBITS);decoded=decoder.decompress(raw,limit+1)
    if len(decoded)>limit or decoder.unconsumed_tail or not decoder.eof or decoder.unused_data:raise ValueError('gzip-bound-or-member')
    return decoded

def descriptor(path,raw):return {'path':relative(path),'sha256':sha(raw),'bytes':len(raw)}

class Snapshot:
    def __init__(self):self.artifacts={};self.originals={};self.total=0
    def generated(self,path,raw):
        relative(path)
        if path in self.artifacts:raise ValueError('duplicate-artifact')
        if len(raw)>FILE_LIMIT or self.total+len(raw)>TOTAL_LIMIT:raise ValueError('package-bound')
        self.artifacts[path]=raw;self.total+=len(raw);return descriptor(path,raw)
    def take(self,source,destination,limit=FILE_LIMIT):
        source=Path(source)
        if source.stat().st_size>limit or self.total+source.stat().st_size>TOTAL_LIMIT:raise ValueError('package-bound')
        raw=read(source,limit);self.originals[source]=(sha(raw),len(raw))
        return self.generated(destination,raw)
    def compressed(self,source,destination):
        source=Path(source);raw=read(source,DECODED_LIMIT);self.originals[source]=(sha(raw),len(raw))
        self.generated(destination,gzip.compress(raw,mtime=0));return raw
    def close(self):
        for p,(digest,size)in self.originals.items():
            raw=read(p,size)
            if len(raw)!=size or sha(raw)!=digest:raise ValueError('closing-input-drift')

@dataclass(frozen=True)
class Config:
    source_root: Path
    build_root: Path
    corpus_root: Path
    output: Path
    realization_id: str
    profile: str = 'paths'
    def __post_init__(self):
        if type(self.profile)is not str or self.profile not in ('paths','paths-keys'):raise ValueError('assembly-profile')
        if any(not isinstance(p,Path)or not p.is_absolute()for p in (self.source_root,self.build_root,self.corpus_root,self.output)):raise ValueError('absolute-config')
        if type(self.realization_id)is not str or not self.realization_id or self.realization_id[0] not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789' or len(self.realization_id)>128 or any(c not in 'abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-._'for c in self.realization_id):raise ValueError('realization-id')


def platform(binary,observed_os):
    if len(binary)<32:raise ValueError('mach-o')
    header=struct.unpack_from('<8I',binary)
    if header[0]!=0xfeedfacf or header[1]!=0x0100000c or header[3]!=2 or not 0<header[4]<=512 or header[5]>1024*1024:raise ValueError('mach-o-arm64')
    offset=32;found=[]
    for _ in range(header[4]):
        if offset+8>32+header[5] or offset+8>len(binary):raise ValueError('load-command')
        command,size=struct.unpack_from('<2I',binary,offset)
        if size<8 or offset+size>32+header[5] or offset+size>len(binary):raise ValueError('load-command')
        if command==0x32:
            if size<24:raise ValueError('build-version')
            p,minimum,sdk=struct.unpack_from('<3I',binary,offset+8)
            if p!=1:raise ValueError('platform')
            def version(n):return f'{n>>16}.{(n>>8)&255}'+(f'.{n&255}'if n&255 else '')
            found.append((version(minimum),version(sdk)))
        offset+=size
    if offset!=32+header[5] or found!=[('11.0','27.0')]or observed_os!='27.0.1':raise ValueError('qualified-platform')
    return {'binaryFormat':'mach-o','machine':'arm64','minimumOS':'11.0','sdk':'27.0','observedOS':observed_os}


def verify_records(receipt,bundle,backend):
    if receipt['sourceCommit']!=SOURCE or receipt['binarySha256']!=BINARY or receipt['declaredCapabilityCount']!=47 or receipt['sourceInventorySha256']!=INVENTORY:raise ValueError('receipt-profile')
    if set(bundle)!= {'sourceCommit','paths','controls','coverage','sources','legacyNamespaceRefusal'}or bundle['sourceCommit']!=SOURCE:raise ValueError('bundle-shape')
    if bytes.fromhex(bundle['legacyNamespaceRefusal'])!=encoded(FENCE)+b'\n':raise ValueError('namespace-fence')
    if len(bundle['paths'])!=30 or [c['id']for c in bundle['controls']]!=list(CONTROL_IDS):raise ValueError('case-inventory')
    cases=receipt['cases']
    if len(cases)!=512 or len({r['id']for r in cases})!=512:raise ValueError('complete-cases')
    expected=[('paths:'+c['id'],'paths',c)for c in bundle['paths']]+[('controls:'+c['id'],'controls',c)for c in bundle['controls']]
    for row,(identity,scope,case)in zip(cases[463:],expected):
        if row['id']!=identity or row['scope']!=scope or row['role']!=case['role']or row['requestHex']!=case['requestHex']or row['responseHex']!=case['responseHex']:raise ValueError('case-byte-correspondence')
    for row in cases:
        if row['exit']!=0 or row['stderrHex']or row['migrations']:raise ValueError('transport-custody')
        response=document(bytes.fromhex(row['responseHex']))
        if response['interfaceVersion']!='weft-compile/0.4.0' or response['status']not in ('compiled','blocked'):raise ValueError('response-version')
        if row['scope']=='legacy'and(row['role']!='namespace'or bytes.fromhex(row['responseHex'])!=encoded(FENCE)+b'\n'):raise ValueError('legacy-namespace')
    counts={}
    for scope in ('legacy','paths','controls'):
        rows=[r for r in cases if r['scope']==scope];counts[scope]=(len(rows),sum(document(bytes.fromhex(r['responseHex']))['status']=='compiled'for r in rows))
    if counts!={'legacy':(463,0),'paths':(30,26),'controls':(19,3)}:raise ValueError('counts')
    if backend['backendId']!='ashlar.databricks.paths'or backend['backendVersion']!='0.4.0-paths-candidate'or backend['interfaceVersion']!='weft-backend/0.3.0'or backend['languageProfiles']!=[{'dialectProfile':'weft-sql/0.4.0','irVersion':'weft-ir/0.4.0'}]:raise ValueError('backend-profile')
    caps={c['id']for c in backend['capabilities']}
    if len(caps)!=47 or len(backend['capabilities'])!=47 or receipt['coverage']!=bundle['coverage']or set(bundle['coverage'])!=caps:raise ValueError('coverage')
    by_id={r['id']:document(bytes.fromhex(r['responseHex']))for r in cases}
    for capability,item in bundle['coverage'].items():
        if set(item)!= {'accepted','refused','scope'}or not item['scope']or not(item['accepted']or item['refused']):raise ValueError('coverage-item')
        for identity in item['accepted']:
            response=by_id[identity]
            if response['status']!='compiled'or capability not in response['logicalPlan']['requiredCapabilities']:raise ValueError('selection')
        for identity in item['refused']:
            if by_id[identity]['status']!='blocked':raise ValueError('refusal')
    if [r['id']for r in receipt['transport']]!=list(TRANSPORT_IDS):raise ValueError('transport-inventory')
    codes={'oversize':b'WEFT_CLI_INPUT_LIMIT\n','invalid-utf8':b'WEFT_CLI_UTF8\n','split-utf8-at-limit':b'WEFT_CLI_UTF8\n','directory-input':b'WEFT_CLI_INPUT_IO\n','closed-output':b'WEFT_CLI_OUTPUT_IO\n'}
    for row in receipt['transport']:
        if row['id']in codes:
            if row['exit']!=2 or row['stdoutHex']or bytes.fromhex(row['stderrHex'])!=codes[row['id']]:raise ValueError('transport-refusal')
        elif row['exit']!=0 or row['stderrHex']or document(bytes.fromhex(row['stdoutHex']))['status']!='blocked':raise ValueError('transport-refusal')
    return cases


def verify_legacy(source_root, harness_path, cases, expected_harness_sha):
    """Reuse only the byte-pinned producer's pure historical source extractor."""
    snapshot=read(harness_path)
    if sha(snapshot)!=expected_harness_sha:raise ValueError('extractor-pin')
    name='_paths530_pinned_source_extractor'
    module=types.ModuleType(name);module.__file__=str(harness_path);sys.modules[name]=module
    try:
        exec(compile(snapshot,str(harness_path),'exec'),module.__dict__)
        originals,_=module.extract_legacy(source_root,FILE_LIMIT)
        for row,original in zip(cases[:463],originals):
            if row['id']!='legacy:'+original['id']or bytes.fromhex(row['requestHex'])!=module.encoded(original['request'])or bytes.fromhex(row['originalExpectedHex'])!=module.encoded(original['response'])+b'\n':raise ValueError('original-legacy-bytes')
    finally:sys.modules.pop(name,None)


def assemble(config):
    if config.profile=='paths-keys':
        import paths_keys_distribution
        return paths_keys_distribution.assemble_keys(config,sys.modules[__name__])
    if config.output.exists()or config.output.is_symlink()or any(p.is_symlink()for p in config.output.parents):raise ValueError('fresh-contained-output')
    s=Snapshot()
    inventory_raw=s.compressed(config.build_root/'source-inventory.json','evidence/source-inventory.json.gz')
    if sha(inventory_raw)!=INVENTORY:raise ValueError('source-inventory-pin')
    inventory=document(inventory_raw)
    if set(inventory)!= {'sourceCommit','files'}or inventory['sourceCommit']!=SOURCE or len(inventory['files'])!=1842:raise ValueError('complete-source-inventory')
    entries=inventory['files'];names=[relative(d['path'])for d in entries]
    if names!=sorted(set(names)):raise ValueError('source-order')
    indexed=dict(zip(names,entries))
    def source(name):
        raw=read(config.source_root/relative(name));entry=indexed[name]
        if sha(raw)!=entry['sha256']or len(raw)!=entry['bytes']or hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()!=entry['gitBlob']:raise ValueError('source-entry')
        if bool((config.source_root/name).stat().st_mode&0o111)!=(entry['mode']=='100755'):raise ValueError('source-mode')
        if 'source-subset/'+name not in s.artifacts:
            s.originals[config.source_root/name]=(sha(raw),len(raw));s.generated('source-subset/'+name,raw)
        return raw
    s.take(config.build_root/'weft-paths','bin/weft-paths');binary=s.artifacts['bin/weft-paths']
    if sha(binary)!=BINARY or len(binary)!=8273840:raise ValueError('binary-pin')
    s.take(config.build_root/'backend-manifest.json','backend/backend-manifest.json');backend_raw=s.artifacts['backend/backend-manifest.json']
    if sha(backend_raw)!=BACKEND:raise ValueError('backend-pin')
    receipt_raw=s.compressed(config.corpus_root/'producer-output/receipt.json','evidence/full-receipt.json.gz')
    bundle_desc=s.take(config.corpus_root/'cases.json','evidence/expected-cases.json');bundle_raw=s.artifacts[bundle_desc['path']]
    if sha(receipt_raw)!=RECEIPT or sha(bundle_raw)!=CASES:raise ValueError('exact-qualified-inputs')
    receipt=document(receipt_raw);bundle=document(bundle_raw);cases=verify_records(receipt,bundle,document(backend_raw))
    if receipt['casesInput']!= {'sha256':CASES,'bytes':len(bundle_raw)}or receipt['backendInput']!= {'sha256':BACKEND,'bytes':len(backend_raw)}:raise ValueError('receipt-inputs')
    for desc in receipt['sources']:
        raw=source(desc['path'])
        if len(raw)!=desc['bytes']or sha(raw)!=desc['sha256']:raise ValueError('corpus-source')
    harness=source('scripts/distribution/check-paths-cli.py')
    if sha(harness)!=receipt['harnessSha256']:raise ValueError('producer-pin')
    verify_legacy(config.source_root,config.source_root/'scripts/distribution/check-paths-cli.py',cases,receipt['harnessSha256'])
    s.take(config.build_root/'command-c.json','evidence/build-command.json',4*1024*1024)
    command_raw=s.artifacts['evidence/build-command.json'];command=document(command_raw)
    if sha(command_raw)!=BUILD or command['sourceCommit']!=SOURCE:raise ValueError('build-pin')
    build=next(c for c in command['commands']if c['phase']=='cli-build')
    expected=['build','--offline','--locked','--release','-j1','--target','aarch64-apple-darwin','-p','weft-runtime','--no-default-features','--features','ashlar-databricks-paths','--bin','weft-paths']
    if build['argv'][1:]!=expected or command['tools']['rustc']!='rustc 1.90.0 (1159e78c4 2025-09-14)':raise ValueError('build-composition')
    for phase,name in [('cli-build','cli-build-outcome.json'),('source-metadata','metadata-build-outcome.json')]:
        s.take(config.build_root/name,'evidence/'+name);outcome=document(s.artifacts['evidence/'+name])
        if outcome['commandSha256']!=BUILD or outcome['phase']!=phase or outcome['exitCode']!=0 or outcome['openingClosingCustody']is not True or outcome['environmentInherited']is not False:raise ValueError('build-outcome')
        if phase=='cli-build'and outcome['binary']['sha256']!=BINARY:raise ValueError('built-binary')
    public_schemas=[]
    for name in names:
        if name.startswith('docs/helix/02-design/contracts/')and name.endswith('.schema.json'):
            source(name);public_schemas.append(descriptor('source-subset/'+name,s.artifacts['source-subset/'+name]))
    for name in ['Cargo.lock','rust-toolchain.toml','spec/upstream/umf-0.7.0.schema.json','spec/upstream/umf-0.8.0.schema.json','scripts/distribution/backend-paths-metadata.rs']:source(name)
    for kind,rows in [('corpus',cases[:493]),('controls',cases[493:])]:
        raw=b''.join(encoded(r)+b'\n'for r in rows)
        if len(raw)>DECODED_LIMIT:raise ValueError('decoded-corpus-bound')
        s.generated('evidence/'+kind+'.jsonl.gz',gzip.compress(raw,mtime=0))
        compiled=26 if kind=='corpus'else 3;summary={'profile':'weft-paths530-produced-corpus/0.1','cases':len(rows),'compiled':compiled,'blocked':len(rows)-compiled,'binarySha256':BINARY,'sourceCommit':SOURCE,'decodedSha256':sha(raw),'decodedBytes':len(raw),'qualification':'Exact compiler/schema/accepted-case correspondence; no native semantics or authority discharge.'}
        s.generated('evidence/'+kind+'-summary.json',encoded(summary)+b'\n')
        s.generated('evidence/'+kind+'-custody.json',encoded({'sourceCommit':SOURCE,'binarySha256':BINARY,'fullReceiptSha256':RECEIPT,'expectedCasesSha256':CASES,'decodedSha256':sha(raw),'decodedBytes':len(raw)})+b'\n')
    s.generated('evidence/transport.json',encoded({'binarySha256':BINARY,'controls':receipt['transport']})+b'\n')
    s.generated('evidence/tools.json',encoded(command['tools'])+b'\n')
    platform_value=platform(binary,command['platformObservation']['observedOS'])
    env=command['environment'];observed={'CARGO_HOME':env['CARGO_HOME'],'RUSTUP_HOME':env['RUSTUP_HOME'],'CARGO_TARGET_DIR':env['CARGO_TARGET_DIR'],'PATHPrefix':env['PATH']}
    record={'format':'weft-distribution/0.1','realizationId':config.realization_id,'source':{'commit':SOURCE,'inventory':{'artifact':descriptor('evidence/source-inventory.json.gz',s.artifacts['evidence/source-inventory.json.gz']),'decodedSha256':INVENTORY,'decodedBytes':len(inventory_raw),'trackedFiles':1842}},'build':{'release':True,'target':'aarch64-apple-darwin','features':['ashlar-databricks-paths'],'command':build['argv'],'tools':descriptor('evidence/tools.json',s.artifacts['evidence/tools.json']),'lockfiles':[descriptor('source-subset/Cargo.lock',s.artifacts['source-subset/Cargo.lock'])],'toolchain':descriptor('source-subset/rust-toolchain.toml',s.artifacts['source-subset/rust-toolchain.toml']),'effectiveEnvironment':{'observed':observed,'unknowns':command['uncertainties']},'platform':platform_value},'executable':descriptor('bin/weft-paths',binary),'backendManifests':[descriptor('backend/backend-manifest.json',backend_raw)],'publicSchemas':public_schemas,'conformance':{'corpus':{'cases':493,'compiled':26,'blocked':467,'responses':descriptor('evidence/corpus.jsonl.gz',s.artifacts['evidence/corpus.jsonl.gz']),'summary':descriptor('evidence/corpus-summary.json',s.artifacts['evidence/corpus-summary.json']),'custody':descriptor('evidence/corpus-custody.json',s.artifacts['evidence/corpus-custody.json'])},'controls':{'cases':19,'responses':descriptor('evidence/controls.jsonl.gz',s.artifacts['evidence/controls.jsonl.gz']),'summary':descriptor('evidence/controls-summary.json',s.artifacts['evidence/controls-summary.json']),'custody':descriptor('evidence/controls-custody.json',s.artifacts['evidence/controls-custody.json'])},'transport':descriptor('evidence/transport.json',s.artifacts['evidence/transport.json'])}}
    s.take(Path(__file__).resolve(),'producer/assemble-paths-distribution.py')
    s.generated('manifest.json',encoded(record)+b'\n')
    proof={'format':'weft-distribution-assembly-custody/0.1','sourceCommit':SOURCE,'qualification':'Inert Paths530 compiler-qualified candidate only. No self-registration, installation, native query support, source/ACK authority or hermetic rebuild claim. Full public530 checkout is a separate rebuild prerequisite. Historical native c6 evidence remains separate.','producerSha256':sha(s.artifacts['producer/assemble-paths-distribution.py']),'artifacts':[descriptor(name,raw)for name,raw in sorted(s.artifacts.items())],'sourceSubset':sorted(name[len('source-subset/'):]for name in s.artifacts if name.startswith('source-subset/'))}
    s.generated('assembly-custody.json',encoded(proof)+b'\n');publish(s,config.output);return record


def write_exclusive(path,raw):
    """Owned stream: every BaseException primary survives close failure."""
    stream=None;primary=None
    try:
        stream=path.open('xb');stream.write(raw);stream.flush();os.fsync(stream.fileno())
    except BaseException as error:primary=error
    finally:
        if stream is not None:
            try:stream.close()
            except BaseException as error:
                if primary is None:primary=error
                else:
                    try:primary.cleanup_failed=True
                    except BaseException:pass
    if primary is not None:raise primary


def publish(snapshot,output,*,executable='bin/weft-paths'):
    if type(executable)is not str or executable not in ('bin/weft-paths','bin/weft-paths-keys'):raise ValueError('closed-executable-selection')
    snapshot.close();parent=output.parent.resolve(strict=True);temporary=Path(tempfile.mkdtemp(prefix='.paths-distribution-',dir=parent))
    primary=None
    try:
        for name,raw in snapshot.artifacts.items():
            p=temporary/name;p.parent.mkdir(parents=True,exist_ok=True)
            write_exclusive(p,raw)
            p.chmod(0o555 if name==executable else 0o444)
            if read(p)!=raw:raise ValueError('copy-drift')
        snapshot.close()
        if output.exists()or output.is_symlink():raise ValueError('output-appeared')
        # Explicit cooperating single-writer parent, not cross-principal fencing.
        os.rename(temporary,output)
    except BaseException as exc:primary=exc
    finally:
        try:
            if temporary.exists():shutil.rmtree(temporary)
        except BaseException as exc:
            if primary is None:primary=exc
            else:
                try:setattr(primary,'cleanup_failed',True)
                except BaseException:pass
    if primary is not None:raise primary


def main():
    p=argparse.ArgumentParser()
    for name in ['source-root','build-root','corpus-root','output']:p.add_argument('--'+name,type=Path,required=True)
    p.add_argument('--realization-id',required=True);p.add_argument('--profile',choices=('paths','paths-keys'),default='paths');a=p.parse_args()
    record=assemble(Config(a.source_root,a.build_root,a.corpus_root,a.output,a.realization_id,a.profile))
    print(json.dumps({'state':'inert-candidate-assembled','realizationId':record['realizationId']}))
if __name__=='__main__':main()
