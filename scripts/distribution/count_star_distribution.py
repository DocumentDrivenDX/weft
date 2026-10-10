"""Explicit 5dd/0.4.1 candidate assembly; no execution or index authority.

The fixed compiler source, later qualification producer and assembly producer
have distinct identities. Historical Paths and PathsKeys policies stay separate.
"""
import gzip
import hashlib
import os
from pathlib import Path
import shutil
import stat
import tempfile

SOURCE = '5ddcebd6941c572ddd62b78925fdb1fe78b1c688'
BINARY = '0e8199a8953f46d7151c87ec001e6c8edff414c0af9cecc6c13e0ea8d23ede80'
BINARY_BYTES = 8532672
BACKEND = '075dc92c7ca08b37abd1ed003fcd80674f816353742b79fa3e3a98021826b273'
VERSION = '0.4.1-count-star-having-candidate'
FEATURE = 'ashlar-databricks-paths-keys'
FILE_LIMIT = 32 * 1024 * 1024
TOTAL_LIMIT = 96 * 1024 * 1024
PINS = {
    'source-inventory.json': (687929, '08e32444e85f71cb9e960d1731e41e7fe05d93d44d4b4f710c0c1dfbbe392c70'),
    'build-closure-inventory.json': (79935, '29528ce28cfe22f248bcee1a324434a1c211570a0cd5088b8497ea4bbbe837d3'),
    'command-g.json': (20527, '1fd5580d6f86a0b456414fcaf8305ed7f624928a504bc121134c1fe9a06cd5fa'),
    'command-h.json': (23350, 'f41c8206adfac46eb0f0fca8831ea0ff3a3bd39f0874ab194763809719c2e7c7'),
    'cli-build-g-outcome.json': (439, '6289bf16689f4ca60c0c442e2cc5c0ae2e5f4e70efc284135e1377a63c9e6627'),
    'source-metadata-h-outcome.json': (250, '9b46e52f19e1f78960b0c2ec549c3bfd30afef59283c4d81fb00b65b490900e7'),
    'source-metadata-h.stdout.log': (53114, BACKEND),
    'full041-command-d.json': (38338, 'aef833ed297eb14d375b24edafbd5a19a8e5eed1ee507bbc07815f2532a391f5'),
    'full041-a/actual-80-cases-d.json.gz': (635113, '81cdac9213c064ccf78ff90cd8fa643b78ed245d11977bcb87b36439b3890f35'),
    'full041-a/cli-d-outcome.json': (478, 'b3ef1be1173f0a87598e6ce4f2daf755b84c7a2810e4b2be0ea3ebfa711f8796'),
    'full80-embeddings-e/python-g.json': (3032495, '585c426b3ddcb375ba8a46f54b91dc845ce900d1453e04a539b56b7bdf900544'),
    'full80-embeddings-e/browser-g.json': (3032731, '22783d75b2d403ee8bdcc9bad735b203cd2e478e4ba4c09e0f12897e8c74de6b'),
    'full80-embeddings-e/python-g-outcome.json': (550, '8966bab1843bfca07f0d9af9c535c5ae673ce2f24c28fb330dfbf8ff22ed03a3'),
    'full80-embeddings-e/browser-g-outcome.json': (551, 'aa3384a7d1aba57fc40d2759649657636b64ae90004eb0702f94cc21f8aa458a'),
    'full80-embedding-command-g.json': (26762, '455a23de87899c4cc3311f1fa347bbe4169478a44d44fdd00a40a0c3d46b73ec'),
    'count-star-boundary-command-b.json': (3689, '5885cdbf086ba1429df852d704e381ef886379d2533bd2123ee39614485086ba'),
    'count-star-boundaries-b.observations.jsonl': (2667, 'cafd677534dcce6bc4a78e9034ee0080dc90d299e17f99cfd3321aeda07b05ed'),
    'count-star-boundaries-b-outcome.json': (621, 'f0229bbeb325fc89ca4cbfb0c272d55c9de01942b9bd5e712a771239894f9606'),
    'count-star-boundary-evidence-b.tar.gz': (11079, 'af8e2dcc2dd52b617c6a3e5ed584be1060fe9bbab130155b161fb2c0fe9731d0'),
    'current-assembly-host-os-b.json': (781, 'dcdbb1b487be9dbcf715f2a64fdaf43d4e1c2dbca74be9a25b2dd056ce3dfbf4'),
    'current-exact-build-tool-versions-a.json': (2293, '67621722c99230a50cdce9eb7cd6995deea7f257d5b8a257e56866d547d87fcd'),
    'component-parity-evidence-ab.tar.gz': (1105219, '0e169b25969e92937e2f5994fd5a81ede0943098eeca4debc70e7aaea3a77ca7'),
    'full041-corpus-component-evidence-dg.tar.gz': (3482923, '151a38c5558a446e380a916eaf9abc9314410cdb622ba355145d2b17621e464e'),
}
CORPUS = ('tests/distribution-count-star-corpus/fixtures/count-star-having-041-corpus.json.gz',
          621947, 'd5185357481b49e2c7000190406972264108801a6b99504e70bafee38995a5d9')


