"""Fixed PathsKeys3a proof/assembly profile; no execution or index authority.

Compiler source3a, qualified producer H and retained historical530 evidence have
separate byte identities. Shared I/O, platform and publication mechanics belong
to assemble-paths-distribution; no caller-selected pin/count policy exists here.
"""
import gzip
import hashlib
from pathlib import Path

SOURCE='3a2a79ccc19d636f20d31116662d61365c1b4ff7'
BINARY='471fc5dedc8f8eba3156444d19cc2161fac7a06833672dd5621ed49ead8b522c'
BACKEND='ead2df8de9927765775ec7267a9a1865b1c02ec21c849255ada85c36a645fb84'
INVENTORY='e9f1e0ac01d7b9dc1b2d61148c74274c1fed5a5593726487aa4562f2e4bf3512'
RECEIPT='d5f586b09499d8a099d4b270ab49f03e797dbd9e9ca38ec1bc8034aa0dbf5aa0'
CASES='b9661c7ff024e9696417ef5bce3ca70a28d8cd8b93b109835ab0517ededf63d0'
BUILD='1ad88380c1c0ece419c7054c6a8d474aaaa425fdef1027798848df1f9e17ac90'
QUALIFICATION='5be699a29f0fc67d95c9e99222c7981d86b074d615b8344983bb6bd5dba4c0b2'
HARNESS='77517097fe5c5c3b27f90433c505c36514e409896beaa3d473f1937542eb75a5'
BRIDGE='fccf88deb54e04bc1c599a8dd11303669a15e8e0c71e3ae0d2c5f649c6ee68b8'
CHECKER='bf9523f086466fd485d1cb90b54da9d02677585f015216ad90b236daa9acc276'
BACKEND_ID='ashlar.databricks.paths-keys'
VERSION='0.4.0-paths-keys-candidate'
TARGET='spark4-delta4-paths-keys-candidate'
FEATURE='ashlar-databricks-paths-keys'
BINARY_NAME='weft-paths-keys'
NAMESPACE_IDS=('0.1-pair','0.2-pair','0.3-pair','reverse-mixed','unknown-pair')


