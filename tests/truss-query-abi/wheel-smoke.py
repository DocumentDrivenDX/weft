"""Exact installed Weft wheel compiler checks; no database admission claim."""
from copy import deepcopy
from hashlib import sha256
from importlib import import_module, metadata, machinery
import json
from pathlib import Path

import weft

ROOT = Path(__file__).resolve().parent
EXPECTED_VERSION = '0.4.35'
assert metadata.version('weft-sql') == weft.__version__ == EXPECTED_VERSION
native = import_module('weft.weft')
package_root = Path(weft.__file__).resolve().parent
assert package_root == Path(metadata.distribution('weft-sql').locate_file('weft')).resolve()
assert Path(native.__file__).resolve().is_relative_to(package_root)
assert any(str(native.__file__).endswith(suffix) for suffix in machinery.EXTENSION_SUFFIXES)
assert callable(weft.compile_truss_runtime_json)
custody = json.loads((ROOT / 'custody.json').read_bytes())
for name, digest in custody['originalFiles'].items():
    assert sha256((ROOT / 'fixtures' / name).read_bytes()).hexdigest() == digest
bundle = json.loads((ROOT / 'fixtures/native-query-bundle.json').read_bytes())
assert sha256(bundle['bindingJson'].encode()).hexdigest() == custody['bindingSha256']
abi = json.loads((ROOT / 'fixtures/query-abi.json').read_bytes())
assert json.loads(bundle['bindingJson'])['queryAbi'] == {
    'identity': abi['identity'], 'version': abi['version'],
    'sha256': sha256((ROOT / 'fixtures/query-abi.json').read_bytes()).hexdigest(),
}
controls = []


def request(sql, source=bundle, version='0.4.38', target='truss-runtime-pg18.6/0.5'):
    raw = source['bindingJson']
    return {
        'interfaceVersion': 'weft-compile/0.2.6', 'dialect': 'weft-sql/0.2.0',
        'timestampComparisonProfile': 'umf-rfc3339-exact-instant/1.0.0',
        'modules': source['modules'], 'sql': sql, 'parameters': {}, 'exactParameters': {},
        'options': {'allowCandidate': True},
        'target': {'backendId': 'truss.postgresql.runtime', 'backendVersion': version,
                   'targetProfile': target, 'bindingJson': raw,
                   'bindingSha256': sha256(raw.encode()).hexdigest()},
    }


def alter(original, change):
    selected = deepcopy(original)
    binding = json.loads(selected['target']['bindingJson'])
    change(binding)
    # Deliberate whitespace proves original bytes, rather than normalized JSON,
    # remain the native parameter.
    raw = json.dumps(binding, ensure_ascii=False, indent=2) + '\n'
    selected['target'].update(bindingJson=raw, bindingSha256=sha256(raw.encode()).hexdigest())
    return selected


def run(label, selected, status):
    output = json.loads(weft.compile_truss_runtime_json(json.dumps(selected)))
    assert output['status'] == status, (label, output)
    if status == 'blocked':
        assert 'sql' not in output, (label, output)
    else:
        assert output['backend']['backendVersion'] == selected['target']['backendVersion']
        assert output['backend']['targetProfile'] == selected['target']['targetProfile']
        assert output['bindingSha256'] == selected['target']['bindingSha256']
    controls.append(label)
    return output


for label, sql in (
    ('count', 'SELECT COUNT(*) FROM "A" a'),
    ('complete-entity', 'SELECT a.* FROM "A" a'),
    ('owned-related-keys', 'SELECT RELATED_KEYS(a.children, 4) FROM "A" a'),
    ('independent-related-keys', 'SELECT RELATED_KEYS(a."A-B", 2) FROM "A" a'),
):
    selected = request(sql)
    if label == 'complete-entity':
        selected['readProfile'] = {'version': 'weft-truss-entity-read/0.3.0', 'subset': 'complete-entity'}
    run(label, selected, 'compiled')

legacy = json.loads((ROOT.parent.parent / 'docs/helix/04-build/evidence/'
                     'mutation-current-truss-20261014/native-query-bundle.json').read_bytes())
legacy_request = request('SELECT COUNT(*) FROM products p', legacy, '0.4.35',
                         'truss-runtime-pg18.6/0.4')
run('original-legacy-positive', legacy_request, 'compiled')
for version in ('0.4.35', '0.4.36', '0.4.37'):
    run('new-binding-refused-by-' + version,
        request('SELECT COUNT(*) FROM "A" a', version=version,
                target='truss-runtime-pg18.6/0.4'), 'blocked')
# Use the genuine positive predecessor so these refusals specifically exercise
# forbidden raw ABI fields, independent of its already-valid release pin.
for label, pin in (('null', None), ('object', {})):
    run('legacy-raw-abi-' + label, alter(legacy_request,
        lambda binding, pin=pin: binding.update(queryAbi=pin)), 'blocked')

for field in ('absent', 'null', 'identity', 'version', 'sha256', 'extra'):
    def change(binding):
        if field == 'absent':
            binding.pop('queryAbi')
        elif field == 'null':
            binding['queryAbi'] = None
        elif field == 'extra':
            binding['queryAbi']['extra'] = True
        else:
            binding['queryAbi'][field] = 'changed'
    run('abi-' + field, alter(request('SELECT COUNT(*) FROM "A" a'), change), 'blocked')

changed = alter(request('SELECT COUNT(*) FROM "A" a'), lambda binding: binding.update(
    releaseId='truss-catalog-runtime/0.70-candidate', releaseSha256='ab' * 32))
output = run('changed-runtime-forwarded-exactly', changed, 'compiled')
assert any(parameter.get('value') == changed['target']['bindingJson']
           for parameter in output['parameters'])
assert 'query_objects' in output['sql']
assert any(obligation['id'] == 'truss.runtime.context/0.1' for obligation in output['obligations'])
print(json.dumps({'wheelVersion': EXPECTED_VERSION, 'nativeModuleSha256': sha256(
    Path(native.__file__).read_bytes()).hexdigest(), 'compilerControls': len(controls),
    'controls': controls, 'scope': 'installed compiler only; no native database admission'}))
