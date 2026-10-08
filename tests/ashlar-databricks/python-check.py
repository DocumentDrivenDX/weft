"""Native Python ABI parity against saved Databricks-tested compiler artifacts.

This checks embedding parity; independent result oracles live in native harnesses.
"""
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys

root = Path(__file__).resolve().parents[2]
out = Path(os.environ.get('WEFT_ASHLAR_EMBEDDING_OUTPUT', root / 'target/b006/embeddings'))
out.mkdir(parents=True, exist_ok=True)
evidence = root / 'docs/helix/04-build/evidence'
cases = []
for scope in ['columns-native', 'application-native', 'key-refusal', 'unsigned-columns', 'optional-native', 'relationship-native', 'compound-native', 'compound-boundaries-native', 'compound-application-native']:
    for line in (evidence / f'B-006-{scope}/compile-artifacts.jsonl').read_text().splitlines():
        case = json.loads(line)
        case['id'] = scope + ':' + str(case['id'])
        cases.append(case)
for scope in ['scalar-native', 'global-native']:
    for path in sorted((evidence / f'B-006-{scope}').glob('*-compile.json')):
        case = json.loads(path.read_text())
        case['id'] = scope + ':' + path.stem
        cases.append(case)
case = json.loads((evidence / 'B-006-cross-module-native/compile.json').read_text())
case['id'] = 'cross-module'
cases.append(case)
assert len(cases) == 463, len(cases)
# Historical native receipts keep their original declarations. The sole metadata
# correction is specified independently here; native SQL and meaning stay exact.
metadata_corrections = []
for case in cases:
    if case['response']['status'] != 'compiled':
        assert case['response']['status'] == 'blocked', case['id']
        continue
    before = json.dumps(case['response'], ensure_ascii=False, sort_keys=True)
    corrections = 0
    for operation in case['response']['qualification']['operations']:
        declaration = operation['declaration']
        if declaration['id'] == 'value.presence':
            domain = declaration['logicalDomain']
            old = {'subset': 'optional scalar envelopes; absent or exact value; explicit native null refuses'}
            new = {'subset': 'optional scalar or compound envelopes; absent or exact value; explicit native null refuses'}
            assert domain in [old, new], (case['id'], domain)
            if domain == old:
                declaration['logicalDomain'] = new
                corrections += 1
    if corrections:
        metadata_corrections.append({'id': case['id'], 'nativeResponseSha256': hashlib.sha256(before.encode()).hexdigest(), 'fields': corrections, 'correction': 'value.presence logicalDomain only'})
(out / 'metadata-corrections.json').write_text(json.dumps(metadata_corrections, indent=2) + '\n')
from weft import weft
assert Path(weft.__file__).suffix in ['.so', '.pyd'], weft.__file__
module_path = Path(weft.__file__)
module_hash = hashlib.sha256(module_path.read_bytes()).hexdigest()

def forbidden(*args, **kwargs):
    raise AssertionError('compiler attempted subprocess IO')
subprocess.Popen = forbidden
os.environ['PATH'] = ''
reports = []
for case in cases:
    request = json.dumps(case['request'], ensure_ascii=False, separators=(',', ':'))
    raw = weft.compile_json(request)
    assert json.loads(raw) == case['response'], case['id']
    assert weft.compile_json(request) == raw, case['id']
    reports.append({'id': case['id'], 'raw': raw})
(out / 'cases.json').write_text(json.dumps(cases, ensure_ascii=False))
(out / 'reports.json').write_text(json.dumps(reports, ensure_ascii=False))
summary = {'cases': len(cases), 'python': sys.version, 'extensionSha256': module_hash,
           'nativeExtension': True, 'subprocessForbidden': True, 'pathEmpty': True,
           'fullResponseParity': True, 'deterministic': True,
           'scope': 'saved native-tested compiler artifact parity; no database execution'}
(out / 'python-summary.json').write_text(json.dumps(summary, indent=2) + '\n')
print(json.dumps(summary))
