"""Public source-backend metadata probe; never executable or index admission."""
import argparse
import gzip
import hashlib
import io
import json
from pathlib import Path
import subprocess
import sys
import tempfile

LIMIT = 4 * 1024 * 1024
SOURCE_FILE_LIMIT = 32 * 1024 * 1024
SOURCE_TOTAL_LIMIT = 256 * 1024 * 1024


def sha(data):
    return hashlib.sha256(data).hexdigest()


def unique(pairs):
    result = {}
    for key, value in pairs:
        if key in result:
            raise ValueError('duplicate metadata member')
        result[key] = value
    return result


def load_inventory(inventory):
    with inventory.open('rb') as stream:
        compressed = stream.read(LIMIT + 1)
    if len(compressed) > LIMIT:
        raise ValueError('compressed inventory limit')
    with gzip.GzipFile(fileobj=io.BytesIO(compressed)) as stream:
        raw = stream.read(LIMIT + 1)
    if len(raw) > LIMIT:
        raise ValueError('source inventory limit')
    entries = json.loads(raw, object_pairs_hook=unique)
    if not isinstance(entries, list) or not 1 <= len(entries) <= 20000:
        raise ValueError('source inventory shape')
    return raw, entries, compressed


def verify_entries(root, raw, entries):
    paths = []
    total = 0
    for entry in entries:
        if type(entry) is not dict or set(entry) != {'path', 'mode', 'gitBlob', 'sha256', 'bytes'}:
            raise ValueError('source inventory entry')
        path = entry['path']
        if type(path) is not str or not path or '\\' in path or path.startswith('/') or any(part in ('', '.', '..') for part in path.split('/')):
            raise ValueError('source path')
        candidate = root / path
        if any(parent.is_symlink() for parent in (candidate, *candidate.parents)) or not candidate.is_file():
            raise ValueError('source containment')
        if type(entry['bytes']) is not int or not 0 <= entry['bytes'] <= SOURCE_FILE_LIMIT or candidate.stat().st_size != entry['bytes']:
            raise ValueError('source byte bound')
        total += entry['bytes']
        if total > SOURCE_TOTAL_LIMIT:
            raise ValueError('source total limit')
        with candidate.open('rb') as stream:
            data = stream.read(SOURCE_FILE_LIMIT + 1)
        if len(data) > SOURCE_FILE_LIMIT:
            raise ValueError('source byte bound')
        blob = hashlib.sha1(b'blob ' + str(len(data)).encode() + b'\0' + data).hexdigest()
        mode = '100755' if candidate.stat().st_mode & 0o111 else '100644'
        if type(entry['bytes']) is not int or len(data) != entry['bytes'] or sha(data) != entry['sha256'] or blob != entry['gitBlob'] or mode != entry['mode']:
            raise ValueError('source custody')
        paths.append(path)
    if paths != sorted(set(paths)):
        raise ValueError('source order or duplicate')
    actual = sorted(p.relative_to(root).as_posix() for p in root.rglob('*') if p.is_file() or p.is_symlink())
    if actual != paths:
        raise ValueError('source closure')
    return sha(raw), len(raw), len(paths)


def verify_source(root, inventory):
    raw, entries, _ = load_inventory(inventory)
    return verify_entries(root, raw, entries)


