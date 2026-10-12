"""Validate an installed release wheel, its RECORD, ABI and complete corpus.
Run from repository root, outside the source package, using a fresh venv.
"""
import argparse, base64, csv, gzip, hashlib, importlib.metadata, io, json
import os, platform, subprocess, sys, zipfile
from pathlib import Path
if sys.flags.optimize:
    raise RuntimeError('Do not disable validation assertions')
p = argparse.ArgumentParser()
p.add_argument('--wheel', required=True, type=Path)
p.add_argument('--out', required=True, type=Path)
p.add_argument('--corpus', type=Path, default=Path('docs/helix/04-build/evidence/main-ed2-integration-20261010/baseline/main-reference-artifacts.jsonl.gz'))
a = p.parse_args()
sha = lambda b: hashlib.sha256(b).hexdigest()
with zipfile.ZipFile(a.wheel) as z:
    names = z.namelist()
    assert len(names) == len(set(names))
    assert all(not n.startswith('/') and '..' not in Path(n).parts for n in names)
    record = next(n for n in names if n.endswith('.dist-info/RECORD'))
    rows = list(csv.reader(io.StringIO(z.read(record).decode())))
    assert {r[0] for r in rows} == set(names)
    for name, digest, size in rows:
        if name == record:
            assert digest == size == ''
        else:
            raw = z.read(name)
            assert digest == 'sha256=' + base64.urlsafe_b64encode(hashlib.sha256(raw).digest()).decode().rstrip('=')
            assert size == str(len(raw))
    metadata = z.read(next(n for n in names if n.endswith('.dist-info/METADATA'))).decode()
    assert 'Name: weft-sql\n' in metadata and 'Version: 0.4.2\n' in metadata
    assert 'License-Expression: MIT OR Apache-2.0\n' in metadata
    assert 'Requires-Python: >=3.9\n' in metadata
    assert 'Requires-Dist:' not in metadata
    assert sum(n.endswith('/LICENSE-MIT') for n in names) == 1
    assert sum(n.endswith('/LICENSE-APACHE') for n in names) == 1
    assert sum(n.endswith('/THIRD-PARTY-LICENSES.txt') for n in names) == 1
    assert all(not n.endswith(('.js', '.wasm')) for n in names)
    native = [n for n in names if n.endswith(('.so', '.pyd'))]
    assert len(native) == 1
    wheel_metadata = z.read(next(n for n in names if n.endswith('.dist-info/WHEEL'))).decode()
    assert 'Root-Is-Purelib: false\n' in wheel_metadata
    assert 'Tag: cp39-abi3-' in wheel_metadata
    native_sha = sha(z.read(native[0]))
import weft
import weft.weft as extension
assert importlib.metadata.version('weft-sql') == weft.__version__ == '0.4.2'
assert sha(Path(extension.__file__).read_bytes()) == native_sha
assert not hasattr(weft, 'compile_json_with_conformance_configuration')
assert hasattr(weft, 'compile_paths_keys_json')
assert hasattr(weft, 'compile_implicit_arithmetic_json')
# Compilation must run without a JavaScript runtime or subprocess.
platform_info = platform.platform()
machine_info = platform.machine()
os.environ['PATH'] = ''
def forbidden(*args, **kwargs):
    raise AssertionError('Native compile attempted subprocess')
subprocess.Popen = subprocess.run = subprocess.check_output = forbidden
canonical = lambda x: json.dumps(x, sort_keys=True, ensure_ascii=False, separators=(',', ':'), allow_nan=False)
records = [json.loads(l) for l in gzip.decompress(a.corpus.read_bytes()).decode().splitlines()]
assert len(records) == len({r['id'] for r in records}) == 2181
receipts = []
for row in records:
    raw = json.dumps(row['request'], ensure_ascii=False)
    actual = weft.compile_json(raw)
    assert canonical(json.loads(actual)) == canonical(row['response']), row['id']
    assert weft.compile_json(raw) == actual
    receipts.append({'id': row['id'], 'responseSha256': sha(actual.encode())})
candidate_receipts = []
for realization, api in [
    ('weft-5ddcebd-count-star-aarch64-apple-darwin-candidate', weft.compile_paths_keys_json),
    ('weft-f36f206-implicit-arithmetic-aarch64-apple-darwin-candidate', weft.compile_implicit_arithmetic_json),
]:
    base = Path('distributions/realizations')/realization/'evidence'
    for filename in ['corpus.jsonl.gz','controls.jsonl.gz']:
        for row in map(json.loads, gzip.decompress((base/filename).read_bytes()).decode().splitlines()):
            request = bytes.fromhex(row['requestHex']).decode()
            expected = bytes.fromhex(row['responseHex']).decode()
            actual = api(request)
            assert canonical(json.loads(actual)) == canonical(json.loads(expected)), (realization,filename,row['id'])
            assert api(request) == actual
            candidate_receipts.append({'realization':realization,'id':row['id'],'responseSha256':sha(actual.encode())})
for value in [None, {}, 1, b'{}']:
    try:
        weft.compile_json(value)
    except TypeError:
        pass
    else:
        raise AssertionError('Invalid transport type accepted')
try:
    weft.compile_json('\ud800')
except UnicodeError:
    pass
else:
    raise AssertionError('Invalid Unicode accepted')
controls = [('duplicate', '{"x":1,"x":2}', 'WFT-JSON-DUPLICATE'), ('truncated','{"x":[1,','WFT-INPUT'), ('request-over', ' '*(16*1024*1024+1), 'WFT-LIMIT')]
for name, raw, code in controls:
    result = json.loads(weft.compile_json(raw))
    assert result['status'] == 'blocked' and result['diagnostics'][0]['code'] == code, (name,result)
    assert not set(result) & {'sql','parameters','logicalPlan','columns','obligations'}
a.out.parent.mkdir(parents=True, exist_ok=True)
summary = {'status':'passed','wheel':a.wheel.name,'wheelSha256':sha(a.wheel.read_bytes()),'nativeExtensionSha256':native_sha,'python':sys.version,'platform':platform_info,'machine':machine_info,'cases':len(receipts),'candidateCases':len(candidate_receipts),'candidateReceiptsSha256':sha(json.dumps(candidate_receipts,sort_keys=True).encode()),'controls':len(controls)+5,'subprocessDisabled':True,'corpusSha256':sha(a.corpus.read_bytes()),'receiptsSha256':sha(json.dumps(receipts,sort_keys=True).encode()),'scope':'Installed-wheel compatibility with retained main corpus. No new native database qualification; 53 changed outputs retain the recorded native-requalification gap.'}
a.out.write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary))
