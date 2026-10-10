"""Assemble one inert, offline-verifiable CLI candidate; never admit an index.

Only retained source/corpus receipts are consumed. No compiler, native engine,
network or package resolver is invoked. Complete rebuild inputs remain the
explicit immutable upstream Git commit, not this selected verification subset.
"""
import argparse
from dataclasses import dataclass
import hashlib
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile

SOURCE_COMMIT = '362a9c6af85996a80ed4af6f68efecfe6eb22110'
BINARY_SHA = 'ab899150d217a26368ab0c0954a627401d41031fc768f758247231ea513a3437'
BINARY_BYTES = 4494112
# These are retained qualification receipt identities for this initial producer
# profile, not a trusted distribution index or an authorization source.
RECEIPT_SHA = {
    'cli-produced-corpus-20261009': '5f42488a80d1a24dd628664019f78e6a7985b87c79605de3901470042731b015',
    'cli-candidate-controls-20261009': '6f0e73e7ca82c76e18fd487054ab81c365c146906385692ade267ec02ad29bca',
    'source-backend-metadata-20261009': '16089766839c2cc25c417f8e84f913eed3320a4cda90095dba6c7042fb302abc',
}
FILE_LIMIT = 32 * 1024 * 1024
TOTAL_LIMIT = 96 * 1024 * 1024
JSON_LIMIT = 4 * 1024 * 1024
DECODED_LIMIT = 20 * 1024 * 1024
SCOPES = ('columns-native', 'application-native', 'key-refusal', 'unsigned-columns',
          'optional-native', 'relationship-native', 'compound-native',
          'compound-boundaries-native', 'compound-application-native')
CONTROL_IDS = ('candidate-opt-out', 'unknown-backend', 'wrong-backend-version',
               'wrong-profile', 'unknown-envelope-member', 'mismatched-interface-dialect',
               'binding-digest-mismatch', 'model-digest-mismatch', 'sql-byte-limit',
               'binding-byte-limit', 'duplicate-envelope-member', 'missing-publication',
               'negative-table-version', 'duplicate-table-uuid', 'missing-table-mapping',
               'inconsistent-model-pin', 'fresh-binding-0', 'fresh-binding-1', 'fresh-binding-2')


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def encoded(value):
    return json.dumps(value, ensure_ascii=False, separators=(',', ':')).encode()


def unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate JSON member')
        result[key] = value
    return result


def json_value(raw):
    return json.loads(raw, object_pairs_hook=unique, parse_constant=lambda _: (_ for _ in ()).throw(ValueError('nonfinite JSON')))


def relative(path):
    if type(path) is not str or not path or '\\' in path or ':' in path or path.startswith('/') or any(ord(c) < 32 or ord(c) == 127 for c in path) or any(p in ('', '.', '..') for p in path.split('/')):
        raise ValueError('artifact path')
    return path


def contained(root, path):
    candidate = root / relative(path)
    if any(p.is_symlink() for p in (candidate, *candidate.parents)) or not candidate.is_file():
        raise ValueError('artifact containment')
    candidate.resolve(strict=True).relative_to(root.resolve(strict=True))
    return candidate


def bounded_read(path, limit=FILE_LIMIT):
    if any(p.is_symlink() for p in (path, *path.parents)) or not path.is_file():
        raise ValueError('input containment')
    with path.open('rb') as stream:
        raw = stream.read(limit + 1)
    if len(raw) > limit:
        raise ValueError('input byte limit')
    return raw


def inflate(raw, limit=DECODED_LIMIT):
    # A single member only; concatenated/trailing gzip data is not accepted.
    import zlib
    decoder = zlib.decompressobj(16 + zlib.MAX_WBITS)
    result = decoder.decompress(raw, limit + 1)
    if len(result) > limit or decoder.unconsumed_tail or not decoder.eof or decoder.unused_data:
        raise ValueError('gzip member or decoded byte limit')
    return result


