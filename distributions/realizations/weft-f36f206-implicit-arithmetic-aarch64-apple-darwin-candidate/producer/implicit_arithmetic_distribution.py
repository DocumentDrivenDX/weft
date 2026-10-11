"""Additive 0.4.2 candidate assembly; this owner grants no index authority.

Historical assemblers and realization artifacts remain separate. Input evidence
must describe the fresh compiler, not an inherited 0.4.1 qualification.
"""
import json
import os
from pathlib import Path
import shutil
import tempfile

from count_star_distribution import Snapshot, cleanup_primary, read, write_exclusive

SOURCE = 'f36f206eef70f69f2386a2acb466814d8302b016'
VERSION = '0.4.2-implicit-arithmetic-labels-candidate'
BINARY = '8e1e43aa81b312a124e88698477e1392f6c538451d5cb8f582a38934c776e55a'
BINARY_BYTES = 8688144
BACKEND = '33de51e60a1f2eb46813abe8601d90dc7cbe67d6a6b248890dcc70842bfcbe6e'
BACKEND_BYTES = 53868
FEATURE = 'ashlar-databricks-implicit-arithmetic'
REALIZATION = 'weft-f36f206-implicit-arithmetic-aarch64-apple-darwin-candidate'
EXECUTABLE = 'bin/weft-implicit-arithmetic'


TRANSPORT = 'b1c3b32d6038dd43bc50c2b09028ca4242d79f0548d6f3656da0c807e53db6bb'
FIXED_EVIDENCE = {
    'producer/corpus-expectations.py': (3471, '66ee1270d2b08ac53c7660b1c7bd66ef2b09d44a554d027362fdfbe53a306264'),
    'producer/corpus-author.py': (7921, 'ce21e5d96f0c6b9aef4b2be9c1feb96a84f7dae64dc9a1ce3075d6a38b403c94'),
    'evidence/migration-ledger.json': (22748, '0d5c1d428c3f6c6d8b11a87369cb97315989b35f91cb3726a1e21122f185d4f7'),
    'evidence/authored-corpus.json.gz': (616168, 'd941d430062481058d27813a3e0de0b8861ec1684b705648f0ab174b6d9690af'),
    'evidence/source-inventory.json.gz': (202908, 'a3c76de463004879c8b2bdbecc8c809a96a712b794b8f34e4b95fd31e1f6e65b'),
    'evidence/corpus-responses.json.gz': (552991, 'eceff6a9669857fb8506b0d262daff0cf897bb2c2c547ab0f855d60244f3d6bf'),
    'evidence/controls-responses.json.gz': (199351, 'e0cc3bed5a65c3005bd88acfb4bf87cc4ad601d5c6de51f2897c115d810f27b6'),
    'evidence/build-command.json': (56681, 'a880ed3cd99b34c482165cbae184c58775e521e83afe48e5bada773ff10a2649'),
    'evidence/build-receipt.json': (2947, 'cd53ef9f6736457396f6f3e706732904c36dfa28df82f9b746767c27d32bc24b'),
    'evidence/tool-observations.json': (2142, '8c790e7f63ccb056d27e263c531ec17e0720fc03cbbdef613a68d9c614a8cb9e'),
    'evidence/python-parity.json': (10405195, '3a633774153dc7a8cc28c97582ab4a5998d7319627fbadb48197c52aa668bef7'),
    'evidence/browser-parity.json': (10416153, 'c010f43e34ddeb8f854b83420121f464d350bc6140ef0f5934c3ff19fcdcabe2'),
    'evidence/metadata-outcome.json': (311, 'da304ef38d8a850cc1336b3fb103e5f982c7feb00a28d417bb8d778e95cf2d3b'),
    'evidence/transport-outcome.json': (311, '9edf27547eff3d7e752efb23e58d6b18c7d70f4861c832979f46cf9e5f8937c1'),
}

PARITY_RECEIPTS = {
    'python': '9f6697223781e25a8b3a419ae1a4ea8d99f1dcdf50252d9544376b959bbd3c69',
    'browser': '105034c403c66850c4eabe8e2d39eb8c619ba1246d7344a4672a4bd9407cca93',
}


