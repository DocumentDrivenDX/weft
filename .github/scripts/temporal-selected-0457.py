"""Selected artifact evidence only; never publishes or grants native authority."""
from pathlib import Path
import hashlib
import json
import os
import platform
import subprocess
import sys
import time
import tomllib

SOURCE = 'd52644f4b2fb5579f9a303012cfc3a98ac6c174e'
UMF = 'e3f38ef6f05ae636d2837f9d69f8d6805222664b'
LOCK = '92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d'

def sha(path):
    with Path(path).open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()

def save(path, value):
    Path(path).write_text(json.dumps(value, indent=2) + '\n')

def check_identity(root, expected):
    def git(*args):
        return subprocess.check_output(['git', '-C', str(root), *args], text=True).strip()
    if git('rev-parse', 'HEAD') != SOURCE:
        raise ValueError('Wrong artifact source HEAD')
    if git('status', '--porcelain', '--untracked-files=all'):
        raise ValueError('Artifact source checkout is dirty')
    if sha(root / 'Cargo.lock') != LOCK:
        raise ValueError('Frozen lock differs')
    lock = tomllib.loads((root / 'Cargo.lock').read_text())
    actual = sorted((p['name'], p['version'], p['source']) for p in lock['package']
                    if p.get('source', '').startswith('git+'))
    wanted = sorted((p['name'], p['version'], p['sourceID']) for p in expected['lockGitPackages'])
    if actual != wanted:
        raise ValueError('Frozen lock package/source identities differ')
    for item in expected['resources']:
        if sha(root / item['path']) != item['sha256']:
            raise ValueError('Resource differs: ' + item['path'])
    project = tomllib.loads((root / 'crates/weft-python/pyproject.toml').read_text())
    crate = tomllib.loads((root / 'crates/weft-python/Cargo.toml').read_text())
    if project['project']['version'] != '0.4.57' or crate['package']['version'] != '0.4.57':
        raise ValueError('Distribution/crate version differs')
    return project['tool']['maturin']['features']