def cleanup_primary(primary, closing):
    """Keep body cancellation; closing cancellation outranks an ordinary error."""
    if primary is None:
        return closing
    if isinstance(primary, Exception) and not isinstance(closing, Exception):
        return closing
    try:
        primary.cleanup_failed = True
    except Exception:
        pass
    return primary


def read(path, limit=FILE_LIMIT):
    """Bounded regular descriptor read within a cooperating filesystem custody."""
    path = Path(path)
    if any(p.is_symlink() for p in (path, *path.parents)):
        raise ValueError('contained-regular-input')
    descriptor = None
    primary = None
    raw = None
    try:
        descriptor = os.open(path, os.O_RDONLY | os.O_NONBLOCK | os.O_NOFOLLOW)
        if not stat.S_ISREG(os.fstat(descriptor).st_mode):
            raise ValueError('regular-input')
        chunks = []
        remaining = limit + 1
        while remaining:
            chunk = os.read(descriptor, min(remaining, 65536))
            if not chunk:
                break
            chunks.append(chunk)
            remaining -= len(chunk)
        raw = b''.join(chunks)
        if len(raw) > limit:
            raise ValueError('input-bound')
    except BaseException as error:
        primary = error
    finally:
        if descriptor is not None:
            try:
                os.close(descriptor)
            except BaseException as error:
                primary = cleanup_primary(primary, error)
    if primary is not None:
        raise primary
    return raw


class Snapshot:
    """New-profile custody; never alters shared historical assembly mechanics."""
    def __init__(self, codec):
        self.codec = codec
        self.artifacts = {}
        self.originals = {}
        self.total = 0

    def generated(self, name, raw):
        self.codec.relative(name)
        if name in self.artifacts:
            raise ValueError('duplicate-artifact')
        if len(raw) > FILE_LIMIT or self.total + len(raw) > TOTAL_LIMIT:
            raise ValueError('package-bound')
        self.artifacts[name] = raw
        self.total += len(raw)
        return self.codec.descriptor(name, raw)

    def take(self, path, name, pin=None, size=None):
        path = Path(path)
        raw = read(path)
        if pin is not None and self.codec.sha(raw) != pin:
            raise ValueError('fixed-input-pin:' + name)
        if size is not None and len(raw) != size:
            raise ValueError('fixed-input-size:' + name)
        self.originals[path] = (self.codec.sha(raw), len(raw))
        self.generated(name, raw)
        return raw

    def close(self):
        for path, (pin, size) in self.originals.items():
            raw = read(path, size)
            if len(raw) != size or self.codec.sha(raw) != pin:
                raise ValueError('closing-input-drift')


def write_exclusive(path, raw):
    stream = None
    primary = None
    try:
        stream = path.open('xb')
        if stream.write(raw) != len(raw):
            raise ValueError('short-write')
        stream.flush()
        os.fsync(stream.fileno())
    except BaseException as error:
        primary = error
    finally:
        if stream is not None:
            try:
                stream.close()
            except BaseException as error:
                primary = cleanup_primary(primary, error)
    if primary is not None:
        raise primary