def verify_parity_outcome(parity, outcome):
    if parity.get('success') is not True or parity.get('failure') is not None:
        raise ValueError('embedding-final-success')
    if outcome.get('openingClosingCustody') is not True or outcome.get('failure') is not None:
        raise ValueError('embedding-closing-custody')
    if type(outcome.get('capture')) is not dict or type(outcome['capture'].get('exitCode')) is not int or outcome['capture']['exitCode'] != 0:
        raise ValueError('embedding-exit')


def verify_transport(raw, codec):
    # This exact independently accepted byte record binds input hashes, exit,
    # raw stdout/stderr and all expected markers, not merely eight names.
    if codec.sha(raw) != TRANSPORT:
        raise ValueError('actual-transport-correspondence')
    records = [codec.document(line) for line in raw.splitlines()]
    if [item['id'] for item in records] != list(codec.TRANSPORT_IDS):
        raise ValueError('transport-count')
    return records


def verify_records(rows, authored, backend, codec, semantic_check):
    """Bind every original request/response and every accepted capability witness.

    semantic_check is the independently reviewed authored-corpus checker. Its
    executable identity belongs to the assembly command's closed input pins.
    """
    if backend['backendId'] != 'ashlar.databricks.paths-keys' or backend['backendVersion'] != VERSION:
        raise ValueError('implicit-backend')
    if backend['languageProfiles'] != [{'dialectProfile': 'weft-sql/0.4.2', 'irVersion': 'weft-ir/0.4.2'}]:
        raise ValueError('implicit-language')
    if backend['evidence'] != []:
        raise ValueError('candidate-evidence')
    caps = {item['id'] for item in backend['capabilities']}
    if len(caps) != 50 or len(backend['capabilities']) != 50:
        raise ValueError('implicit-capabilities')
    if len(rows) != 94 or len(authored) != 94 or len({r['id'] for r in rows}) != 94:
        raise ValueError('implicit-case-count')
    selected = set()
    groups = {'corpus': [], 'controls': []}
    for row, case in zip(rows, authored):
        if row['id'] != case['id'] or row['requestHex'] != case['requestHex']:
            raise ValueError('original-request-correspondence')
        request = bytes.fromhex(row['requestHex'])
        raw = bytes.fromhex(row['responseHex'])
        if len(request) > 16 * 1024 * 1024 or len(raw) > 4 * 1024 * 1024 or not raw.endswith(b'\n'):
            raise ValueError('response-bound')
        if codec.sha(raw) != row['responseSha256']:
            raise ValueError('response-pin')
        response = codec.document(raw)
        semantic_check(case, response)
        required = response.get('logicalPlan', {}).get('requiredCapabilities', [])
        if row['status'] != response['status'] or row['requiredCapabilities'] != required:
            raise ValueError('response-witness')
        if row['diagnosticCodes'] != [d['code'] for d in response.get('diagnostics', [])]:
            raise ValueError('diagnostic-witness')
        if response['status'] == 'compiled':
            if response['backend']['backendVersion'] != VERSION or response['logicalPlan']['irVersion'] != 'weft-ir/0.4.2':
                raise ValueError('compiled-profile')
            selected.update(required)
        elif response['status'] != 'blocked':
            raise ValueError('response-status')
        kind = 'controls' if row['id'].startswith(('inherited042:controls:', 'labels:', 'fence:')) else 'corpus'
        groups[kind].append(row)
    if selected != caps or sum(r['status'] == 'compiled' for r in rows) != 51:
        raise ValueError('actual-capability-coverage')
    if len(groups['corpus']) != 65 or len(groups['controls']) != 29:
        raise ValueError('conformance-split')
    if sum(r['status'] == 'compiled' for r in groups['corpus']) != 43:
        raise ValueError('corpus-split')
    return groups


def publish(snapshot, output):
    """Publish cooperating readonly copies; Git modes remain a distinct layer."""
    output = Path(output)
    if output.exists() or output.is_symlink() or any(p.is_symlink() for p in output.parents):
        raise ValueError('fresh-contained-output')
    snapshot.close()
    temporary = Path(tempfile.mkdtemp(prefix='.implicit-arithmetic-distribution-', dir=output.parent))
    primary = None
    try:
        for name, raw in snapshot.artifacts.items():
            destination = temporary / name
            destination.parent.mkdir(parents=True, exist_ok=True)
            write_exclusive(destination, raw)
            destination.chmod(0o555 if name == EXECUTABLE else 0o444)
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