def verify_records(receipt,bundle,backend,codec):
    """Local byte/inventory correspondence; independent semantics remain review-owned."""
    if receipt['sourceCommit']!=SOURCE or receipt['binarySha256']!=BINARY or receipt['sourceInventorySha256']!=INVENTORY or receipt['declaredCapabilityCount']!=48 or receipt['profile']!='paths-keys':raise ValueError('keys-receipt-profile')
    if set(bundle)!= {'format','sourceCommit','paths','controls','namespaceFences','coverage','sources','historicalQualification'} or bundle['format']!='weft-paths-keys-corpus/0.1' or bundle['sourceCommit']!=SOURCE:raise ValueError('keys-bundle')
    if len(bundle['paths'])!=50 or [c['id']for c in bundle['controls']]!=list(codec.CONTROL_IDS) or [c['id']for c in bundle['namespaceFences']]!=list(NAMESPACE_IDS):raise ValueError('keys-cases')
    if backend['backendId']!=BACKEND_ID or backend['backendVersion']!=VERSION or backend['interfaceVersion']!='weft-backend/0.3.0' or backend['languageProfiles']!=[{'dialectProfile':'weft-sql/0.4.0','irVersion':'weft-ir/0.4.0'}] or len(backend['targetProfiles'])!=1 or backend['targetProfiles'][0]['id']!=TARGET:raise ValueError('keys-backend')
    rows=receipt['cases']
    expected=[('namespace:'+c['id'],'legacy' if i<3 else 'controls',c)for i,c in enumerate(bundle['namespaceFences'])]+[('paths:'+c['id'],'paths',c)for c in bundle['paths']]+[('controls:'+c['id'],'controls',c)for c in bundle['controls']]
    if len(rows)!=74 or len({r['id']for r in rows})!=74 or receipt['executedProtocolCases']!=74:raise ValueError('keys-cases')
    statuses=[];responses={}
    for row,(identity,scope,case)in zip(rows,expected):
        if (row['id'],row['scope'],row['role'],row['requestHex'],row['responseHex'])!=(identity,scope,case['role'],case['requestHex'],case['responseHex']) or type(row['exit'])is not int or row['exit']!=0 or row['stderrHex'] or row['migrations']:raise ValueError('keys-original-case')
        request=bytes.fromhex(row['requestHex']);raw=bytes.fromhex(row['responseHex'])
        if len(request)>16*1024*1024 or len(raw)>4*1024*1024:raise ValueError('keys-protocol-bound')
        response=codec.document(raw);status=response['status'];statuses.append(status);responses[identity]=response
        if response['interfaceVersion']!='weft-compile/0.4.0' or status not in ('compiled','blocked'):raise ValueError('keys-response')
        if identity.startswith('namespace:') and response!=codec.FENCE:raise ValueError('keys-namespace')
        if status=='compiled' and (response['backend']!=dict(backendId=BACKEND_ID,backendVersion=VERSION,interfaceVersion='weft-backend/0.3.0',targetProfile=TARGET) or response['targetContext']['id']!=TARGET):raise ValueError('keys-compiled-profile')
    if statuses[:5]!=['blocked']*5 or statuses[5:55].count('compiled')!=36 or statuses[55:].count('compiled')!=3 or statuses.count('compiled')!=39 or statuses.count('blocked')!=35 or receipt['protocolStatusCounts']!={'compiled':39,'blocked':35}:raise ValueError('keys-statuses')
    caps=[c['id']for c in backend['capabilities']]
    if len(caps)!=48 or len(set(caps))!=48 or 'relationship.boundedKeys'not in caps or set(bundle['coverage'])!=set(caps) or receipt['coverage']!=bundle['coverage']:raise ValueError('keys-coverage')
    for cap,item in bundle['coverage'].items():
        if set(item)!= {'accepted','refused','scope'} or not item['scope'] or not(item['accepted']or item['refused']):raise ValueError('keys-coverage')
        for identity in item['accepted']:
            if responses[identity]['status']!='compiled' or cap not in responses[identity]['logicalPlan']['requiredCapabilities']:raise ValueError('keys-coverage')
        for identity in item['refused']:
            if responses[identity]['status']!='blocked':raise ValueError('keys-coverage')
    if [r['id']for r in receipt['transport']]!=list(codec.TRANSPORT_IDS):raise ValueError('keys-transport')
    for row in receipt['transport']:
        raw=bytes.fromhex(row['stdoutHex']);err=bytes.fromhex(row['stderrHex'])
        if len(raw)>4*1024*1024 or len(err)>4096 or row['exit']not in (0,2):raise ValueError('keys-transport')
        if row['id']in ('oversize','invalid-utf8','split-utf8-at-limit','directory-input','closed-output'):
            marker='INPUT_LIMIT' if row['id']=='oversize'else 'UTF8' if 'utf8'in row['id']else 'INPUT_IO' if row['id']=='directory-input'else 'OUTPUT_IO'
            if row['exit']!=2 or raw or err!=('WEFT_CLI_'+marker+'\n').encode():raise ValueError('keys-transport')
        elif row['exit']!=0 or err or codec.document(raw)['status']!='blocked':raise ValueError('keys-transport')
    if receipt['historicalQualification']!=bundle['historicalQualification'] or receipt['harnessSha256']!=HARNESS or receipt['schemaCheckerSha256']!=BRIDGE or receipt['producerProvenance']['harnessSha256']!=HARNESS:raise ValueError('keys-producer')
    return rows


