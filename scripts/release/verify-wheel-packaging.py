"""Exact installed ABI3 wheel RECORD, metadata and bundled notices."""
import argparse,base64,csv,hashlib,importlib.metadata,io,json,sys,zipfile
from pathlib import Path
if sys.flags.optimize:raise RuntimeError('Packaging checks require assertions')
p=argparse.ArgumentParser();p.add_argument('--wheel',type=Path,required=True);p.add_argument('--version',required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args()
sha=lambda raw:hashlib.sha256(raw).hexdigest()
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
    assert 'Name: weft-sql\n' in metadata and 'Version: '+a.version+'\n' in metadata
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
assert importlib.metadata.version('weft-sql')==weft.__version__==a.version
assert sha(Path(extension.__file__).read_bytes())==native_sha
with zipfile.ZipFile(a.wheel)as archive:assert Path(weft.__file__).read_bytes()==archive.read('weft/__init__.py')
a.output.parent.mkdir(parents=True,exist_ok=True)
a.output.write_text(json.dumps({'status':'passed','version':a.version,'wheel':a.wheel.name,'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':native_sha,'python':sys.version,'scope':'exact installed archive, RECORD, metadata, ABI tag and notices; native semantics separately gated'},indent=2)+'\n')
