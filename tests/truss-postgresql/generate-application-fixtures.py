"""Reuse authored application models; candidate mappings do not qualify codecs."""
import copy
import hashlib
import importlib.util
import json
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('physical', HERE / 'generate-compiler-fixtures.py')
physical = importlib.util.module_from_spec(spec)
spec.loader.exec_module(physical)
source = json.loads((HERE.parent / 'application/fixtures/cases.json').read_text())
optional = copy.deepcopy(next(c for c in source if c['id'] == 'injection-text'))
optional['id'] = 'optional-scalar'
optional['request']['sql'] = 'SELECT c.id,c.nickname FROM Customer c ORDER BY c.id LIMIT 10'
optional['request'].pop('parameters', None)
source.append(optional)
forward = copy.deepcopy(next(c for c in source if c['id']=='related-filter'))
forward['id']='has-related-scalar'
forward['request']['sql']='SELECT c.id FROM Customer c WHERE HAS_RELATED(c.orders,KEY(:order_id)) ORDER BY c.id LIMIT 5'
source.append(forward)
inverse = copy.deepcopy(forward)
inverse['id']='inverse-has-related'
inverse['request']['sql']='SELECT o.id FROM Orders o WHERE HAS_RELATED(o.customer,KEY(:customer_id)) ORDER BY o.id LIMIT 5'
inverse['request']['parameters']={'customer_id':{'family':'integer','value':'1'}}
source.append(inverse)
for original_id,new_id,old_alias,new_alias in [('related-page','related-alias-collision','c','weft_related_1'),('has-related-scalar','exists-alias-collision','c','weft_edge'),('related-page','row-alias-collision','c','weft_state_0')]:
    case=copy.deepcopy(next(c for c in source if c['id']==original_id))
    case['id']=new_id
    case['request']['sql']=case['request']['sql'].replace(old_alias+'.',new_alias+'.').replace(' '+old_alias+' ', ' '+new_alias+' ')
    source.append(case)