def main():
    root, control, out = (Path(x).resolve() for x in sys.argv[1:4])
    out.mkdir(parents=True, exist_ok=True)
    expected = json.loads((control / 'expected-source.json').read_text())
    features = check_identity(root, expected)
    corpus = control / 'rust-boundary-corpus.json'
    if sha(corpus) != expected['corpusSHA256']:
        raise ValueError('Retained native Rust oracle differs')
    if platform.system() != 'Linux' or platform.machine() != 'x86_64':
        raise ValueError('Selected producer is Linux x86_64 only')
    if os.environ.get('GITHUB_REPOSITORY') != 'DocumentDrivenDX/weft':
        raise ValueError('Wrong workflow repository')
    if os.environ.get('GITHUB_SHA') != os.environ.get('EXPECTED_CONTROL_SHA'):
        raise ValueError('Workflow control commit is not exact reviewed commit')
    if not os.environ.get('GITHUB_RUN_ID'):
        raise ValueError('Not a hosted admission')
    if os.environ.get('CARGO_BUILD_JOBS') != '1':
        raise ValueError('Selected workstream requires one Cargo worker')
    stages = []
    def run(name, argv, *, required=True):
        if sha(root / 'Cargo.lock') != LOCK:
            raise ValueError('Frozen lock drift')
        row = dict(name=name, argv=list(map(str, argv)), started=time.time())
        save(out / 'current-stage.json', row)
        with (out / (name + '.stdout')).open('wb') as stdout, (out / (name + '.stderr')).open('wb') as stderr:
            proc = subprocess.run(row['argv'], cwd=root, stdout=stdout, stderr=stderr)
        row.update(exitCode=proc.returncode, finished=time.time())
        stages.append(row)
        save(out / 'stages.json', stages)
        if required and proc.returncode:
            raise RuntimeError('Failed stage: ' + name)
        return proc.returncode
    save(out / 'inputs.json', dict(source=SOURCE, UMF=UMF, lockSHA256=LOCK,
        workflowCommit=os.environ['GITHUB_SHA'], runID=os.environ['GITHUB_RUN_ID'],
        runAttempt=os.environ.get('GITHUB_RUN_ATTEMPT'), platform=platform.platform(),
        wheelFeatures=features, corpusSHA256=sha(corpus),
        oracle='Retained 495 macOS Rust output; not Linux/native database execution',
        nativeDatabaseExecution=False, publication=False))
    if sys.argv[4:] == ['--preflight']:
        return 0
    if sys.argv[4:]:
        raise ValueError('Unknown operation')
    wheel, = (out / 'wheels').glob('*.whl')
    if 'manylinux' not in wheel.name or 'x86_64' not in wheel.name or '0.4.57' not in wheel.name:
        raise ValueError('Not the selected Linux wheel')
    container_metadata = json.loads((out / 'wheels/container-wheel-metadata.json').read_text())
    container_sources = sorted((p['name'], p['version'], p['source'])
        for p in container_metadata['packages'] if (p.get('source') or '').startswith('git+'))
    if container_sources != sorted((p['name'], p['version'], p['sourceID']) for p in expected['lockGitPackages']):
        raise ValueError('Actual container wheel source closure differs')
    venv = out / 'installed'
    run('venv', [sys.executable, '-m', 'venv', venv])
    python = str(venv / 'bin/python')
    run('normal-install', [python, '-I', '-m', 'pip', 'install', wheel])
    run('pip-check', [python, '-I', '-m', 'pip', 'check'])
    run('packaging', [python, '-I', root / 'scripts/release/verify-wheel-packaging.py',
        '--wheel', wheel, '--version', '0.4.57', '--distribution', 'umf-weft', '--output', out / 'packaging.json'])
    run('native-origin', [python, '-I', '-c', 'import weft.weft;print(weft.weft.__file__)'])
    native = Path((out / 'native-origin.stdout').read_text().strip())
    run('installed-temporal', [python, '-I', root / 'tests/engine-catalog/temporal-boundary.py', native, corpus])
    run('installed-routing', [python, '-I', root / 'tests/engine-catalog/temporal-routing-boundary.py', native])
    # Test-control checkout carries the reviewed smoke-only correction. Artifact
    # source stays SOURCE; report both identities, never relabel the built module.
    sdk_script = control.parents[2] / 'tests/engine-catalog/wheel-smoke.py'
    sdk_exit = run('reviewed-sdk-smoke', [python, '-I', sdk_script,
        '--wheel', wheel, '--output', out / 'sdk-smoke.json'], required=False)
    wasm_features = 'raw-postgresql-reads,' + ','.join('weft-runtime/' + x for x in features)
    for label, package, target, selected in [
        ('native', 'weft-runtime', 'x86_64-unknown-linux-gnu', 'raw-postgresql-reads'),
        ('Python', 'weft-python', 'x86_64-unknown-linux-gnu', 'raw-postgresql-reads'),
        ('wheel', 'weft-python', 'x86_64-unknown-linux-gnu', ','.join(features)),
        ('WASM', 'weft-wasm', 'wasm32-unknown-unknown', wasm_features)]:
        run(label + '-metadata', ['cargo', 'metadata', '--locked', '--format-version', '1',
            '--manifest-path', 'crates/' + package + '/Cargo.toml',
            '--filter-platform', target, '--features', selected])
        metadata = json.loads((out / (label + '-metadata.stdout')).read_text())
        sources = sorted((p['name'], p['version'], p['source']) for p in metadata['packages']
            if (p.get('source') or '').startswith('git+'))
        if sources != sorted((p['name'], p['version'], p['sourceID']) for p in expected['lockGitPackages']):
            raise ValueError('Actual target metadata source closure differs')
        by_identity = {(p['name'], p['version'], p.get('source')): p for p in metadata['packages']}
        for wanted in expected['packages']:
            actual = by_identity[(wanted['name'], wanted['version'], wanted['sourceID'])]
            if sha(actual['manifest_path']) != wanted['manifestSHA256']:
                raise ValueError('Consumed canonical manifest differs')
        save(out / (label + '-source-identities.json'), [dict(name=p['name'], version=p['version'],
            id=p['id'], source=p.get('source')) for p in metadata['packages']])
        run(label + '-features', ['cargo', 'tree', '--locked', '-p', package, '--target', target,
            '--features', selected, '--edges', 'normal,build,features', '--invert', 'serde_json@1.0.145',
            '--no-dedupe', '--format', '{p} features=[{f}]'])
    run('wasm-build', ['cargo', 'build', '--locked', '--release', '-p', 'weft-wasm',
        '--features', wasm_features, '--target', 'wasm32-unknown-unknown', '--message-format=json-render-diagnostics'])
    raw = Path(os.environ['CARGO_TARGET_DIR']) / 'wasm32-unknown-unknown/release/weft_wasm.wasm'
    for mode in ['web', 'nodejs']:
        run('bindgen-' + mode, ['wasm-bindgen', raw, '--target', mode, '--out-dir', out / mode])
    run('wasm-node', ['node', root / 'tests/engine-catalog/temporal-boundary.mjs', out / 'nodejs/weft_wasm.js', corpus])
    run('wasm-browser', ['node', root / 'tests/engine-catalog/temporal-browser.mjs', out / 'web', corpus])
    check_identity(root, expected)
    artifacts = [wheel, native, raw, out / 'web/weft_wasm_bg.wasm', out / 'nodejs/weft_wasm_bg.wasm']
    save(out / 'artifact-manifest.json', dict(source=SOURCE, canonicalUMF=UMF, lockSHA256=LOCK,
        workflowCommit=os.environ['GITHUB_SHA'], runID=os.environ['GITHUB_RUN_ID'],
        wheelTarget='x86_64-unknown-linux-gnu', wasmTarget='wasm32-unknown-unknown',
        wheelFeatures=features, wasmFeatures=wasm_features,
        artifacts=[dict(path=str(p), bytes=p.stat().st_size, sha256=sha(p)) for p in artifacts],
        allStagesPassed=all(x['exitCode'] == 0 for x in stages), SDKSmokeExit=sdk_exit,
        nativeDatabaseExecution=False, HF62SecuritySnapshot='OPEN', publication=False))
    return int(sdk_exit != 0)

if __name__ == '__main__':
    sys.exit(main())
