# SPDX-License-Identifier: MIT OR Apache-2.0
"""Smoke the installed wheel against original native65/native66 bindings."""
import hashlib
import json
from pathlib import Path
import weft

fixture = json.loads((Path(__file__).parent / 'fixtures/truss066-products.json').read_text())
configuration = json.loads((Path(__file__).parent / 'fixtures/truss066-configuration.json').read_text())
compiler = weft.MutationCompiler(configuration['utf8'])
for member, runtime in (('historicalNative65Binding', '0.65'), ('binding', '0.66')):
    binding = fixture[member]
    handle = compiler.register_truss_selection(json.dumps({
        'model': fixture['model'], 'modelPin': fixture['modelPin'],
        'binding': binding, 'evidence': ['Original native binding installed-wheel smoke only']}))
    lowered = json.loads(compiler.lower_selection_json(handle, json.dumps({
        'selection': fixture['selection'], 'model': fixture['model'],
        'binding': binding, 'maxItems': 2})))
    assert lowered['status'] == 'lowered', lowered
    assert lowered['bounds'] == {'maxItems': 2, 'fetchLimit': 3}
    assert lowered['parameters'][0]['value'] == {'string': 'bar'}
    assert 'runtime' + runtime in lowered['obligations'][0]
changed = json.loads(fixture['binding']['utf8'])
changed['releaseSha256'] = '0' * 64
raw = json.dumps(changed)
binding = {'mediaType': 'application/json', 'utf8': raw,
           'sha256': hashlib.sha256(raw.encode()).hexdigest()}
handle = compiler.register_truss_selection(json.dumps({
    'model': fixture['model'], 'modelPin': fixture['modelPin'],
    'binding': binding, 'evidence': ['Changed native66 manifest refusal']}))
refused = json.loads(compiler.lower_selection_json(handle, json.dumps({
    'selection': fixture['selection'], 'model': fixture['model'],
    'binding': binding, 'maxItems': 2})))
assert refused['status'] == 'blocked', refused
print('Installed wheel: original native65/native66 mutation registration and lowering passed')

# Compiler-only compatibility fixture: native67 retains this physical layout.
# This does not claim original native67 execution or caller authority.
for digest, expected in (('8c553480e1bcb95ccaebb9aa04150a0253217838c8df6ad09acd5ecece2402ee', 'lowered'), ('0' * 64, 'blocked')):
    compatible = json.loads(fixture['binding']['utf8'])
    compatible['releaseId'] = 'truss-catalog-runtime/0.67-candidate'
    compatible['releaseSha256'] = digest
    raw = json.dumps(compatible)
    binding = {'mediaType': 'application/json', 'utf8': raw, 'sha256': hashlib.sha256(raw.encode()).hexdigest()}
    handle = compiler.register_truss_selection(json.dumps({
        'model': fixture['model'], 'modelPin': fixture['modelPin'],
        'binding': binding, 'evidence': ['Compiler-only native67 compatibility; no native execution or authority claim']}))
    result = json.loads(compiler.lower_selection_json(handle, json.dumps({
        'selection': fixture['selection'], 'model': fixture['model'], 'binding': binding, 'maxItems': 2})))
    assert result['status'] == expected, result
    if expected == 'lowered':
        assert 'runtime0.67' in result['obligations'][0]
        assert result['parameters'][0]['value'] == {'string': 'bar'}
print('Installed wheel: compiler-only native67 lowering and exact manifest refusal passed')