large=copy.deepcopy(next(c for c in source if c['id']=='related-page'))
large['id']='related-numeric-order'
source.append(large)
sequence=copy.deepcopy(optional)
sequence['id']='sequence-page'
sequence['request']['sql']='SELECT c.id,c.tags FROM Customer c ORDER BY c.id LIMIT 10'
source.append(sequence)
numeric_sequence=copy.deepcopy(sequence)
numeric_sequence['id']='numeric-sequence-page'
for module_input in numeric_sequence['request']['modules']:
    doc=json.loads(module_input['documentJson'])
    for module in doc['modules']:
        for element in module['elements']:
            if element['id']=='tag-item':element.update(scalarType='integer',facets={'integerWidth':{'bits':64,'signed':False}})
    raw=json.dumps(doc,ensure_ascii=False,separators=(',',':'))
    module_input['documentJson']=raw
    module_input['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
source.append(numeric_sequence)
optional_sequence=copy.deepcopy(sequence)
optional_sequence['id']='optional-sequence-page'
for module_input in optional_sequence['request']['modules']:
    doc=json.loads(module_input['documentJson'])
    for module in doc['modules']:
        for element in module['elements']:
            if element['id']=='tags':element['nullability']='absent-allowed'
    raw=json.dumps(doc,ensure_ascii=False,separators=(',',':'))
    module_input['documentJson']=raw
    module_input['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
source.append(optional_sequence)
structured=copy.deepcopy(optional)
structured['id']='structured-page'
structured['request']['sql']='SELECT c.id,c.address FROM Customer c ORDER BY c.id LIMIT 10'
source.append(structured)
small_cursor=copy.deepcopy(next(c for c in source if c['id']=='single-cursor'))
small_cursor['id']='whole-cursor-one'
small_cursor['request']['parameters']['cursor']['value']='1'
source.append(small_cursor)
numeric_structured=copy.deepcopy(structured)
numeric_structured['id']='numeric-structured-page'
for module_input in numeric_structured['request']['modules']:
    doc=json.loads(module_input['documentJson'])
    for module in doc['modules']:
        for element in module['elements']:
            if element['id']=='zip':element.update(scalarType='integer',facets={'integerWidth':{'bits':64,'signed':False}})
    raw=json.dumps(doc,ensure_ascii=False,separators=(',',':'))
    module_input['documentJson']=raw;module_input['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
source.append(numeric_structured)
for base,new_id in [(sequence,'map-page'),(optional_sequence,'optional-map-page'),(numeric_sequence,'numeric-map-page')]:
    case=copy.deepcopy(base);case['id']=new_id
    for module_input in case['request']['modules']:
        doc=json.loads(module_input['documentJson'])
        for module in doc['modules']:
            for element in module['elements']:
                if element['id']=='tags':element['cardinality']='map'
        raw=json.dumps(doc,ensure_ascii=False,separators=(',',':'))
        module_input['documentJson']=raw;module_input['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
    source.append(case)
recursive_ids=set()
for base,new_id in [(structured,'cyclic-structured-props'),(sequence,'nested-sequence-props')]:
    case=copy.deepcopy(base);case['id']=new_id;recursive_ids.add(new_id)
    for module_input in case['request']['modules']:
        doc=json.loads(module_input['documentJson']);module=doc['modules'][0]
        if new_id=='cyclic-structured-props':
            record=next(e for e in module['elements'] if e['id']=='address-record')
            record['members'].append({'module':'sales','element':'next'})
            module['elements'].append(dict(id='next',name='next',kind='field',nullability='absent-allowed',cardinality='one',extensions={},references=[{'role':'record-type','module':'sales','element':'address-record'}]))
        else:
            item=next(e for e in module['elements'] if e['id']=='tag-item')
            item.pop('scalarType');item.update(cardinality='array',itemType={'module':'sales','element':'nested-item'})
            module['elements'].append(dict(id='nested-item',name='nested-item',kind='field',scalarType='integer',nullability='required',cardinality='one',extensions={},facets={'integerWidth':{'bits':64,'signed':False}}))
        raw=json.dumps(doc,ensure_ascii=False,separators=(',',':'))
        module_input['documentJson']=raw;module_input['pin']['sha256']=hashlib.sha256(raw.encode()).hexdigest()
    source.append(case)
for base in ['global-count','grouped-count','global-sum']:
    case=copy.deepcopy(next(c for c in source if c['id']==base));case['id']='empty-'+base;source.append(case)
selected = {'empty-global-count','empty-grouped-count','empty-global-sum','unicode-key-order', 'unicode-key-cursor', 'exact-large-key-order', 'injection-text', 'global-count', 'grouped-count', 'join-count', 'global-sum', 'case-folded-param', 'optional-scalar', 'composite-cursor', 'has-related-scalar', 'inverse-has-related', 'related-page', 'inverse-page', 'related-alias-collision', 'exists-alias-collision', 'related-numeric-order', 'row-alias-collision', 'sequence-page', 'numeric-sequence-page', 'optional-sequence-page', 'map-page', 'optional-map-page', 'numeric-map-page', 'structured-page', 'whole-entity', 'scalar-page', 'single-cursor', 'related-filter', 'whole-cursor-one', 'numeric-structured-page'}
cases = []
for case in source:
    if case['id'] not in selected and case['id'] not in recursive_ids:
        continue
    for home in ['props','row']:
        request = copy.deepcopy(case['request'])
        request['interfaceVersion'] = 'weft-compile/0.2.0'
        binding, mapping = physical.make(request, home)
        raw = json.dumps(binding, ensure_ascii=False, separators=(',', ':'))
        request['target'] = dict(backendId='truss.postgresql', backendVersion='0.1.0-candidate', targetProfile='pg17.9-candidate', bindingJson=raw, bindingSha256=hashlib.sha256(raw.encode()).hexdigest())
        request['options'] = {'allowCandidate': True}
        cases.append(dict(id=case['id']+'-'+home, home=home, request=request, mapping=mapping))
# A scalar root can use the full native tree profile without losing typed SQL.
import base64
for base in ['global-sum','whole-entity']:
    case=copy.deepcopy(next(c for c in cases if c['id']==base+'-row'))
    binding=json.loads(case['request']['target']['bindingJson'])
    for prop in binding['properties']:
        home=json.loads(base64.b64decode(prop['homeDefinition']['bytesBase64']))
        home['access']='complete-value-tree'
        prop['homeDefinition']=physical.artifact(home)
    raw=json.dumps(binding,ensure_ascii=False,separators=(',',':'))
    case['request']['target']['bindingJson']=raw
    case['request']['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()
    case['id']=base+'-complete-tree-row'
    cases.append(case)
(HERE / 'fixtures/application-cases.json').write_text(json.dumps(cases, ensure_ascii=False, indent=2)+'\n')
print('Candidate application component cases:', len(cases))
