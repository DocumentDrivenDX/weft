# SPDX-License-Identifier: MIT OR Apache-2.0
"""Public Python wheel parity vectors for Unity current SUM 0.4 and AVG 0.5.
No database or native admission is exercised.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import pathlib
import zipfile
import importlib
import weft

parser = argparse.ArgumentParser()
parser.add_argument('--wheel', type=pathlib.Path, required=True)
parser.add_argument('--output', type=pathlib.Path, required=True)
args = parser.parse_args()
root = pathlib.Path(__file__).resolve().parents[2]

def compact(value):
    return json.dumps(value, separators=(',', ':'), ensure_ascii=False)

def sha(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()

native_path = pathlib.Path(importlib.import_module('weft.weft').__file__)
with zipfile.ZipFile(args.wheel) as wheel:
    native_members = [name for name in wheel.namelist() if name.endswith(('.so', '.pyd'))]
    assert len(native_members) == 1
    assert native_path.read_bytes() == wheel.read(native_members[0])

model_raw = (root / 'crates/weft-databricks/tests/fixtures/original-unity-model.json').read_bytes()
binding_raw = (root / 'crates/weft-databricks/tests/fixtures/original-unity-binding.json').read_bytes()
model = json.loads(model_raw)
access_columns = model['modules'][0]['extensions']['umf.access']['columns'][0]
target = access_columns['target']
model_digest = sha(model_raw)
binding_digest = sha(binding_raw)
column_set = {
    'documentId': model['id'], 'coreVersion': '0.10.0', 'documentSha256': model_digest,
    'module': 'search', 'pointer': '/modules/0/extensions/umf.access/columns/0',
}

def request(version: str, aggregate: str):
    amount_type = 'DECIMAL(38,10)'
    expected_columns = [
        {'fieldRef': {'module': 'search', 'element': 'count'}, 'nativeName': 'count',
         'nativeType': 'BIGINT', 'nullable': False,
         **({'sourceFacets': {'integerWidth': {'bits': 64, 'signed': True}}} if aggregate == 'sum' else {})},
        {'fieldRef': {'module': 'search', 'element': 'amount'}, 'nativeName': 'amount',
         'nativeType': amount_type, 'nullable': False,
         **({'sourceFacets': {'precision': 38, 'scale': 10}} if aggregate == 'sum' else {})},
    ]
    if aggregate == 'sum':
        measures = [
            {'id': 'sum_count', 'kind': 'sum', 'fieldRef': {'module': 'search', 'element': 'count'}},
            {'id': 'sum_amount', 'kind': 'sum', 'fieldRef': {'module': 'search', 'element': 'amount'}},
        ]
        kind = 'finite-sum'
    else:
        measures = [
            {'id': 'avg_count', 'kind': 'avg', 'fieldRef': {'module': 'search', 'element': 'count'}, 'scale': 0},
            {'id': 'avg_amount', 'kind': 'avg', 'fieldRef': {'module': 'search', 'element': 'amount'}, 'scale': 2},
        ]
        kind = 'finite-avg'
    return {
        'version': version,
        'source': {'version': '0.10.0', 'utf8': model_raw.decode('utf-8'), 'sha256': model_digest},
        'expectedTarget': {
            'kind': 'unity-catalog', 'warehouseId': 'warehouse-source-fixture',
            'catalog': 'main', 'schema': 'search', 'table': 'docs',
            'tableUuid': 'source-fixture-uuid', 'runtimeVersion': '2026-10-16',
            'columnSetTarget': target, 'columns': expected_columns,
        },
        'nativeColumns': {
            'target': target,
            'sets': [{
                'columnSet': column_set,
                'binding': {'sourceId': 'uc-search-binding', 'documentJson': binding_raw.decode('utf-8'), 'documentSha256': binding_digest},
            }],
        },
        'read': {
            'kind': kind, 'tableRef': {'module': 'search', 'element': 'Doc'},
            'predicate': None,
            'summary': {'kind': 'aggregate', 'groups': [], 'measures': measures, 'order': [], 'groupCap': 1},
            'frameCapBytes': 65536, 'deadlineMs': 5000,
        },
        'parameters': [],
    }

vectors = []
for name, version, aggregate, response_version, profile, algorithm in [
    ('sum04', 'weft.unity-current-read-request/0.4', 'sum', 'weft.unity-current-selection/0.5',
     'bounded-current-exact-sum/0.1', 'signed-coefficient-base10^19/1'),
    ('avg05', 'weft.unity-current-read-request/0.5', 'avg', 'weft.unity-current-selection/0.6',
     'bounded-current-exact-avg/0.1', 'signed-coefficient-base10^19-digit-quotient/1'),
]:
    good = request(version, aggregate)
    raw = compact(good)
    response_raw = weft.compile_raw_read_json(raw)
    response = json.loads(response_raw)
    assert response['status'] == 'compiled' and response['version'] == response_version, response
    assert response['obligations']['summaryProfile'] == profile
    arithmetic = response['arithmetic'] if aggregate == 'sum' else response['obligations']['arithmetic']
    assert arithmetic['algorithm'] == algorithm
    assert len(response['outputs']) == 2
    assert response['sql'] and ('TRY_SUM' in response['sql'])
    vectors.append({'id': name, 'request': raw, 'response': response_raw})

    missing_custody = dict(good)
    missing_custody.pop('nativeColumns')
    missing_raw = compact(missing_custody)
    refused = json.loads(weft.compile_raw_read_json(missing_raw))
    expected_code = 'WFT-UNITY-CURRENT-SUM' if aggregate == 'sum' else 'WFT-UNITY-CURRENT-AVG'
    assert refused['status'] == 'blocked' and refused['diagnostics'][0]['code'] == expected_code, refused
    vectors.append({'id': name + '-missing-native-columnset', 'request': missing_raw, 'response': compact(refused)})

# Prove a valid existing generic Spark 0.1 request still compiles through the
# public wheel dispatcher. This reuses the repository's authored source fixture.
legacy_raw = (root / 'tests/spark-pages/read-request.json').read_text(encoding='utf-8')
legacy_response_raw = weft.compile_raw_read_json(legacy_raw)
legacy_response = json.loads(legacy_response_raw)
assert legacy_response['status'] == 'compiled' and legacy_response['version'] == 'weft.raw-read/0.1', legacy_response
assert legacy_response['sql'] and len(legacy_response['outputs']) == 2, legacy_response
vectors.append({'id': 'legacy-generic-spark-0.1-compiled', 'request': legacy_raw, 'response': legacy_response_raw})

# Malformed legacy requests still reach the historical Spark refusal boundary.
for version in ['weft.raw-read/0.1', 'weft.unity-current-read-request/0.1', 'weft.unity-current-read-request/0.2', 'weft.unity-current-read-request/0.3']:
    raw = compact({'target': 'spark-delta4', 'version': version})
    response_raw = weft.compile_raw_read_json(raw)
    response = json.loads(response_raw)
    assert response['status'] == 'blocked' and response['version'] == 'weft.raw-read/0.1'
    assert response['diagnostics'][0]['code'] == 'WFT-RAW-SPARK'
    vectors.append({'id': 'legacy-generic-route-refusal-' + version.replace('/', '-'), 'request': raw, 'response': response_raw})

args.output.write_text(compact({'profile': 'weft.unity-current-sum-avg-host-parity/0.1', 'wheelSha256': sha(args.wheel.read_bytes()), 'nativeModuleSha256': sha(native_path.read_bytes()), 'vectors': vectors}) + '\n', encoding='utf-8')
print(json.dumps({'vectors': len(vectors), 'nativeDatabase': False, 'nativeAuthority': False}))
