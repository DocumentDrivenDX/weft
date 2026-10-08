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
assert len(pairs) == 10
for path in pairs:
    composition = json.loads(path.read_text())
    assert composition['interfaceVersion'] == 'weft-original-conformance-composition/0.1.0'
    assert 'native' not in composition
    for selected in composition['records'] + composition['properties'] + composition.get('relationships', []):
        artifact(selected['inventory'])
    for prop in composition['properties']:
        for definition in list(prop['leafCodecs'].values()) + ([prop['rowJoin']] if prop['rowJoin'] is not None else []):
            document = json.loads(definition['originalJson'])
            for pointer, encoded in definition['originalArtifacts'].items():
                selected = document
                for part in pointer.split('/'):
                    selected = selected[int(part)] if isinstance(selected, list) else selected[part]
                assert raw(encoded) == artifact(selected)
        for presence in prop['recordPresence'].values():
            document = json.loads(presence['originalJson'])
            assert raw(presence['acceptedDefinitionBase64']) == artifact(document['acceptedDefinition'])
    for comparator in composition.get('comparators', {}).values():
        document = json.loads(comparator['originalJson'])
        for pointer, encoded in comparator['originalArtifacts'].items():
            selected = document
            for part in pointer.split('/'):
                selected = selected[int(part)] if isinstance(selected, list) else selected[part]
            assert raw(encoded) == artifact(selected)
    if path.name == 'original-relationship-composition.json':
        transports = json.loads(path.with_name('original-relationship-public-transport.json').read_text())
        assert len(transports) == 4
        assert {r['inverse'] for r in composition['relationships']} == {False, True}
        assert len(composition['comparators']) == 2
        for relation in composition['relationships']:
            names = [relation[k] for k in ['relationshipType','sourceId','sourceType','targetId','targetType']]
            assert len(set(names)) == 5
            assert all(relation['columns'][n]['relationIdentity'] == relation['relationIdentity'] for n in names)
    elif path.name.startswith('original-optional-'):
        home=path.name.split('-')[2]
        transports=[t for t in json.loads(path.with_name('original-optional-public-transport.json').read_text()) if json.loads(t['request']['target']['bindingJson'])['properties'][-1]['home']==home]
        assert len(transports)==3
    else:
        transports = [json.loads(path.with_name(path.name.replace('-composition.json', '-compile-transport.json')).read_text())]
    for transport in transports:
        assert transport['response']['status'] == 'compiled'
        assert transport['response']['qualification']['status'] == 'candidate'
        target = transport['request']['target']
        assert hashlib.sha256(target['bindingJson'].encode()).hexdigest() == target['bindingSha256']
        assert transport['response']['bindingSha256'] == target['bindingSha256']
        binding = json.loads(target['bindingJson'])
        for key, comparator in composition.get('comparators', {}).items():
            identity = json.loads(key)
            entity = next(e for e in binding['entities'] if e['logical'] == identity['owner'])
            prop = next(p for p in binding['properties'] if p['ownerTypeId'] == entity['typeId'] and p['logical'] == identity['field'])
            definition = json.loads(comparator['originalJson'])
            assert artifact(definition['valueDefinition']) == artifact(prop['valueDefinition'])
            assert artifact(definition['sourceDomainDefinition']) == artifact(prop['acceptedDefinition'])
print('10 compositions / 17 compiler responses preserve original byte custody.')
