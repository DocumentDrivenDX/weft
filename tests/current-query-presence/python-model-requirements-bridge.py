# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual native Python requirements bridge; original data fixtures, no SQL/admission."""
import copy
import gc
import hashlib
import json
import pickle
import sys
from pathlib import Path

sys.path.insert(0, sys.argv[1])
import weft

mode = sys.argv[2]
root = Path(__file__).resolve().parents[2]
checks = []

def encode(value):
    return json.dumps(value, ensure_ascii=False, sort_keys=True, separators=(',', ':'))

def digest(value):
    return hashlib.sha256(value.encode()).hexdigest()

def ok(name, condition):
    assert condition, name
    checks.append(name)

def refuses(name, call):
    try:
        call()
    except (TypeError, ValueError, AttributeError) as error:
        checks.append(name)
        return str(error)
    raise AssertionError(name)

def request():
    return json.loads((root / 'tests/current-query-presence/fixtures/unadmitted-context.request.json').read_text())

def source(value, role):
    return next(s for s in value['opaqueSources'] if s['role'] == role)

def change_profile(value, change):
    selected = source(value, 'presence-designation')
    profile = json.loads(selected['utf8'])
    change(profile)
    selected['utf8'] = encode(profile)
    selected['sha256'] = digest(selected['utf8'])

def set_ontology(value, raw):
    selected = source(value, 'ontology')
    selected['utf8'] = raw
    selected['sha256'] = digest(raw)
    def update(profile):
        next(s for s in profile['sources'] if s['role'] == 'ontology')['sha256'] = selected['sha256']
    change_profile(value, update)

def change_model(value, change):
    module = value['modules'][0]
    document = json.loads(module['documentJson'])
    change(document['modules'][0]['elements'])
    module['documentJson'] = encode(document)
    module['pin']['sha256'] = digest(module['documentJson'])
    change_profile(value, lambda p: p['modelPins'].__setitem__(0, copy.deepcopy(module['pin'])))

if mode == 'default':
    ok('default class absent', not hasattr(weft, 'CurrentPresencePreparation'))
    ok('default factory absent', not hasattr(weft.ApplicationLineagePreparation, 'prepare_current_presence'))
elif mode == 'old-presence':
    ok('old presence class present', hasattr(weft, 'CurrentPresencePreparation'))
    ok('new method absent on old class', not hasattr(weft.CurrentPresencePreparation, 'model_requirements_json'))
    child = weft.prepare_application_lineage(encode(request())).prepare_current_presence()
    ok('new method absent on old instance', not hasattr(child, 'model_requirements_json'))
