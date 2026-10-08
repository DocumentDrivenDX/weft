"""Audit actual qualified wheel/native payload, ABI record and WASM/glue custody.
@covers US-005-AC1 @covers US-005-AC2 @covers US-005-AC4
"""
import base64,csv,hashlib,io,json,os,zipfile
from pathlib import Path
OUT=Path(os.environ.get('WEFT_QUALIFIED_HOST_OUT','/private/tmp/weft-b007-qualified-hosts'))
wheel=Path('/private/tmp/weft-b007-qualified-wheels/weft_sql-0.1.0-cp39-abi3-macosx_11_0_arm64.whl')
sha=lambda b:hashlib.sha256(b).hexdigest()
with zipfile.ZipFile(wheel) as archive:
 names=archive.namelist();assert len(names)==len(set(names))==5,names
 assert not any(n.endswith(('.js','.wasm')) for n in names)
 record=next(n for n in names if n.endswith('/RECORD'))
 rows=list(csv.reader(io.StringIO(archive.read(record).decode())));assert {r[0] for r in rows}==set(names)
 for name,digest,size in rows:
  data=archive.read(name)
  if name==record:assert digest==size==''
  else:assert digest=='sha256='+base64.urlsafe_b64encode(hashlib.sha256(data).digest()).decode().rstrip('=') and size==str(len(data)),name
 assert archive.read('weft/__init__.py').decode().strip()=='from .weft import *\n\n__doc__ = weft.__doc__\nif hasattr(weft, "__all__"):\n    __all__ = weft.__all__'
 native=archive.read('weft/weft.abi3.so');nativeSha=sha(native)
 assert native==Path('/private/tmp/weft-b007-qualified-python-package/weft/weft.abi3.so').read_bytes()
 assert 'Tag: cp39-abi3-macosx_11_0_arm64' in archive.read(next(n for n in names if n.endswith('/WHEEL'))).decode()
wasm=Path('/private/tmp/weft-b007-qualified-web/weft_wasm_bg.wasm');glue=Path('/private/tmp/weft-b007-qualified-web/weft_wasm.js');wrapper=Path('/private/tmp/weft-b007-qualified-wrapper/index.js')
assert wasm.read_bytes().startswith(b'\0asm')
python=json.loads((OUT/'python-summary.json').read_text());browser=json.loads((OUT/'browser-summary.json').read_text())
assert nativeSha==python['extensionSha256'] and sha(wasm.read_bytes())==browser['wasmSha256']
summary=dict(status='passed',wheelSha256=sha(wheel.read_bytes()),members=names,recordEntriesVerified=len(rows),nativeExtensionSha256=nativeSha,wasmSha256=sha(wasm.read_bytes()),jsGlueSha256=sha(glue.read_bytes()),wrapperSha256=sha(wrapper.read_bytes()),features=['truss-postgresql-qualified','ashlar-databricks-qualified'],rust='1.90.0',maturin='1.9.6',wasmBindgen='0.2.105',scope='Actual local ABI3 CPython package and qualified web build content verified against loaded native payloads. No JavaScript/WASM wheel sidecar, distribution, signing or platform-wide qualification.')
(OUT/'package-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
