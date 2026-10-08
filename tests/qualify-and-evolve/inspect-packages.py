"""Inspect local test wheels and browser manifest; never authorizes distribution."""
import base64,csv,hashlib,io,json,pathlib,zipfile
ROOT=pathlib.Path(__file__).resolve().parents[2]
reports=[]
for feature,directory,receipt in [('test-third','weft-b007-public-wheels','B-007-public-python/summary.json'),('truss-postgresql-candidate','weft-b007-truss-wheels','B-007-truss-embeddings/python-summary.json')]:
 path=pathlib.Path('/private/tmp')/directory/'weft_sql-0.1.0-cp39-abi3-macosx_11_0_arm64.whl'
 with zipfile.ZipFile(path) as z:
  names=z.namelist();assert len(names)==len(set(names))
  assert all(not pathlib.PurePosixPath(n).is_absolute() and '..' not in pathlib.PurePosixPath(n).parts for n in names)
  native=[n for n in names if n.endswith('.so')];assert len(native)==1
  data=z.read(native[0]);assert data[:4] in [bytes.fromhex('cffaedfe'),bytes.fromhex('feedfacf')]
  expected=json.loads((ROOT/'docs/helix/04-build/evidence'/receipt).read_text())['extensionSha256']
  assert hashlib.sha256(data).hexdigest()==expected
  record=next(n for n in names if n.endswith('.dist-info/RECORD'))
  rows=list(csv.reader(io.StringIO(z.read(record).decode())));assert {r[0] for r in rows}==set(names)
  for name,digest,size in rows:
   if name==record:assert digest==size=='';continue
   raw=z.read(name);assert int(size)==len(raw) and digest=='sha256='+base64.urlsafe_b64encode(hashlib.sha256(raw).digest()).decode().rstrip('=')
  metadata=z.read(next(n for n in names if n.endswith('.dist-info/METADATA'))).decode()
  wheel=z.read(next(n for n in names if n.endswith('.dist-info/WHEEL'))).decode()
  assert 'Name: weft-sql\n' in metadata and 'Version: 0.1.0\n' in metadata and 'Requires-Python: >=3.9\n' in metadata
  assert 'Tag: cp39-abi3-macosx_11_0_arm64\n' in wheel and 'Root-Is-Purelib: false\n' in wheel
  assert not any(n.endswith(('.js','.wasm')) for n in names)
  reports.append({'buildFeature':feature,'wheelSha256':hashlib.sha256(path.read_bytes()).hexdigest(),'files':names,'nativeExtensionSha256':expected,'recordVerified':True,'tag':'cp39-abi3-macosx_11_0_arm64','licenseMetadataPresent':any(l.startswith(('License:','License-Expression:','License-File:')) for l in metadata.splitlines())})
manifest=ROOT/'packages/weft-browser/package.json';browser=json.loads(manifest.read_text())
report={'status':'passed','inspectionScope':'Local test artifacts only; ABI tags do not prove every Python/OS release. No registry or distribution action.','wheels':reports,'browser':{'manifestSha256':hashlib.sha256(manifest.read_bytes()).hexdigest(),'manifest':browser,'files':[str(p.relative_to(ROOT)) for p in manifest.parent.rglob('*') if p.is_file()]},'sourceSha256':hashlib.sha256(pathlib.Path(__file__).read_bytes()).hexdigest()}
OUT=ROOT/'docs/helix/04-build/evidence/B-007-package-inspection';OUT.mkdir(exist_ok=True)
(OUT/'summary.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'status':'passed','wheels':len(reports),'licensedWheels':sum(r['licenseMetadataPresent'] for r in reports),'releaseQualified':False}))