def publish(snapshot, output):
    """Fresh cooperating single-writer publication; no cross-principal fencing."""
    if output.exists() or output.is_symlink() or any(p.is_symlink() for p in output.parents):
        raise ValueError('fresh-contained-output')
    snapshot.close()
    temporary = Path(tempfile.mkdtemp(prefix='.count-star-distribution-', dir=output.parent))
    primary = None
    try:
        for name, raw in snapshot.artifacts.items():
            destination = temporary / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            write_exclusive(destination, raw)
            destination.chmod(0o555 if name == 'bin/weft-paths-keys' else 0o444)
            if read(destination) != raw:
                raise ValueError('copy-drift')
        snapshot.close()
        if output.exists() or output.is_symlink():
            raise ValueError('output-appeared')
        os.rename(temporary, output)
    except BaseException as error:
        primary = error
    finally:
        try:
            if temporary.exists():
                shutil.rmtree(temporary)
        except BaseException as error:
            primary = cleanup_primary(primary, error)
    if primary is not None:
        raise primary


def verify_records(receipt, bundle, backend, codec):
    """Check retained actual response bytes and selected-capability witnesses."""
    if bundle['format'] != 'weft-count-star-having-corpus/0.1' or bundle['sourceCommit'] != SOURCE or bundle['backendVersion'] != VERSION:
        raise ValueError('count-star-authored-profile')
    if receipt['sourceCommit'] != SOURCE or receipt['binarySha256'] != BINARY:
        raise ValueError('count-star-receipt-profile')
    if receipt['protocolCases'] != 80 or receipt['declaredCapabilities'] != 49:
        raise ValueError('count-star-full-corpus')
    if receipt['casesInputSha256'] != CORPUS[2] or receipt['backendInputSha256'] != BACKEND:
        raise ValueError('count-star-input-correspondence')
    if backend['backendId'] != 'ashlar.databricks.paths-keys' or backend['backendVersion'] != VERSION:
        raise ValueError('count-star-backend')
    if backend['interfaceVersion'] != 'weft-backend/0.3.0' or backend['languageProfiles'] != [
            {'dialectProfile': 'weft-sql/0.4.1', 'irVersion': 'weft-ir/0.4.1'}]:
        raise ValueError('count-star-language')
    caps = {c['id'] for c in backend['capabilities']}
    if len(caps) != 49 or len(backend['capabilities']) != 49 or set(receipt['coverage']) != caps:
        raise ValueError('count-star-capabilities')
    rows = receipt['cases']
    expected = bundle['cases']
    if len(rows) != 80 or len(expected) != 80 or len({r['id'] for r in rows}) != 80:
        raise ValueError('count-star-case-count')
    responses = {}
    selected = set()
    for row, case in zip(rows, expected):
        if row['id'] != case['id'] or row['requestHex'] != case['requestHex']:
            raise ValueError('count-star-request-correspondence')
        if len(bytes.fromhex(row['requestHex'])) > 16 * 1024 * 1024:
            raise ValueError('count-star-request-bound')
        raw = bytes.fromhex(row['responseHex'])
        if len(raw) > 4 * 1024 * 1024 or not raw.endswith(b'\n') or codec.sha(raw) != row['responseSha256']:
            raise ValueError('count-star-response-bound')
        response = codec.document(raw)
        if case['expectation'] == 'exact-migrated-bytes':
            if raw != bytes.fromhex(case['responseHex']):
                raise ValueError('count-star-migrated-correspondence')
        elif response != case['expectedResponse']:
            raise ValueError('count-star-independent-correspondence')
        if response['status'] != row['status'] or response['status'] not in ('compiled', 'blocked'):
            raise ValueError('count-star-status')
        required = response.get('logicalPlan', {}).get('requiredCapabilities', [])
        if row['requiredCapabilities'] != required or row['diagnosticCodes'] != [d['code'] for d in response.get('diagnostics', [])]:
            raise ValueError('count-star-response-witness')
        if response['status'] == 'compiled':
            if response['backend']['backendVersion'] != VERSION or response['logicalPlan']['irVersion'] != 'weft-ir/0.4.1':
                raise ValueError('count-star-compiled-profile')
            selected.update(required)
        responses[row['id']] = response
    if selected != caps or sum(r['status'] == 'compiled' for r in rows) != 42:
        raise ValueError('count-star-actual49')
    for cap, item in receipt['coverage'].items():
        authored = bundle['coverage'][cap]
        if any(item[key] != authored[key] for key in ('accepted', 'refused', 'scope')):
            raise ValueError('count-star-authored-witness-correspondence')
        if not item['accepted']:
            raise ValueError('count-star-missing-accepted-witness')
        for identity in item['accepted']:
            response = responses[identity]
            if response['status'] != 'compiled' or cap not in response['logicalPlan']['requiredCapabilities']:
                raise ValueError('count-star-accepted-witness')
        for identity in item['refused']:
            if responses[identity]['status'] != 'blocked':
                raise ValueError('count-star-refused-witness')
    return rows