PRODUCT_NAMES = {
    'weft-implicit-arithmetic': EXECUTABLE,
    'libweft.dylib': 'components/python/weft.abi3.so',
    'weft_sql-0.1.0-cp39-abi3-macosx_11_0_arm64.whl': 'components/python/weft_sql.whl',
    'weft_wasm.wasm': 'components/browser/raw-weft.wasm',
    'weft.js': 'components/browser/weft.js',
    'weft_bg.wasm': 'components/browser/weft_bg.wasm',
    'weft_bg.wasm.d.ts': 'components/browser/weft_bg.wasm.d.ts',
    'weft.d.ts': 'components/browser/weft.d.ts',
}


def verify_products(artifacts, receipt, codec):
    products = [p for row in receipt['records'] for p in row['products']]
    by_name = {}
    for product in products:
        name = Path(product['path']).name
        if name in by_name:
            raise ValueError('duplicate-build-product')
        by_name[name] = product
    delivered = {name for name in artifacts if name.startswith(('bin/', 'components/'))}
    if set(by_name) != set(PRODUCT_NAMES) or delivered != set(PRODUCT_NAMES.values()):
        raise ValueError('complete-delivered-products')
    for original, destination in PRODUCT_NAMES.items():
        product = by_name[original]
        raw = artifacts[destination]
        if type(product['bytes']) is not int or len(raw) != product['bytes'] or codec.sha(raw) != product['sha256']:
            raise ValueError('fresh-component-correspondence')


def build_environment(environment):
    """Owning manifest subset; full observed environment stays in build evidence."""
    return {key: environment[key] for key in ('CARGO_HOME', 'RUSTUP_HOME', 'CARGO_TARGET_DIR')} | {'PATHPrefix': environment['PATH']}


def build_phases(receipt):
    if [r['phase'] for r in receipt['records']] != ['cli', 'python', 'wasm', 'wasm-bindings'] or any(r['capture']['exitCode'] != 0 for r in receipt['records']):
        raise ValueError('build-phases')


