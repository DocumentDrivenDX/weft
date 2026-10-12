"""Basic native Python transport check; does not qualify a database runtime."""
import json
import subprocess
from pathlib import Path
import weft

root = Path(__file__).resolve().parents[2]
cut = json.loads((root / 'tests/truss-postgresql/fixtures/original-boolean-inputs.json').read_text())[1]
composition = cut['composition']
composition['interfaceVersion'] = 'weft-truss-pg18-composition/0.1.0'
for prop in composition['properties']:
    prop['nativeTree'] = False
request = cut['requests'][1]['request']
request['target']['backendVersion'] = '0.2.0'
request['target']['targetProfile'] = 'truss-pg18.6-live/0.1'

def forbidden(*args, **kwargs):
    raise AssertionError('Python compilation attempted a subprocess')
subprocess.Popen = subprocess.run = subprocess.check_output = forbidden
response = json.loads(weft.compile_truss_postgresql_json(json.dumps(request), json.dumps(composition)))
assert response['status'] == 'compiled', response
assert '$1' in response['sql']
assert not hasattr(weft, 'compile_json_with_conformance_configuration')
request.pop('readProfile', None)
request['sql'] = 'SELECT COUNT(*) AS total FROM Customer c WHERE c.id = :flag'
count = json.loads(weft.compile_truss_postgresql_json(json.dumps(request), json.dumps(composition)))
assert count['status'] == 'compiled', count
assert 'COUNT(' in count['sql'].upper()
request['target']['targetProfile'] = 'pg17.9-candidate'
refused = json.loads(weft.compile_truss_postgresql_json(json.dumps(request), json.dumps(composition)))
assert refused['status'] == 'blocked' and 'sql' not in refused
print('PG18 original scalar filter compiled through native Python; old target refused; no conformance export or subprocess')