def assemble_count_star(config, codec):
    """Inert fixed-evidence assembly; execution and trusted index are other ports.

    source_root is the retained 5dd build closure; build_root is the retained
    build/qualification directory; corpus_root contains the later authored corpus.
    """
    snapshot = Snapshot(codec)
    inputs = {}
    for name, (size, pin) in PINS.items():
        inputs[name] = snapshot.take(config.build_root / name, 'evidence/inputs/' + name, pin, size)
    inventory_raw = inputs['source-inventory.json']
    inventory = codec.document(inventory_raw)
    if set(inventory) != {'sourceCommit', 'files'} or inventory['sourceCommit'] != SOURCE or len(inventory['files']) != 2295:
        raise ValueError('count-star-source-inventory')
    names = [codec.relative(d['path']) for d in inventory['files']]
    if names != sorted(set(names)):
        raise ValueError('count-star-source-order')
    indexed = dict(zip(names, inventory['files']))

    def source(name):
        name = codec.relative(name)
        entry = indexed[name]
        raw = snapshot.take(config.source_root / name, 'source-subset/' + name, entry['sha256'], entry['bytes'])
        if hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() != entry['gitBlob']:
            raise ValueError('count-star-source-blob')
        if bool((config.source_root / name).stat().st_mode & 0o111) != (entry['mode'] == '100755'):
            raise ValueError('count-star-source-mode')
        return raw

    binary = snapshot.take(config.build_root / 'weft-paths-keys', 'bin/weft-paths-keys', BINARY, BINARY_BYTES)
    backend_raw = inputs['source-metadata-h.stdout.log']
    snapshot.generated('backend/backend-manifest.json', backend_raw)
    corpus_raw = snapshot.take(config.corpus_root / CORPUS[0], 'evidence/authored-corpus.json.gz', CORPUS[2], CORPUS[1])
    receipt = codec.document(codec.inflate(inputs['full041-a/actual-80-cases-d.json.gz']))
    rows = verify_records(receipt, codec.document(codec.inflate(corpus_raw)), codec.document(backend_raw), codec)

    def outcome(name, command, phase):
        value = codec.document(inputs[name])
        if value['commandSha256'] != PINS[command][1] or value['phase'] != phase or value['exitCode'] != 0:
            raise ValueError('count-star-outcome:' + name)
        if value['openingClosingCustody'] is not True or value['environmentInherited'] is not False:
            raise ValueError('count-star-outcome-custody:' + name)
        return value

    built = outcome('cli-build-g-outcome.json', 'command-g.json', 'cli-build')
    if built['binary']['sha256'] != BINARY or built['binary']['bytes'] != BINARY_BYTES:
        raise ValueError('count-star-built-binary')
    outcome('source-metadata-h-outcome.json', 'command-h.json', 'source-metadata')
    outcome('full041-a/cli-d-outcome.json', 'full041-command-d.json', 'count-star-full-cli')
    outcome('count-star-boundaries-b-outcome.json', 'count-star-boundary-command-b.json', 'count-star-boundaries')
    for kind in ('python', 'browser'):
        outcome('full80-embeddings-e/' + kind + '-g-outcome.json', 'full80-embedding-command-g.json', 'full80-' + kind)
        parity = codec.document(inputs['full80-embeddings-e/' + kind + '-g.json'])
        if len(parity['cases']) != 80:
            raise ValueError('count-star-parity-count')
        for actual, row in zip(parity['cases'], rows):
            expected = bytes.fromhex(row['responseHex'])[:-1]
            if actual['id'] != row['id'] or actual['response'].encode('utf8') != expected or actual['responseSha256'] != codec.sha(expected):
                raise ValueError('count-star-parity-correspondence')
        if kind == 'browser' and parity['trapRetired'] is not True:
            raise ValueError('count-star-browser-trap')

    transports = [codec.document(line) for line in inputs['count-star-boundaries-b.observations.jsonl'].splitlines()]
    if [r['id'] for r in transports] != list(codec.TRANSPORT_IDS):
        raise ValueError('count-star-boundary-count')
    snapshot.generated('evidence/transport.json', codec.encoded({'binarySha256': BINARY, 'controls': transports}) + b'\n')

    schemas = []
    for name in names:
        if name.startswith('docs/helix/02-design/contracts/') and name.endswith('.schema.json'):
            source(name)
            schemas.append(codec.descriptor('source-subset/' + name, snapshot.artifacts['source-subset/' + name]))
    if len(schemas) != 24:
        raise ValueError('count-star-schema-count')
    for name in ('Cargo.lock', 'rust-toolchain.toml', 'spec/upstream/umf-0.7.0.schema.json', 'spec/upstream/umf-0.8.0.schema.json'):
        source(name)
    snapshot.generated('evidence/source-inventory.json.gz', gzip.compress(inventory_raw, mtime=0))

    controls = [r for r in rows if r['id'].startswith('controls:')]
    corpus = [r for r in rows if not r['id'].startswith('controls:')]
    if len(controls) != 19 or sum(r['status'] == 'compiled' for r in controls) != 3 or len(corpus) != 61:
        raise ValueError('count-star-conformance-split')
    conformance = {}
    for kind, selected in (('corpus', corpus), ('controls', controls)):
        raw = b''.join(codec.encoded(r) + b'\n' for r in selected)
        if len(raw) > codec.DECODED_LIMIT:
            raise ValueError('count-star-corpus-bound')
        snapshot.generated('evidence/' + kind + '.jsonl.gz', gzip.compress(raw, mtime=0))
        compiled = sum(r['status'] == 'compiled' for r in selected)
        summary = {'profile': 'weft-count-star-produced-corpus/0.1', 'cases': len(selected), 'compiled': compiled,
                   'blocked': len(selected) - compiled, 'sourceCommit': SOURCE, 'binarySha256': BINARY,
                   'decodedSha256': codec.sha(raw), 'decodedBytes': len(raw),
                   'qualification': 'Actual compiler and component correspondence only; no installation, native SQL or host authority.'}
        snapshot.generated('evidence/' + kind + '-summary.json', codec.encoded(summary) + b'\n')
        custody = {'sourceCommit': SOURCE, 'binarySha256': BINARY, 'fullReceiptCompressedSha256': PINS['full041-a/actual-80-cases-d.json.gz'][1],
                   'expectedCasesSha256': CORPUS[2], 'decodedSha256': codec.sha(raw), 'decodedBytes': len(raw)}
        snapshot.generated('evidence/' + kind + '-custody.json', codec.encoded(custody) + b'\n')
        conformance[kind] = {'cases': len(selected), **{key: codec.descriptor('evidence/' + name, snapshot.artifacts['evidence/' + name])
                            for key, name in [('responses', kind + '.jsonl.gz'), ('summary', kind + '-summary.json'), ('custody', kind + '-custody.json')]}}
        if kind == 'corpus':
            conformance[kind].update(compiled=compiled, blocked=len(selected) - compiled)
    conformance['transport'] = codec.descriptor('evidence/transport.json', snapshot.artifacts['evidence/transport.json'])

    build_command = codec.document(inputs['command-g.json'])
    build = next(c for c in build_command['commands'] if c['phase'] == 'cli-build')
    expected_argv = ['build', '--offline', '--locked', '--release', '-j1', '--target', 'aarch64-apple-darwin',
                     '-p', 'weft-runtime', '--no-default-features', '--features', FEATURE, '--bin', 'weft-paths-keys']
    if build_command['sourceCommit'] != SOURCE or build['argv'][1:] != expected_argv:
        raise ValueError('count-star-build-selection')
    versions = codec.document(inputs['current-exact-build-tool-versions-a.json'])
    if versions['sourceBuildCommandSha256'] != PINS['command-g.json'][1]:
        raise ValueError('count-star-version-command')
    build_resources = {d['path']: d for d in build_command['resources']}
    for observation in versions['observations']:
        identity = observation['openingClosingIdentity']
        if identity != build_resources[identity['path']] or observation['exitCode'] != 0 or observation['stderrHex']:
            raise ValueError('count-star-version-identity')
    tools = {'qualification': 'Retained build-command resource identities and separately scoped current version observations of identical tool bytes; no build-environment replay or hermetic claim.',
             'resources': build_command['resources'], 'currentExactToolVersionObservations': versions}
    snapshot.generated('evidence/tools.json', codec.encoded(tools) + b'\n')
    host = codec.document(inputs['current-assembly-host-os-b.json'])
    if host['exitCode'] != 0 or host['observedProductVersion'] != '27.0.1' or host['stdoutHex'] != b'27.0.1\n'.hex():
        raise ValueError('count-star-current-host-observation')
    env = build_command['environment']
    unknowns = list(build_command['uncertainties']) + [
        'platform.observedOS is the separately retained current assembly-host observation; original build-time host OS was not retained.']
    record = {
        'format': 'weft-distribution/0.1', 'realizationId': config.realization_id,
        'source': {'commit': SOURCE, 'inventory': {'artifact': codec.descriptor('evidence/source-inventory.json.gz', snapshot.artifacts['evidence/source-inventory.json.gz']),
                   'decodedSha256': PINS['source-inventory.json'][1], 'decodedBytes': len(inventory_raw), 'trackedFiles': 2295}},
        'build': {'release': True, 'target': 'aarch64-apple-darwin', 'features': [FEATURE], 'command': build['argv'],
                  'tools': codec.descriptor('evidence/tools.json', snapshot.artifacts['evidence/tools.json']),
                  'lockfiles': [codec.descriptor('source-subset/Cargo.lock', snapshot.artifacts['source-subset/Cargo.lock'])],
                  'toolchain': codec.descriptor('source-subset/rust-toolchain.toml', snapshot.artifacts['source-subset/rust-toolchain.toml']),
                  'effectiveEnvironment': {'observed': {'CARGO_HOME': env['CARGO_HOME'], 'RUSTUP_HOME': env['RUSTUP_HOME'],
                       'CARGO_TARGET_DIR': env['CARGO_TARGET_DIR'], 'PATHPrefix': env['PATH']}, 'unknowns': unknowns},
                  'platform': codec.platform(binary, host['observedProductVersion'])},
        'executable': codec.descriptor('bin/weft-paths-keys', binary),
        'backendManifests': [codec.descriptor('backend/backend-manifest.json', backend_raw)], 'publicSchemas': schemas, 'conformance': conformance}
    snapshot.take(Path(codec.__file__).resolve(), 'producer/assemble-paths-distribution.py')
    snapshot.take(Path(__file__).resolve(), 'producer/count_star_distribution.py')
    snapshot.generated('manifest.json', codec.encoded(record) + b'\n')
    proof = {'format': 'weft-distribution-assembly-custody/0.1', 'sourceCommit': SOURCE,
             'qualification': 'Inert fixed5dd/041 candidate, actual80 corpus and eight CLI boundaries; historical040 evidence retained separately. No index, installed, native SQL or host-authority claim. Current assembly OS is not retroactive build-host evidence.',
             'producerSha256': codec.sha(snapshot.artifacts['producer/assemble-paths-distribution.py']),
             'artifacts': [codec.descriptor(name, raw) for name, raw in sorted(snapshot.artifacts.items())],
             'sourceSubset': sorted(name[len('source-subset/'):] for name in snapshot.artifacts if name.startswith('source-subset/'))}
    snapshot.generated('assembly-custody.json', codec.encoded(proof) + b'\n')
    publish(snapshot, config.output)
    return record