class Snapshot:
    """Freeze all consumed bytes before publishing and verify originals at close."""
    def __init__(self):
        self.inputs = {}
        self.artifacts = {}
        self.total = 0

    def take(self, root, path, destination, limit=FILE_LIMIT):
        source = contained(root, path)
        raw = bounded_read(source, limit)
        key = str(source)
        if key in self.inputs and self.inputs[key] != raw:
            raise ValueError('input drift')
        self.inputs[key] = raw
        destination = relative(destination)
        if destination in self.artifacts and self.artifacts[destination] != raw:
            raise ValueError('conflicting destination')
        if destination not in self.artifacts:
            self.total += len(raw)
            if self.total > TOTAL_LIMIT:
                raise ValueError('assembly total limit')
        self.artifacts[destination] = raw
        return raw

    def generated(self, destination, raw):
        if destination in self.artifacts:
            raise ValueError('generated destination collision')
        self.total += len(raw)
        if self.total > TOTAL_LIMIT:
            raise ValueError('assembly total limit')
        self.artifacts[relative(destination)] = raw

    def close(self):
        for path, raw in self.inputs.items():
            if bounded_read(Path(path)) != raw:
                raise ValueError('input drift')

    def descriptor(self, path):
        raw = self.artifacts[path]
        return {'path': path, 'sha256': sha(raw), 'bytes': len(raw)}


def source_entry(raw, entry):
    if set(entry) != {'path', 'mode', 'gitBlob', 'sha256', 'bytes'} or type(entry['bytes']) is not int:
        raise ValueError('source inventory entry')
    relative(entry['path'])
    if entry['mode'] not in ('100644', '100755') or not re.fullmatch('[0-9a-f]{40}', entry['gitBlob']) or not re.fullmatch('[0-9a-f]{64}', entry['sha256']):
        raise ValueError('source inventory identity')
    blob = hashlib.sha1(b'blob ' + str(len(raw)).encode() + b'\0' + raw).hexdigest()
    if len(raw) != entry['bytes'] or sha(raw) != entry['sha256'] or blob != entry['gitBlob']:
        raise ValueError('source custody')


def verify_reports(snapshot, evidence, scope):
    prefix = 'evidence/' + scope + '/'
    custody_raw = snapshot.take(evidence, scope + '/custody.json', prefix + 'custody.json', JSON_LIMIT)
    if sha(custody_raw) != RECEIPT_SHA[scope]:
        raise ValueError('original receipt identity')
    custody = json_value(custody_raw)
    for desc in custody['reports']:
        if set(desc) != {'path', 'sha256', 'bytes'} or type(desc['bytes']) is not int:
            raise ValueError('report descriptor')
        raw = snapshot.take(evidence, scope + '/' + relative(desc['path']), prefix + desc['path'])
        if sha(raw) != desc['sha256'] or len(raw) != desc['bytes']:
            raise ValueError('retained report custody')
    return custody


def public_case(record):
    if set(record) != {'id', 'requestSha256', 'exit', 'stdoutSha256', 'stderrSha256', 'stdout', 'stderr'}:
        raise ValueError('corpus response shape')
    stdout, stderr = record['stdout'].encode(), record['stderr'].encode()
    if type(record['exit']) is not int or record['exit'] != 0 or stderr or sha(stdout) != record['stdoutSha256'] or sha(stderr) != record['stderrSha256'] or stdout.count(b'\n') != 1 or not stdout.endswith(b'\n'):
        raise ValueError('corpus transport custody')
    response = json_value(stdout)
    if response['status'] not in ('compiled', 'blocked'):
        raise ValueError('corpus response status')
    return response


def read_rows(raw):
    return [json_value(line) for line in raw.splitlines() if line]


