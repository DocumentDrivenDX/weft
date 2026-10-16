from pathlib import Path
import zipfile,hashlib,json,sys
profile=sys.argv[1];assert profile in ['old-presence','default'];wheel=next(Path('/private/tmp/weft-model-requirements-'+profile+'-wheels-20261016').glob('*.whl'));dest=Path('/private/tmp/weft-model-requirements-'+profile+'-installed-20261016');dest.mkdir(exist_ok=False)
features=['truss-postgresql-pg18','current-postgresql-scalar-candidate','current-databricks-scalar-candidate','current-query-refinement-candidate']
if profile=='old-presence':features+=['bounded-current-string-candidate','current-presence-candidate']
with zipfile.ZipFile(wheel) as z:
 assert all(not Path(n).is_absolute() and '..' not in Path(n).parts for n in z.namelist());z.extractall(dest)
 entries={n:{'bytes':len(z.read(n)),'sha256':hashlib.sha256(z.read(n)).hexdigest()} for n in z.namelist() if not n.endswith('/')}
Path('/private/tmp/weft-model-requirements-'+profile+'-wheel-20261016.json').write_text(json.dumps({'wheel':str(wheel),'wheelSha256':hashlib.sha256(wheel.read_bytes()).hexdigest(),'installed':str(dest),'entries':entries,'features':features,'published':False},indent=2)+'\n');print('Installed exact local '+profile+' wheel')