def assemble_keys(config,codec):
    """Assemble inert selected bytes using the owning bounded I/O mechanics."""
    if config.output.exists()or config.output.is_symlink()or any(p.is_symlink()for p in config.output.parents):raise ValueError('fresh-contained-output')
    s=codec.Snapshot();inventory_raw=s.compressed(config.build_root/'source-inventory.json','evidence/source-inventory.json.gz')
    if codec.sha(inventory_raw)!=INVENTORY:raise ValueError('keys-inventory')
    inventory=codec.document(inventory_raw)
    if set(inventory)!= {'sourceCommit','files'}or inventory['sourceCommit']!=SOURCE or len(inventory['files'])!=1987:raise ValueError('keys-inventory')
    entries=inventory['files'];names=[codec.relative(d['path'])for d in entries]
    if names!=sorted(set(names)):raise ValueError('keys-source-order')
    indexed=dict(zip(names,entries))
    def source(name):
        name=codec.relative(name);raw=codec.read(config.source_root/name);entry=indexed[name]
        if codec.sha(raw)!=entry['sha256']or len(raw)!=entry['bytes']or hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()!=entry['gitBlob']or bool((config.source_root/name).stat().st_mode&0o111)!=(entry['mode']=='100755'):raise ValueError('keys-source-entry')
        if 'source-subset/'+name not in s.artifacts:s.originals[config.source_root/name]=(codec.sha(raw),len(raw));s.generated('source-subset/'+name,raw)
        return raw
    s.take(config.build_root/BINARY_NAME,'bin/'+BINARY_NAME)
    if codec.sha(s.artifacts['bin/'+BINARY_NAME])!=BINARY or len(s.artifacts['bin/'+BINARY_NAME])!=8330656:raise ValueError('keys-binary')
    s.take(config.build_root/'backend-manifest.json','backend/backend-manifest.json');backend_raw=s.artifacts['backend/backend-manifest.json']
    if codec.sha(backend_raw)!=BACKEND:raise ValueError('keys-backend-pin')
    receipt_raw=s.compressed(config.corpus_root/'producer-output/receipt.json','evidence/full-receipt.json.gz')
    s.take(config.corpus_root/'command.json','evidence/qualification-command.json',4*1024*1024)
    command=codec.document(s.artifacts['evidence/qualification-command.json'])
    if codec.sha(receipt_raw)!=RECEIPT or codec.sha(s.artifacts['evidence/qualification-command.json'])!=QUALIFICATION:raise ValueError('keys-qualification')
    expected_path=Path(command['commands'][0]['argv'][command['commands'][0]['argv'].index('--cases')+1])
    s.take(expected_path,'evidence/expected-cases.json');bundle_raw=s.artifacts['evidence/expected-cases.json']
    if codec.sha(bundle_raw)!=CASES:raise ValueError('keys-cases-pin')
    receipt=codec.document(receipt_raw);bundle=codec.document(bundle_raw);rows=verify_records(receipt,bundle,codec.document(backend_raw),codec)
    if receipt['casesInput']!={'sha256':CASES,'bytes':len(bundle_raw)}or receipt['backendInput']!={'sha256':BACKEND,'bytes':len(backend_raw)}:raise ValueError('keys-inputs')
    for field,name,pin in [('producerProvenance','producer.py',HARNESS)]:
        p=Path(command[field]['snapshot']['path']);s.take(p,'producer/corpus-harness.py')
        if codec.sha(s.artifacts['producer/corpus-harness.py'])!=pin:raise ValueError('keys-producer-pin')
    for name,pin in [('schema-checker',BRIDGE),('checker.py',CHECKER),('run.py',None),('bounded-launcher.py',None)]:
        p=config.corpus_root/name;s.take(p,'producer/'+name)
        d=next(d for d in command['resources']if d['path']==str(p))
        if codec.sha(s.artifacts['producer/'+name])!=d['sha256']or len(s.artifacts['producer/'+name])!=d['bytes']or pin is not None and codec.sha(s.artifacts['producer/'+name])!=pin:raise ValueError('keys-producer-resource')
    s.take(config.corpus_root/'outcome.json','evidence/qualification-outcome.json');outcome=codec.document(s.artifacts['evidence/qualification-outcome.json'])
    if outcome['commandSha256']!=QUALIFICATION or outcome['completed']is not True or outcome['openingClosingCustody']is not True or outcome['result']['exitCode']!=0:raise ValueError('keys-qualification-outcome')
    for desc in receipt['sources']:
        raw=source(desc['path'])
        if codec.sha(raw)!=desc['sha256']or len(raw)!=desc['bytes']:raise ValueError('keys-corpus-source')
    history=bundle['historicalQualification'];base=Path(history['manifest']['path']).parent.as_posix();hm=codec.document(source(history['manifest']['path']))
    inventory_name=base+'/'+hm['source']['inventory']['artifact']['path'];hi_raw=codec.inflate(source(inventory_name));hi=codec.document(hi_raw)
    if codec.sha(hi_raw)!=hm['source']['inventory']['decodedSha256']or hi['sourceCommit']!=history['sourceCommit']:raise ValueError('keys-historical-inventory')
    hr=codec.document(codec.inflate(source(history['receipt']['path'])))
    if hr['sourceCommit']!=history['sourceCommit']or hm['realizationId']!=history['realizationId']:raise ValueError('keys-historical-profile')
    if codec.sha(hi_raw)!=hr['sourceInventorySha256']or hm['executable']['sha256']!=hr['binarySha256']or not any(d['sha256']==hr['backendInput']['sha256']and d['bytes']==hr['backendInput']['bytes']for d in hm['backendManifests']):raise ValueError('keys-historical-correspondence')
    old_entries={d['path']:d for d in hi['files']}
    if len(old_entries)!=len(hi['files'])or len(old_entries)!=hm['source']['inventory']['trackedFiles']:raise ValueError('keys-historical-inventory')
    for desc in hr['sources']:
        raw=source(base+'/source-subset/'+desc['path']);entry=old_entries[desc['path']]
        if codec.sha(raw)!=desc['sha256']or len(raw)!=desc['bytes']or codec.sha(raw)!=entry['sha256']or len(raw)!=entry['bytes']or hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()!=entry['gitBlob']:raise ValueError('keys-historical-source')
    hp=base+'/source-subset/scripts/distribution/check-paths-cli.py';source(hp)
    codec.verify_legacy(config.source_root/(base+'/source-subset'),config.source_root/hp,hr['cases'],hr['harnessSha256'])
    if history['legacyCases']!=463 or len(hr['cases'])!=512 or any(r['scope']!='legacy'or r['role']!='namespace'or r['exit']!=0 or r['stderrHex']or r['migrations']or bytes.fromhex(r['responseHex'])!=codec.encoded(codec.FENCE)+b'\n'for r in hr['cases'][:463]):raise ValueError('keys-historical-namespace')
    s.take(config.build_root/'command-b.json','evidence/build-command.json',4*1024*1024);build_command=codec.document(s.artifacts['evidence/build-command.json'])
    if codec.sha(s.artifacts['evidence/build-command.json'])!=BUILD or build_command['sourceCommit']!=SOURCE:raise ValueError('keys-build-pin')
    build=next(c for c in build_command['commands']if c['phase']=='cli-build')
    expected=['build','--offline','--locked','--release','-j1','--target','aarch64-apple-darwin','-p','weft-runtime','--no-default-features','--features',FEATURE,'--bin',BINARY_NAME]
    if build['argv'][1:]!=expected or build_command['tools']['rustc']!='rustc 1.90.0 (1159e78c4 2025-09-14)':raise ValueError('keys-build-profile')
    for phase,name in [('cli-build','cli-build-outcome.json'),('source-metadata','metadata-build-outcome.json')]:
        s.take(config.build_root/name,'evidence/'+name);value=codec.document(s.artifacts['evidence/'+name])
        if value['commandSha256']!=BUILD or value['phase']!=phase or value['exitCode']!=0 or value['openingClosingCustody']is not True or value['environmentInherited']is not False or phase=='cli-build'and value['binary']['sha256']!=BINARY:raise ValueError('keys-build-outcome')
    for name,destination in [('Cargo.toml','producer/metadata-probe/Cargo.toml'),('Cargo.lock','producer/metadata-probe/Cargo.lock'),('src/main.rs','producer/metadata-probe/src/main.rs')]:
        path=config.build_root/'metadata-probe'/name;s.take(path,destination)
        desc=next(d for d in build_command['resources']if d['path']==str(path))
        if codec.sha(s.artifacts[destination])!=desc['sha256']or len(s.artifacts[destination])!=desc['bytes']:raise ValueError('keys-metadata-producer')
    schemas=[]
    for name in names:
        if name.startswith('docs/helix/02-design/contracts/')and name.endswith('.schema.json'):source(name);schemas.append(codec.descriptor('source-subset/'+name,s.artifacts['source-subset/'+name]))
    if len(schemas)!=20:raise ValueError('keys-schema-count')
    for name in ['Cargo.lock','rust-toolchain.toml','spec/upstream/umf-0.7.0.schema.json','spec/upstream/umf-0.8.0.schema.json','scripts/distribution/backend-paths-metadata.rs']:source(name)
    corpus_rows=rows[:55];control_rows=rows[55:]
    for kind,selected,compiled in [('corpus',corpus_rows,36),('controls',control_rows,3)]:
        raw=b''.join(codec.encoded(r)+b'\n'for r in selected)
        if len(raw)>codec.DECODED_LIMIT:raise ValueError('keys-decoded-bound')
        s.generated('evidence/'+kind+'.jsonl.gz',gzip.compress(raw,mtime=0))
        summary={'profile':'weft-paths-keys-produced-corpus/0.1','cases':len(selected),'compiled':compiled,'blocked':len(selected)-compiled,'binarySha256':BINARY,'sourceCommit':SOURCE,'decodedSha256':codec.sha(raw),'decodedBytes':len(raw),'qualification':'Exact compiler/schema/accepted-case correspondence only; no native or host authority.'}
        s.generated('evidence/'+kind+'-summary.json',codec.encoded(summary)+b'\n');s.generated('evidence/'+kind+'-custody.json',codec.encoded(dict(sourceCommit=SOURCE,binarySha256=BINARY,fullReceiptSha256=RECEIPT,expectedCasesSha256=CASES,decodedSha256=codec.sha(raw),decodedBytes=len(raw)))+b'\n')
    s.generated('evidence/transport.json',codec.encoded(dict(binarySha256=BINARY,controls=receipt['transport']))+b'\n');s.generated('evidence/tools.json',codec.encoded(build_command['tools'])+b'\n')
    env=build_command['environment'];record={'format':'weft-distribution/0.1','realizationId':config.realization_id,'source':{'commit':SOURCE,'inventory':{'artifact':codec.descriptor('evidence/source-inventory.json.gz',s.artifacts['evidence/source-inventory.json.gz']),'decodedSha256':INVENTORY,'decodedBytes':len(inventory_raw),'trackedFiles':1987}},'build':{'release':True,'target':'aarch64-apple-darwin','features':[FEATURE],'command':build['argv'],'tools':codec.descriptor('evidence/tools.json',s.artifacts['evidence/tools.json']),'lockfiles':[codec.descriptor('source-subset/Cargo.lock',s.artifacts['source-subset/Cargo.lock'])],'toolchain':codec.descriptor('source-subset/rust-toolchain.toml',s.artifacts['source-subset/rust-toolchain.toml']),'effectiveEnvironment':{'observed':{'CARGO_HOME':env['CARGO_HOME'],'RUSTUP_HOME':env['RUSTUP_HOME'],'CARGO_TARGET_DIR':env['CARGO_TARGET_DIR'],'PATHPrefix':env['PATH']},'unknowns':build_command['uncertainties']},'platform':codec.platform(s.artifacts['bin/'+BINARY_NAME],build_command['platformObservation']['observedOS'])},'executable':codec.descriptor('bin/'+BINARY_NAME,s.artifacts['bin/'+BINARY_NAME]),'backendManifests':[codec.descriptor('backend/backend-manifest.json',backend_raw)],'publicSchemas':schemas,'conformance':{'corpus':{'cases':55,'compiled':36,'blocked':19,**{k:codec.descriptor('evidence/'+v,s.artifacts['evidence/'+v])for k,v in [('responses','corpus.jsonl.gz'),('summary','corpus-summary.json'),('custody','corpus-custody.json')]}},'controls':{'cases':19,**{k:codec.descriptor('evidence/'+v,s.artifacts['evidence/'+v])for k,v in [('responses','controls.jsonl.gz'),('summary','controls-summary.json'),('custody','controls-custody.json')]}},'transport':codec.descriptor('evidence/transport.json',s.artifacts['evidence/transport.json'])}}
    s.take(Path(codec.__file__).resolve(),'producer/assemble-paths-distribution.py');s.take(Path(__file__).resolve(),'producer/paths_keys_distribution.py');s.generated('manifest.json',codec.encoded(record)+b'\n')
    proof={'format':'weft-distribution-assembly-custody/0.1','sourceCommit':SOURCE,'qualification':'Inert PathsKeys3a compiler-qualified candidate; producerH separate from compiler source. Historical463 retained, not replayed. No index/native/source/publication/ACK or hermetic rebuild claim.','producerSha256':codec.sha(s.artifacts['producer/assemble-paths-distribution.py']),'artifacts':[codec.descriptor(name,raw)for name,raw in sorted(s.artifacts.items())],'sourceSubset':sorted(name[len('source-subset/'):]for name in s.artifacts if name.startswith('source-subset/'))}
    s.generated('assembly-custody.json',codec.encoded(proof)+b'\n');codec.publish(s,config.output,executable='bin/'+BINARY_NAME);return record
