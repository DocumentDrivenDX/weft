"""Independent byte-custody check of conformance embedding composition captures."""
import base64, hashlib, json
from pathlib import Path

def raw(encoded):
    value = base64.b64decode(encoded, validate=True)
    assert base64.b64encode(value).decode() == encoded
    return value

def artifact(value):
    data = raw(value['bytesBase64'])
    assert hashlib.sha256(data).hexdigest() == value['sha256']
    return data

pairs = sorted(Path('tests/truss-postgresql/fixtures').glob('original-*-composition.json'))
assert len(pairs) == 7
for path in pairs:
    composition = json.loads(path.read_text())
    assert composition['interfaceVersion'] == 'weft-original-conformance-composition/0.1.0'
    assert 'native' not in composition
    for selected in composition['records'] + composition['properties']:
        artifact(selected['inventory'])
    for prop in composition['properties']:
        for definition in list(prop['leafCodecs'].values()) + [prop['rowJoin']]:
            document = json.loads(definition['originalJson'])
            for pointer, encoded in definition['originalArtifacts'].items():
                selected = document
                for part in pointer.split('/'):
                    selected = selected[int(part)] if isinstance(selected, list) else selected[part]
                assert raw(encoded) == artifact(selected)
        for presence in prop['recordPresence'].values():
            document = json.loads(presence['originalJson'])
            assert raw(presence['acceptedDefinitionBase64']) == artifact(document['acceptedDefinition'])
    transport = json.loads(path.with_name(path.name.replace('-composition.json', '-compile-transport.json')).read_text())
    assert transport['response']['status'] == 'compiled'
    assert transport['response']['qualification']['status'] == 'candidate'
    target = transport['request']['target']
    assert hashlib.sha256(target['bindingJson'].encode()).hexdigest() == target['bindingSha256']
    assert transport['response']['bindingSha256'] == target['bindingSha256']
print('7 composition/transport pairs preserve original byte custody.')