def main():
    class SafeParser(argparse.ArgumentParser):
        def error(self, message):
            self.exit(2, 'WEFT-DISTRIBUTION-METADATA-CONFIG\n')
    parser = SafeParser()
    for name in ('source-root', 'source-inventory', 'source-repository', 'toolchain-bin', 'cargo-home', 'rustup-home', 'target-dir', 'output'):
        parser.add_argument('--' + name, type=Path, required=True)
    parser.add_argument('--source-commit', required=True)
    args = parser.parse_args()
    harness_bytes = Path(__file__).read_bytes()
    inventory_raw, inventory, inventory_bytes = load_inventory(args.source_inventory)
    if len(args.source_commit) != 40 or any(c not in '0123456789abcdef' for c in args.source_commit):
        raise ValueError('source commit')
    root = args.source_root.resolve(strict=True)
    opening = verify_entries(root, inventory_raw, inventory)
    tree = subprocess.check_output(['git', '-C', str(args.source_repository.resolve(strict=True)), 'ls-tree', '-rz', '--full-tree', args.source_commit], timeout=30)
    actual_tree = []
    for entry in tree.split(b'\0'):
        if entry:
            meta, path = entry.split(b'\t', 1)
            mode, kind, blob = meta.decode().split()
            if kind != 'blob':
                raise ValueError('unsupported source tree')
            actual_tree.append((path.decode(), mode, blob))
    if actual_tree != [(entry['path'], entry['mode'], entry['gitBlob']) for entry in inventory]:
        raise ValueError('source commit inventory')
    exporter = Path(__file__).with_name('backend-metadata.rs')
    exporter_bytes = exporter.read_bytes()
    env = {'PATH': str(args.toolchain_bin.resolve(strict=True)) + ':/usr/bin:/bin',
           'CARGO_HOME': str(args.cargo_home.resolve(strict=True)),
           'RUSTUP_HOME': str(args.rustup_home.resolve(strict=True)),
           'CARGO_TARGET_DIR': str(args.target_dir.resolve(strict=True))}
    cargo = args.toolchain_bin.resolve(strict=True) / 'cargo'
    rustc = args.toolchain_bin.resolve(strict=True) / 'rustc'
    versions = {'cargo': subprocess.check_output([str(cargo), '--version'], env=env, timeout=15).decode(),
                'rustc': subprocess.check_output([str(rustc), '-Vv'], env=env, timeout=15).decode()}
    if not versions['cargo'].startswith('cargo 1.90.0 ') or not versions['rustc'].startswith('rustc 1.90.0 '):
        raise ValueError('metadata toolchain')
    args.output.mkdir(parents=True, exist_ok=False)
    with tempfile.TemporaryDirectory(prefix='weft-source-metadata-') as temporary:
        probe = Path(temporary)
        dependencies = {'weft-core': root / 'crates/weft-core', 'weft-databricks': root / 'crates/weft-databricks'}
        text = '[package]\nname="weft-distribution-source-metadata"\nversion="0.0.0"\nedition="2021"\n[workspace]\n[dependencies]\n'
        for name, path in dependencies.items():
            text += name + '={path=' + json.dumps(str(path)) + '}\n'
        text += 'serde_json={version="=1.0.145",features=["arbitrary_precision","raw_value"]}\n[patch.crates-io]\nahash={path=' + json.dumps(str(root / 'vendor/ahash')) + '}\n'
        (probe / 'Cargo.toml').write_text(text)
        lock_entry = next(entry for entry in inventory if entry['path'] == 'Cargo.lock')
        with (root / 'Cargo.lock').open('rb') as stream:
            lock_bytes = stream.read(SOURCE_FILE_LIMIT + 1)
        if len(lock_bytes) > SOURCE_FILE_LIMIT or len(lock_bytes) != lock_entry['bytes'] or sha(lock_bytes) != lock_entry['sha256']:
            raise ValueError('source lock custody')
        (probe / 'Cargo.lock').write_bytes(lock_bytes)
        (probe / 'src').mkdir()
        (probe / 'src/main.rs').write_bytes(exporter_bytes)
        command = [str(cargo), 'run', '-j1', '--offline', '--release']
        result = subprocess.run(command, cwd=probe, env=env, capture_output=True, timeout=180)
        (args.output / 'build.log').write_bytes(result.stderr[:LIMIT])
        if result.returncode != 0 or len(result.stdout) > LIMIT or len(result.stderr) > LIMIT:
            raise ValueError('metadata probe failed')
        metadata = json.loads(result.stdout, object_pairs_hook=unique)
        if metadata['backendId'] != 'ashlar.databricks' or metadata['backendVersion'] != '0.1.0-candidate':
            raise ValueError('source backend identity')
        closing_raw, closing_entries, closing_compressed = load_inventory(args.source_inventory)
        if verify_entries(root, closing_raw, closing_entries) != opening or exporter.read_bytes() != exporter_bytes or Path(__file__).read_bytes() != harness_bytes or closing_compressed != inventory_bytes:
            raise ValueError('metadata source drift')
        (args.output / 'backend-manifest.json').write_bytes(result.stdout)
        with (probe / 'Cargo.lock').open('rb') as stream:
            produced_lock = stream.read(SOURCE_FILE_LIMIT + 1)
        if len(produced_lock) > SOURCE_FILE_LIMIT:
            raise ValueError('probe lock limit')
        (args.output / 'probe-Cargo.lock').write_bytes(produced_lock)
        record = {'format': 'weft-source-backend-metadata/0.1', 'sourceCommit': args.source_commit,
                  'sourceInventoryDecodedSha256': opening[0], 'sourceInventoryDecodedBytes': opening[1],
                  'sourceTrackedFiles': opening[2], 'exporterSha256': sha(exporter_bytes), 'harnessSha256': sha(harness_bytes), 'sourceInventoryCompressedSha256': sha(inventory_bytes),
                  'metadataSha256': sha(result.stdout), 'metadataBytes': len(result.stdout),
                  'command': command, 'toolVersions': versions,
                  'producerInvocation': {'argv': [sys.executable, *sys.argv], 'workingDirectory': str(Path.cwd()),
                                         'sourceRoot': str(root), 'sourceRepository': str(args.source_repository.resolve(strict=True)),
                                         'sourceInventory': str(args.source_inventory.resolve(strict=True)), 'effectiveEnvironment': env},
                  'qualification': 'Public Registry validates source-owned Candidate metadata. This is not executable self-description, index admission, native support or a hermetic build.'}
        (args.output / 'custody.json').write_text(json.dumps(record, indent=2) + '\n')
        print(json.dumps({'state': 'source-metadata-observed', 'output': str(args.output), 'metadataSha256': record['metadataSha256']}))


if __name__ == '__main__':
    try:
        main()
    except (ValueError, KeyError, OSError, RuntimeError, subprocess.SubprocessError):
        print('WEFT-DISTRIBUTION-METADATA-REFUSED', file=sys.stderr)
        sys.exit(2)
