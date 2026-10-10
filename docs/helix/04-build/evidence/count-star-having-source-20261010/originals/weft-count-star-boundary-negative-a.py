import importlib.util,json,subprocess,copy
from pathlib import Path
p=Path('scripts/checks/check-module-boundaries.py');s=importlib.util.spec_from_file_location('boundary_owner',p);m=importlib.util.module_from_spec(s);s.loader.exec_module(m)
metadata=json.loads(subprocess.check_output(['cargo','metadata','--no-deps','--format-version','1','--locked','--offline']));policy=json.loads(Path('scripts/checks/module-boundaries.json').read_text());m.check(metadata,policy)
modified=copy.deepcopy(metadata);core=next(x for x in modified['packages']if x['name']=='weft-core');backend=next(x for x in modified['packages']if x['name']=='weft-databricks');core['dependencies'].append({'name':'weft-databricks','path':str(Path(backend['manifest_path']).parent)})
try:m.check(modified,policy)
except ValueError as e:assert str(e)=='Forbidden Cargo dependency: weft-core -> weft-databricks'
else:raise AssertionError('forbidden edge admitted')
print('Actual metadata allowed; forbidden backend edge from core refused. No source manifest mutation.')