def verify_corpus(snapshot, custody, summary, source_entries, take_source):
    raw = inflate(snapshot.artifacts['evidence/cli-produced-corpus-20261009/cases.jsonl.gz'])
    if sha(raw) != custody['caseReportDecodedSha256'] or len(raw) != custody['caseReportDecodedBytes']:
        raise ValueError('decoded corpus custody')
    originals = []
    source_paths = []
    base = 'docs/helix/04-build/evidence/'
    for scope in SCOPES:
        path = base + 'B-006-' + scope + '/compile-artifacts.jsonl'
        source_paths.append(path)
        for item in read_rows(take_source(path)):
            originals.append((scope + ':' + str(item['id']), item))
    for scope in ('scalar-native', 'global-native'):
        paths = sorted(p for p in source_entries if p.startswith(base + 'B-006-' + scope + '/') and p.endswith('-compile.json'))
        for path in paths:
            source_paths.append(path)
            originals.append((scope + ':' + Path(path).stem, json_value(take_source(path))))
    path = base + 'B-006-cross-module-native/compile.json'
    source_paths.append(path)
    originals.append(('cross-module', json_value(take_source(path))))
    if source_paths != [item['path'] for item in summary['sourceFiles']]:
        raise ValueError('corpus source inventory')
    for item in summary['sourceFiles']:
        if sha(snapshot.artifacts['source-subset/' + item['path']]) != item['sha256']:
            raise ValueError('corpus source digest')
    records = read_rows(raw)
    ids = [r['id'] for r in records]
    if len(records) != 463 or ids != [identity for identity, _ in originals] or len(set(ids)) != 463:
        raise ValueError('complete ordered case identities')
    counts = {'compiled': 0, 'blocked': 0}
    migrations, corrections = [], []
    for record, (_, original) in zip(records, originals):
        if record['requestSha256'] != sha(encoded(original['request'])):
            raise ValueError('original request identity')
        response = public_case(record)
        expected = original['response']
        original_expected = json.dumps(expected, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode()
        if record['id'] == 'unsigned-columns:unsigned-64':
            old = {'diagnostics': [{'code': 'WFT-BINDING', 'message': 'Native column carrier cannot establish the original logical scalar domain', 'phase': 'binding', 'recoverability': 'correct-input', 'severity': 'error'}], 'interfaceVersion': 'weft-compile/0.1.0', 'status': 'blocked'}
            if expected != old:
                raise ValueError('historical refusal migration')
            expected['diagnostics'][0]['phase'] = 'lower'
            migrations.append({'id': record['id'], 'path': '/diagnostics/0/phase', 'oldValue': 'binding', 'newValue': 'lower', 'sourceCommit': '39c35f5cfafc305a91d7c602b1775aa8a8d063d5', 'originalExpectedSha256': sha(original_expected), 'migratedExpectedSha256': sha(json.dumps(expected, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode())})
        for obligation in expected.get('obligations', []):
            if obligation['id'] == 'ashlar.candidate.publication':
                profile = obligation['parameters']['nativeProfile']
                if 'versionReported' in profile:
                    if profile.pop('versionReported') != '4.2.0 zero build hash':
                        raise ValueError('historical observation migration')
                    migrations.append({'id': record['id'], 'path': '/obligations/ashlar.candidate.publication/parameters/nativeProfile/versionReported', 'oldValue': '4.2.0 zero build hash', 'originalExpectedSha256': sha(original_expected), 'migratedExpectedSha256': sha(json.dumps(expected, ensure_ascii=False, sort_keys=True, separators=(',', ':')).encode())})
        if expected['status'] == 'compiled':
            for operation in expected['qualification']['operations']:
                declaration = operation['declaration']
                if declaration['id'] == 'value.presence':
                    old = {'subset': 'optional scalar envelopes; absent or exact value; explicit native null refuses'}
                    new = {'subset': 'optional scalar or compound envelopes; absent or exact value; explicit native null refuses'}
                    if declaration['logicalDomain'] not in (old, new):
                        raise ValueError('historical presence metadata')
                    if declaration['logicalDomain'] == old:
                        declaration['logicalDomain'] = new
                        corrections.append(record['id'])
        if response != expected:
            raise ValueError('complete original response correspondence')
        counts[response['status']] += 1
    if migrations != summary['historicalObservationMigration'] or summary['metadataCorrection'] != {'field': 'value.presence.logicalDomain', 'caseIds': corrections}:
        raise ValueError('explicit historical receipt migrations')
    if counts != {'compiled': 462, 'blocked': 1}:
        raise ValueError('corpus counts')
    if any(summary.get(k) is not True for k in ('caseIdsUnique', 'sourceFilesUnchanged', 'fullResponseParity', 'deterministic')) or summary['cases'] != 463:
        raise ValueError('incomplete corpus receipt')
    if json_value(snapshot.artifacts['evidence/cli-produced-corpus-20261009/mismatches.json']) != []:
        raise ValueError('corpus mismatches')
    return counts


def verify_controls(snapshot, custody, summary):
    raw = inflate(snapshot.artifacts['evidence/cli-candidate-controls-20261009/cases.jsonl.gz'])
    if sha(raw) != custody['decodedCaseSha256'] or len(raw) != custody['decodedCaseBytes']:
        raise ValueError('decoded controls custody')
    rows = read_rows(raw)
    if [r['id'] for r in rows] != list(CONTROL_IDS) or summary['cases'] != 19 or summary['blockedCases'] != 16 or summary['compiledCases'] != 3 or summary['custodyClosed'] is not True:
        raise ValueError('control identities or coverage')
    for row in rows:
        if set(row) != {'id', 'requestSha256', 'requestJson', 'stdout', 'stderr', 'exit', 'expectedCode'} or sha(row['requestJson'].encode()) != row['requestSha256'] or type(row['exit']) is not int or row['exit'] != 0 or row['stderr'] or row['stdout'].count('\n') != 1 or not row['stdout'].endswith('\n'):
            raise ValueError('control transport identity')
        response = json_value(row['stdout'])
        if row['expectedCode']:
            if response['status'] != 'blocked' or len(response['diagnostics']) != 1 or response['diagnostics'][0]['code'] != row['expectedCode']:
                raise ValueError('control refusal')
        elif response['status'] != 'compiled':
            raise ValueError('fresh binding control')


def observe_platform(binary):
    # Read-only system tools; the executable is never invoked by assembly.
    def run(argv):
        result = subprocess.run(argv, capture_output=True, timeout=15)
        if result.returncode or len(result.stdout) > JSON_LIMIT or len(result.stderr) > JSON_LIMIT:
            raise ValueError('platform observation')
        return result.stdout.decode()
    file_result = run(['/usr/bin/file', '-b', str(binary)])
    load_commands = run(['/usr/bin/otool', '-l', str(binary)])
    observed_os = run(['/usr/bin/sw_vers', '-productVersion']).strip()
    match = re.search(r'cmd LC_BUILD_VERSION\s+cmdsize \d+\s+platform (\d+)\s+minos ([\d.]+)\s+sdk ([\d.]+)', load_commands)
    if 'Mach-O 64-bit executable arm64' not in file_result or not match or match.group(1) != '1' or observed_os != '27.0.1':
        raise ValueError('unqualified platform')
    platform = {'binaryFormat': 'mach-o', 'machine': 'arm64', 'minimumOS': match.group(2), 'sdk': match.group(3), 'observedOS': observed_os}
    if platform['minimumOS'] != '11.0' or platform['sdk'] != '27.0':
        raise ValueError('platform metadata mismatch')
    return platform, {'file': file_result, 'otool': load_commands, 'sw_vers': observed_os,
                      'qualification': 'Read-only producer platform observation; no portable installation or native query support.'}


@dataclass(frozen=True)
class Config:
    source_root: Path
    evidence_root: Path
    producer_root: Path
    binary: Path
    output: Path
    realization_id: str


def assemble(config):
    if not re.fullmatch('[A-Za-z0-9][A-Za-z0-9._-]{0,255}', config.realization_id):
        raise ValueError('realization identity')
    if config.output.exists() or config.output.is_symlink() or any(p.is_symlink() for p in config.output.parents):
        raise ValueError('output exists or containment')
    snapshot = Snapshot()
    corpus = verify_reports(snapshot, config.evidence_root, 'cli-produced-corpus-20261009')
    controls = verify_reports(snapshot, config.evidence_root, 'cli-candidate-controls-20261009')
    inventory_path = 'evidence/cli-produced-corpus-20261009/source-inventory.json.gz'
    inventory_raw = inflate(snapshot.artifacts[inventory_path], JSON_LIMIT)
    entries = json_value(inventory_raw)
    if type(entries) is not list or len(entries) != 1588:
        raise ValueError('complete source inventory')
    paths = [relative(entry['path']) for entry in entries]
    if paths != sorted(set(paths)):
        raise ValueError('source order or duplicate')
    by_path = dict(zip(paths, entries))
    for custody in (corpus, controls):
        if custody.get('sourceCommit', custody.get('compilerSourceCommit')) != SOURCE_COMMIT or custody['binarySha256'] != BINARY_SHA:
            raise ValueError('source/executable composition')
    if (sha(inventory_raw), len(inventory_raw), len(entries)) != (corpus['sourceInventoryDecodedSha256'], corpus['sourceInventoryDecodedBytes'], corpus['sourceTrackedFiles']):
        raise ValueError('source inventory custody')

    def take_source(path):
        raw = snapshot.take(config.source_root, path, 'source-subset/' + path)
        source_entry(raw, by_path[path])
        actual_mode = '100755' if contained(config.source_root, path).stat().st_mode & 0o111 else '100644'
        if actual_mode != by_path[path]['mode']:
            raise ValueError('source mode custody')
        return raw

    binary = snapshot.take(config.binary.parent, config.binary.name, 'bin/weft-runtime')
    if sha(binary) != BINARY_SHA or len(binary) != BINARY_BYTES:
        raise ValueError('unqualified executable')
    summary = json_value(snapshot.artifacts['evidence/cli-produced-corpus-20261009/summary.json'])
    control_summary = json_value(snapshot.artifacts['evidence/cli-candidate-controls-20261009/summary.json'])
    if any(summary.get(k) != BINARY_SHA for k in ('openingBinarySha256', 'closingBinarySha256', 'binarySha256')) or control_summary['binarySha256'] != BINARY_SHA:
        raise ValueError('executable receipt custody')
    counts = verify_corpus(snapshot, corpus, summary, by_path, take_source)
    verify_controls(snapshot, controls, control_summary)
    for custody in (corpus, controls):
        harness = custody['harness']
        raw = snapshot.take(config.producer_root, harness['path'], 'producer/' + harness['path'])
        if sha(raw) != harness['sha256'] or len(raw) != harness['bytes']:
            raise ValueError('harness custody')
    checker = controls['publicResponseChecker']
    if sha(take_source(checker['path'])) != checker['sha256']:
        raise ValueError('public response checker custody')
    if sha(take_source('docs/helix/04-build/evidence/B-006-cross-module-native/compile.json')) != control_summary['sourceSha256']:
        raise ValueError('control original fixture')
    transport = summary['controls'] + json_value(snapshot.artifacts['evidence/cli-produced-corpus-20261009/io-controls.json'])['controls']
    if [r['id'] for r in transport] != ['oversize', 'invalid-utf8', 'split-utf8-at-limit', 'exact-limit', 'exact-limit-multibyte', 'malformed-json', 'directory-input', 'closed-output']:
        raise ValueError('transport control coverage')
    if json_value(snapshot.artifacts['evidence/cli-produced-corpus-20261009/io-controls.json'])['binarySha256'] != BINARY_SHA:
        raise ValueError('transport executable custody')
    snapshot.generated('evidence/transport-controls.json', encoded({'binarySha256': BINARY_SHA, 'controls': transport,
                       'qualification': 'Original retained eight transport observations; assembly does not reexecute them.'}) + b'\n')

    meta_prefix = 'source-backend-metadata-20261009/'
    metadata_raw = snapshot.take(config.evidence_root, meta_prefix + 'custody.json', 'evidence/' + meta_prefix + 'custody.json', JSON_LIMIT)
    if sha(metadata_raw) != RECEIPT_SHA['source-backend-metadata-20261009']:
        raise ValueError('original metadata receipt identity')
    metadata = json_value(metadata_raw)
    manifest_bytes = snapshot.take(config.evidence_root, meta_prefix + 'backend-manifest.json', 'backend/backend-manifest.json', JSON_LIMIT)
    if metadata['sourceCommit'] != SOURCE_COMMIT or metadata['metadataSha256'] != sha(manifest_bytes) or metadata['metadataBytes'] != len(manifest_bytes) or metadata['sourceInventoryDecodedSha256'] != sha(inventory_raw) or metadata['sourceInventoryDecodedBytes'] != len(inventory_raw) or metadata['sourceTrackedFiles'] != len(entries) or metadata['sourceInventoryCompressedSha256'] != sha(snapshot.artifacts[inventory_path]):
        raise ValueError('source metadata correspondence')
    for name in ('README.md', 'build.log', 'probe-Cargo.lock'):
        snapshot.take(config.evidence_root, meta_prefix + name, 'evidence/' + meta_prefix + name)
    for name, field in (('backend-metadata.rs', 'exporterSha256'), ('describe-backend.py', 'harnessSha256')):
        raw = snapshot.take(config.producer_root, 'scripts/distribution/' + name, 'producer/scripts/distribution/' + name)
        if sha(raw) != metadata[field]:
            raise ValueError('metadata producer correspondence')
    backend = json_value(manifest_bytes)
    if backend['backendId'] != 'ashlar.databricks' or backend['backendVersion'] != '0.1.0-candidate' or backend['interfaceVersion'] != 'weft-backend/0.2.0':
        raise ValueError('backend composition')
    public_schemas = []
    for path in paths:
        if path.startswith('docs/helix/02-design/contracts/') and path.endswith('.schema.json'):
            take_source(path)
            public_schemas.append(snapshot.descriptor('source-subset/' + path))
    if len(public_schemas) != 13:
        raise ValueError('public schema inventory')
    for path in ('Cargo.lock', 'rust-toolchain.toml'):
        take_source(path)
    for path in ('spec/upstream/umf-0.7.0.schema.json', 'spec/upstream/umf-0.8.0.schema.json'):
        take_source(path)
    schema = snapshot.take(config.producer_root, 'docs/helix/02-design/contracts/distribution-manifest.schema.json', 'schemas/distribution-manifest.schema.json', JSON_LIMIT)
    json_value(schema)
    producer = snapshot.take(config.producer_root, 'scripts/distribution/assemble-distribution.py', 'producer/scripts/distribution/assemble-distribution.py')
    platform, observation = observe_platform(config.binary)
    snapshot.generated('evidence/platform-observation.json', encoded(observation) + b'\n')
    build = corpus['build']
    if build['target'] != 'aarch64-apple-darwin' or build['rust'] != '1.90.0' or build['command'] != ['cargo', 'build', '-j1', '-p', 'weft-runtime', '--bin', 'weft-runtime', '--release', '--no-default-features', '--features', 'ashlar-databricks-candidate', '--offline', '--locked']:
        raise ValueError('build composition')
    record = {'format': 'weft-distribution/0.1', 'realizationId': config.realization_id,
              'source': {'commit': SOURCE_COMMIT, 'inventory': {'artifact': snapshot.descriptor(inventory_path), 'decodedSha256': sha(inventory_raw), 'decodedBytes': len(inventory_raw), 'trackedFiles': len(entries)}},
              'build': {'release': True, 'target': build['target'], 'features': ['ashlar-databricks-candidate'], 'command': build['command'],
                        'tools': snapshot.descriptor('evidence/cli-produced-corpus-20261009/tool-versions.json'),
                        'lockfiles': [snapshot.descriptor('source-subset/Cargo.lock')], 'toolchain': snapshot.descriptor('source-subset/rust-toolchain.toml'),
                        'effectiveEnvironment': {'observed': build['effectiveEnvironment'], 'unknowns': build['unknowns']}, 'platform': platform},
              'executable': snapshot.descriptor('bin/weft-runtime'), 'backendManifests': [snapshot.descriptor('backend/backend-manifest.json')], 'publicSchemas': public_schemas,
              'conformance': {'corpus': {'cases': 463, **counts, 'responses': snapshot.descriptor('evidence/cli-produced-corpus-20261009/cases.jsonl.gz'), 'summary': snapshot.descriptor('evidence/cli-produced-corpus-20261009/summary.json'), 'custody': snapshot.descriptor('evidence/cli-produced-corpus-20261009/custody.json')},
                              'controls': {'cases': 19, 'responses': snapshot.descriptor('evidence/cli-candidate-controls-20261009/cases.jsonl.gz'), 'summary': snapshot.descriptor('evidence/cli-candidate-controls-20261009/summary.json'), 'custody': snapshot.descriptor('evidence/cli-candidate-controls-20261009/custody.json')},
                              'transport': snapshot.descriptor('evidence/transport-controls.json')}}
    snapshot.generated('manifest.json', encoded(record) + b'\n')
    proof = {'format': 'weft-distribution-assembly-custody/0.1', 'sourceCommit': SOURCE_COMMIT,
             'qualification': 'Inert assembled candidate only; no trusted index, installation, rebuild, native execution or hermeticity admission. Selected local source subset verifies corpus requests; complete source rebuild separately requires this immutable public Git commit.',
             'producerSha256': sha(producer), 'artifacts': [snapshot.descriptor(p) for p in sorted(snapshot.artifacts)],
             'sourceSubset': sorted(p.removeprefix('source-subset/') for p in snapshot.artifacts if p.startswith('source-subset/'))}
    snapshot.generated('assembly-custody.json', encoded(proof) + b'\n')
    publish(snapshot, config.output)
    return record


def publish(snapshot, output):
    snapshot.close()
    parent = output.parent.resolve(strict=True)
    temporary = Path(tempfile.mkdtemp(prefix='.weft-distribution-', dir=parent))
    try:
        for path, raw in snapshot.artifacts.items():
            destination = temporary / path
            destination.parent.mkdir(parents=True, exist_ok=True)
            with destination.open('xb') as stream:
                stream.write(raw)
                stream.flush()
                os.fsync(stream.fileno())
            destination.chmod(0o555 if path == 'bin/weft-runtime' else 0o444)
            if bounded_read(destination) != raw:
                raise ValueError('copied artifact drift')
        snapshot.close()
        if output.exists() or output.is_symlink():
            raise ValueError('output appeared')
        # Parent directory has explicit trusted single-writer ownership; no
        # cross-principal fencing or trusted installation is claimed here.
        os.rename(temporary, output)
    finally:
        if temporary.exists():
            shutil.rmtree(temporary)

def main():
    class SafeParser(argparse.ArgumentParser):
        def error(self, message):
            self.exit(2, 'WEFT-DISTRIBUTION-ASSEMBLY-CONFIG\n')
    parser = SafeParser()
    for name in ('source-root', 'evidence-root', 'producer-root', 'binary', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--realization-id', required=True)
    args = parser.parse_args()
    result = assemble(Config(args.source_root, args.evidence_root, args.producer_root, args.binary, args.output, args.realization_id))
    print(json.dumps({'state': 'inert-candidate-assembled', 'realizationId': result['realizationId'], 'manifestSha256': sha(bounded_read(args.output / 'manifest.json'))}))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, TypeError, OSError, RuntimeError, RecursionError, subprocess.SubprocessError):
        print('WEFT-DISTRIBUTION-ASSEMBLY-REFUSED', file=__import__('sys').stderr)
        raise SystemExit(2)