def assemble_implicit_arithmetic(config, codec, semantic_check):
    """Inert assembly from pinned original evidence, with a separate index gate.

    The executable checker is a reviewed build input. Config names only passive
    artifact descriptors; it never discovers a compiler, tool, or credential.
    """
    if type(config) is not dict or set(config) != {'source_root', 'output', 'artifacts'}:
        raise ValueError('assembly-config')
    if type(config['source_root']) is not str or type(config['output']) is not str or type(config['artifacts']) is not dict:
        raise ValueError('assembly-config-types')
    snapshot = Snapshot(codec)
    for name, item in config['artifacts'].items():
        if type(name) is not str or type(item) is not dict or set(item) != {'path', 'bytes', 'sha256'}:
            raise ValueError('artifact-config')
        if type(item['path']) is not str or type(item['bytes']) is not int or item['bytes'] < 0 or type(item['sha256']) is not str:
            raise ValueError('artifact-config-types')
        snapshot.take(item['path'], name, item['sha256'], item['bytes'])

    for name, (size, pin) in FIXED_EVIDENCE.items():
        raw = snapshot.artifacts[name]
        if len(raw) != size or codec.sha(raw) != pin:
            raise ValueError("fixed-qualified-evidence:" + name)

    def document(name):
        return codec.document(snapshot.artifacts[name])

    inventory_raw = codec.inflate(snapshot.artifacts['evidence/source-inventory.json.gz'])
    inventory = codec.document(inventory_raw)
    if inventory['sourceCommit'] != SOURCE or inventory['trackedFiles'] != 2704 or len(inventory['files']) != 2704:
        raise ValueError('complete-source-inventory')
    names = [codec.relative(item['path']) for item in inventory['files']]
    if names != sorted(set(names)):
        raise ValueError('source-order')
    indexed = dict(zip(names, inventory['files']))

    def source(name):
        entry = indexed[name]
        path = Path(config['source_root']) / name
        raw = snapshot.take(path, 'source-subset/' + name, entry['sha256'], entry['bytes'])
        import hashlib
        if hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest() != entry['gitBlob']:
            raise ValueError('source-blob')
        if bool(path.stat().st_mode & 0o111) != (entry['gitMode'] == '100755'):
            raise ValueError('source-git-mode')
        return raw

    binary = snapshot.artifacts[EXECUTABLE]
    if len(binary) != BINARY_BYTES or codec.sha(binary) != BINARY:
        raise ValueError('fresh-compiler')
    backend_raw = snapshot.artifacts['backend/backend-manifest.json']
    if len(backend_raw) != BACKEND_BYTES or codec.sha(backend_raw) != BACKEND:
        raise ValueError('fresh-metadata-pin')
    for name in ('evidence/metadata-outcome.json', 'evidence/transport-outcome.json'):
        outcome = document(name)
        if outcome['openingClosingCustody'] is not True or outcome['failure'] is not None or type(outcome['capture']['exitCode']) is not int or outcome['capture']['exitCode'] != 0:
            raise ValueError('actual-qualification-closing')
    backend = document('backend/backend-manifest.json')
    authored = codec.document(codec.inflate(snapshot.artifacts['evidence/authored-corpus.json.gz']))['cases']
    by_id = {}
    for kind in ('corpus', 'controls'):
        for row in codec.document(codec.inflate(snapshot.artifacts['evidence/' + kind + '-responses.json.gz']))['cases']:
            if row['id'] in by_id:
                raise ValueError('duplicate-case')
            by_id[row['id']] = row
    if set(by_id) != {case['id'] for case in authored}:
        raise ValueError('complete-cases')
    rows = [by_id[case['id']] for case in authored]
    groups = verify_records(rows, authored, backend, codec, semantic_check)
    # Bind retained cross-runtime captures to the exact CLI response bytes.
    for kind in ('python', 'browser'):
        parity = document('evidence/' + kind + '-parity.json')
        outcome_raw = snapshot.artifacts['evidence/' + kind + '-outcome.json']
        if codec.sha(outcome_raw) != PARITY_RECEIPTS[kind]:
            raise ValueError('actual-embedding-receipt-pin')
        verify_parity_outcome(parity, codec.document(outcome_raw))
        if len(parity['cases']) != 94:
            raise ValueError('embedding-count')
        for actual, row in zip(parity['cases'], rows):
            expected = bytes.fromhex(row['responseHex'])[:-1]
            if actual['id'] != row['id'] or actual['response'].encode() != expected or (kind == 'python' and actual['responseSha256'] != codec.sha(expected)):
                raise ValueError('embedding-correspondence')
        if kind == 'browser' and parity['trapRetired'] is not True:
            raise ValueError('browser-trap')
    transports = verify_transport(snapshot.artifacts['evidence/transport.jsonl'], codec)
    snapshot.generated('evidence/transport.json', codec.encoded({'binarySha256': BINARY, 'controls': transports}) + b'\n')
    conformance = {}
    for kind, selected in groups.items():
        raw = b''.join(codec.encoded(row) + b'\n' for row in selected)
        import gzip
        snapshot.generated('evidence/' + kind + '.jsonl.gz', gzip.compress(raw, mtime=0))
        compiled = sum(row['status'] == 'compiled' for row in selected)
        summary = {'profile': 'weft-implicit-arithmetic-produced-corpus/0.1', 'cases': len(selected), 'compiled': compiled,
                   'blocked': len(selected) - compiled, 'sourceCommit': SOURCE, 'binarySha256': BINARY,
                   'decodedSha256': codec.sha(raw), 'decodedBytes': len(raw),
                   'qualification': 'Actual CLI and extracted Python/browser component correspondence; no native SQL or host authority.'}
        snapshot.generated('evidence/' + kind + '-summary.json', codec.encoded(summary) + b'\n')
        snapshot.generated('evidence/' + kind + '-custody.json', codec.encoded({
            'sourceCommit': SOURCE, 'binarySha256': BINARY,
            'expectedCasesSha256': codec.sha(snapshot.artifacts['evidence/authored-corpus.json.gz']),
            'decodedSha256': codec.sha(raw), 'decodedBytes': len(raw)}) + b'\n')
        conformance[kind] = {'cases': len(selected)}
        for key, suffix in (('responses', '.jsonl.gz'), ('summary', '-summary.json'), ('custody', '-custody.json')):
            name = 'evidence/' + kind + suffix
            conformance[kind][key] = codec.descriptor(name, snapshot.artifacts[name])
        if kind == 'corpus':
            conformance[kind].update(compiled=compiled, blocked=len(selected) - compiled)
    conformance['transport'] = codec.descriptor('evidence/transport.json', snapshot.artifacts['evidence/transport.json'])
    schemas = []
    for name in names:
        if name.startswith('docs/helix/02-design/contracts/') and name.endswith('.schema.json'):
            schemas.append(codec.descriptor('source-subset/' + name, source(name)))
    if len(schemas) != 37:
        raise ValueError('complete-owning-schemas')
    for name in ('Cargo.lock', 'rust-toolchain.toml', 'spec/upstream/umf-0.7.0.schema.json', 'spec/upstream/umf-0.8.0.schema.json', 'tests/distribution-count-star-corpus/fixtures/count-star-having-041-corpus.json.gz'):
        source(name)
    build_command = document('evidence/build-command.json')
    build_receipt = document('evidence/build-receipt.json')
    if build_command['sourceCommit'] != SOURCE or build_receipt['sourceCommit'] != SOURCE or build_receipt['success'] is not True:
        raise ValueError('fresh-build-receipt')
    build_phases(build_receipt)
    verify_products(snapshot.artifacts, build_receipt, codec)
    build = build_command['commands'][0]
    expected = ['build', '--offline', '--locked', '--release', '-j1', '--target', 'aarch64-apple-darwin', '-p',
                'weft-runtime', '--no-default-features', '--features', FEATURE, '--bin', 'weft-implicit-arithmetic']
    if build['argv'][1:] != expected:
        raise ValueError('build-selection')
    observations = document('evidence/tool-observations.json')
    if any(o['exitCode'] != 0 or o['stderr'] for o in observations['observations']):
        raise ValueError('tool-observations')
    host = next(o for o in observations['observations'] if o['argv'] == ['/usr/bin/sw_vers'])
    if 'ProductVersion:\t\t27.0.1\n' not in host['stdout']:
        raise ValueError('observed-build-platform')
    manifest = {'format': 'weft-distribution/0.1', 'realizationId': REALIZATION,
                'source': {'commit': SOURCE, 'inventory': {'artifact': codec.descriptor('evidence/source-inventory.json.gz', snapshot.artifacts['evidence/source-inventory.json.gz']),
                    'decodedSha256': codec.sha(inventory_raw), 'decodedBytes': len(inventory_raw), 'trackedFiles': 2704}},
                'build': {'release': True, 'target': 'aarch64-apple-darwin', 'features': [FEATURE], 'command': build['argv'],
                    'tools': codec.descriptor('evidence/tool-observations.json', snapshot.artifacts['evidence/tool-observations.json']),
                    'lockfiles': [codec.descriptor('source-subset/Cargo.lock', snapshot.artifacts['source-subset/Cargo.lock'])],
                    'toolchain': codec.descriptor('source-subset/rust-toolchain.toml', snapshot.artifacts['source-subset/rust-toolchain.toml']),
                    'effectiveEnvironment': {'observed': build_environment(build_command['environment']), 'unknowns': ['Cache origins, OS SDK internal closure and external linker environment are not hermetically reproduced.']},
                    'platform': codec.platform(binary, '27.0.1')},
                'executable': codec.descriptor(EXECUTABLE, binary), 'backendManifests': [codec.descriptor('backend/backend-manifest.json', snapshot.artifacts['backend/backend-manifest.json'])],
                'publicSchemas': schemas, 'conformance': conformance}
    snapshot.take(Path(__file__).resolve(), 'producer/implicit_arithmetic_distribution.py')
    snapshot.take(Path(codec.__file__).resolve(), 'producer/assemble-paths-distribution.py')
    snapshot.take(Path(__file__).with_name('count_star_distribution.py'), 'producer/count_star_distribution.py')
    snapshot.generated('manifest.json', codec.encoded(manifest) + b'\n')
    snapshot.generated('assembly-custody.json', codec.encoded({'format': 'weft-distribution-assembly-custody/0.1', 'sourceCommit': SOURCE,
        'qualification': 'Fresh f36/042 build, original94 CLI and extracted Python/browser correspondence,50 accepted capability witnesses,eight CLI transport observations. No native SQL, installed host authority or trusted index claim. Prepared readonly modes differ from Git delivery modes.',
        'producerSha256': codec.sha(snapshot.artifacts['producer/implicit_arithmetic_distribution.py']),
        'sourceSubset': sorted(name[len('source-subset/'):] for name in snapshot.artifacts if name.startswith('source-subset/')),
        'artifacts': [codec.descriptor(name, raw) for name, raw in sorted(snapshot.artifacts.items())]}) + b'\n')
    publish(snapshot, config['output'])
    return manifest