else:
    ok('new method present', hasattr(weft.CurrentPresencePreparation, 'model_requirements_json'))
    raw = encode(request())
    parent = weft.prepare_application_lineage(raw)
    child = parent.prepare_current_presence()
    other_parent = weft.prepare_application_lineage(raw)
    other_child = other_parent.prepare_current_presence()
    report_raw = child.model_requirements_json()
    report = json.loads(report_raw)
    ok('same actual Python parent', child.source_preparation() is parent)
    ok('equal source other parent distinct', other_child.source_preparation() is other_parent and other_parent is not parent)
    ok('equal bytes do not replace actual custody', other_child.model_requirements_json() == report_raw and child.source_preparation() is not other_parent)
    ok('exact original request', report['sourceRequestJson'] == raw)
    ok('exact original source hash', report['sourceRequestSha256'] == digest(raw))
    ok('actual preparation hash', report['preparationSha256'] == digest(child.json()))
    ok('exact original pins', report['modelPins'] == [m['pin'] for m in request()['modules']])
    ok('all authority flags false', report['complete'] is False and report['authority'] is False and report['securityReady'] is False and report['nativeAdmission'] == 'required')
    expected = {
        'Staff': (['staffId'], ['staffId']),
        'Project': (['projectId'], ['projectId']),
        'Resource': (['resourceId', 'salary', 'token'], ['resourceId']),
        'Ownership': (['ownerId', 'ownerResource', 'ownerProject'], ['ownerId']),
        'Assignment': (['assignmentId', 'assignmentStaff', 'assignmentProject', 'active'], ['assignmentId']),
    }
    actual = {r['record']['element']: ([f['field']['element'] for f in r['fields']], [f['element'] for f in r['selectedKey']['fields']]) for r in report['records']}
    ok('independent complete five-Record mapping', actual == expected)
    for record in report['records']:
        ok('original complete Record identity ' + record['record']['element'], record['record'] == {'documentId': 'domain', 'revision': 'query-presence-core-r1', 'module': 'm', 'element': record['record']['element']})
        for field in record['fields']:
            ok('root descriptor original identity ' + field['field']['element'], field['descriptors'][0]['identity'] == field['field'])
    resource = next(r for r in report['records'] if r['record']['element'] == 'Resource')
    token = next(f for f in resource['fields'] if f['field']['element'] == 'token')
    ok('optional availability retained', token['descriptors'][0]['availability'] == 'absent-allowed')
    original_id = id(parent)
    del parent
    gc.collect()
    ok('actual parent survives local drop', id(child.source_preparation()) == original_id)
    ok('requirements stable after parent local drop', child.model_requirements_json() == report_raw)
    for label, action in [
        ('no report constructor', lambda: weft.CurrentPresencePreparation(report_raw)),
        ('no dictionary constructor', lambda: weft.CurrentPresencePreparation(report)),
        ('no copying capability', lambda: copy.copy(child)),
        ('no pickle reconstruction', lambda: pickle.dumps(child)),
        ('no subclass constructor', lambda: type('Imported', (weft.CurrentPresencePreparation,), {})),
        ('frozen original field', lambda: setattr(child, 'original', other_parent)),
    ]:
        refuses(label, action)
    legacy_raw = (root / 'tests/current-query-designation/fixtures/disclosed-token-equality.request.json').read_text()
    legacy = weft.prepare_application_lineage(legacy_raw).prepare_current_query()
    ok('legacy object has no new method', not hasattr(legacy, 'model_requirements_json'))
    calls = []
    class Trap:
        def __str__(self):
            calls.append('str')
            raise AssertionError('no coercion')
        def __len__(self):
            calls.append('len')
            raise AssertionError('no inspection')
    for receiver in [None, {}, report_raw, report, other_parent, legacy, Trap()]:
        refuses('wrong receiver cannot import issued context', lambda receiver=receiver: weft.CurrentPresencePreparation.model_requirements_json(receiver))
    refuses('additional argument refused', lambda: child.model_requirements_json(Trap()))
    ok('no caller coercion callbacks', calls == [])
    ok('old compiler does not import requirements', json.loads(weft.compile_json(report_raw)).get('status') != 'compiled')
    # Exact unknown numeric and Unicode source bytes never pass through float.
    value = request()
    ontology_raw = source(value, 'ontology')['utf8']
    extended = ontology_raw.rstrip()[:-1] + ',"x-original-number":9007199254740993.000000000000000001,"x-original-unicode":"雪"}'
    set_ontology(value, extended)
    changed_raw = encode(value)
    extended_child = weft.prepare_application_lineage(changed_raw).prepare_current_presence()
    extended_report = json.loads(extended_child.model_requirements_json())
    ok('exact opaque numeric Unicode bytes retained', extended_report['sourceRequestJson'] == changed_raw and source(json.loads(extended_report['sourceRequestJson']), 'ontology')['utf8'] == extended)
    ok('independent binding unchanged for changed context', source(value, 'native-binding') == source(request(), 'native-binding'))
    ok('context-specific hashes change independently', extended_report['preparationSha256'] != report['preparationSha256'])
    # A new authored fixture exercises original descriptor graph identities.
    # It is data only, not a repaired/admitted full UMF policy/catalog source.
    nested = request()
    def add_nested(elements):
        selected = next(e for e in elements if e['id'] == 'Resource')
        selected['members'].append({'module': 'm', 'element': 'details'})
        elements.extend([
            {'id': 'details', 'name': 'details', 'kind': 'field', 'cardinality': 'one', 'nullability': 'absent-allowed', 'references': [{'module': 'm', 'element': 'Details', 'role': 'record-type'}], 'extensions': {}},
            {'id': 'Details', 'name': 'Details', 'kind': 'record', 'members': [{'module': 'm', 'element': 'detailText'}], 'extensions': {}},
            {'id': 'detailText', 'name': 'detailText', 'kind': 'field', 'scalarType': 'string', 'cardinality': 'one', 'nullability': 'required', 'extensions': {}},
        ])
    change_model(nested, add_nested)
    ontology = json.loads(source(nested, 'ontology')['utf8'])
    next(e for e in ontology['entities'] if e['type']['elementId'] == 'Resource')['fields'].append({'ref': {'documentId': 'domain', 'moduleId': 'm', 'elementId': 'details'}, 'protection': 'unprotected'})
    set_ontology(nested, encode(ontology))
    nested_raw = encode(nested)
    nested_child = weft.prepare_application_lineage(nested_raw).prepare_current_presence()
    nested_report = json.loads(nested_child.model_requirements_json())
    record = next(r for r in nested_report['records'] if r['record']['element'] == 'Resource')
    field = next(f for f in record['fields'] if f['field']['element'] == 'details')
    ok('separate field and nested descriptor identities', field['field']['element'] == 'details' and [d['identity']['element'] for d in field['descriptors']] == ['details', 'Details', 'detailText'])
    ok('nested graph shapes not flattened', [d['kind'] for d in field['descriptors']] == ['structured', 'record', 'scalar'])
    ok('nested graph original complete identities', all(d['identity'] == {'documentId': 'domain', 'revision': 'query-presence-core-r1', 'module': 'm', 'element': name} for d, name in zip(field['descriptors'], ['details', 'Details', 'detailText'])))
    ok('nested source retained exactly', nested_report['sourceRequestJson'] == nested_raw)
    ok('nested data grants no authority', nested_report['authority'] is False and nested_report['complete'] is False)
    # Same actual Core factory refuses wrong membership, not a Python interpreter.
    invalid = request()
    ontology = json.loads(source(invalid, 'ontology')['utf8'])
    next(e for e in ontology['entities'] if e['type']['elementId'] == 'Resource')['fields'][0]['ref']['elementId'] = 'staffId'
    set_ontology(invalid, encode(ontology))
    invalid_child = weft.prepare_application_lineage(encode(invalid)).prepare_current_presence()
    refuses('original compiler rejects foreign Record member', invalid_child.model_requirements_json)

receipt = {'version': 'weft-python-model-requirements-checks/0.1', 'mode': mode, 'passed': True, 'checks': len(checks), 'checksPassed': checks, 'nativeModule': weft.weft.__file__, 'nativeModuleSha256': hashlib.sha256(Path(weft.weft.__file__).read_bytes()).hexdigest(), 'packagePath': weft.__file__, 'nativeDatabase': False, 'authority': False, 'scope': 'actual native Python types over supplied compiler data fixtures; no native binding/current-authority admission'}
Path(sys.argv[3]).write_text(json.dumps(receipt, indent=2) + '\n')
print(json.dumps({'mode': mode, 'checks': len(checks), 'passed': True, 'nativeDatabase': False}))
