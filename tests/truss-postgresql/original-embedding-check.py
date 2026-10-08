"""Conformance-only original composition through the actual native Python ABI."""
import hashlib, json, os, subprocess
from pathlib import Path
import weft
ROOT = Path(__file__).resolve().parents[2]
files = sorted((ROOT/'tests/truss-postgresql/fixtures').glob('original-*-compile-transport.json'))
def forbidden(*args, **kwargs):
    raise AssertionError('Subprocess attempted during compile')
os.environ['PATH'] = ''
subprocess.Popen = subprocess.run = subprocess.check_output = forbidden
cases, reports = [], []
entries=[(path,json.loads(path.read_text())) for path in files]
relationship_path=ROOT/'tests/truss-postgresql/fixtures/original-relationship-public-transport.json'
entries += [(relationship_path.with_name(f'relationship-{index}.json'),fixture) for index,fixture in enumerate(json.loads(relationship_path.read_text()))]
optional_path=ROOT/'tests/truss-postgresql/fixtures/original-optional-public-transport.json'
entries += [(optional_path.with_name(f'optional-{index}.json'),fixture) for index,fixture in enumerate(json.loads(optional_path.read_text()))]
entity_path=ROOT/'tests/truss-postgresql/fixtures/original-entity-public-transport.json'
entries += [(entity_path.with_name(f'entity-{index}.json'),fixture) for index,fixture in enumerate(json.loads(entity_path.read_text()))]
signed_path=ROOT/'tests/truss-postgresql/fixtures/original-signed-public-transport.json'
entries += [(signed_path.with_name(f'signed-{index}.json'),fixture) for index,fixture in enumerate(json.loads(signed_path.read_text()))]
boolean_path=ROOT/'tests/truss-postgresql/fixtures/original-boolean-public-transport.json'
entries += [(boolean_path.with_name(f'boolean-{index}.json'),fixture) for index,fixture in enumerate(json.loads(boolean_path.read_text()))]
multi_path=ROOT/'tests/truss-postgresql/fixtures/original-multi-recursive-entity-public.json'
entries += [(multi_path.with_name(f'multi-recursive-{index}.json'),fixture) for index,fixture in enumerate(json.loads(multi_path.read_text()))]
assert len(entries)==84
for path,fixture in entries:
    request = fixture['request']
    raw = weft.compile_json(json.dumps(request))
    assert json.loads(raw) == fixture['response'], path.name
    assert weft.compile_json(json.dumps(request)) == raw, path.name
    cases.append(dict(id=path.stem, request=request)); reports.append(dict(raw=raw))
    for kind in ['candidate-disabled', 'unsupported-cut', 'wrong-digest']:
        altered = json.loads(json.dumps(request))
        if kind == 'candidate-disabled': altered['options']['allowCandidate'] = False
        else:
            altered['target']['bindingJson'] += ' '
            if kind == 'unsupported-cut':
                altered['target']['bindingSha256'] = hashlib.sha256(altered['target']['bindingJson'].encode()).hexdigest()
        refused_raw = weft.compile_json(json.dumps(altered))
        refused = json.loads(refused_raw)
        assert refused['status'] == 'blocked' and 'sql' not in refused, (path.name, kind)
        cases.append(dict(id=path.stem+'-'+kind, request=altered)); reports.append(dict(raw=refused_raw))
    if path.stem.startswith('boolean-'):
        altered = json.loads(json.dumps(request))
        altered['sql'] = 'SELECT SUM(c.id) AS total FROM Customer c'
        altered.pop('readProfile', None)
        altered['parameters'] = {}
        refused_raw = weft.compile_json(json.dumps(altered))
        refused = json.loads(refused_raw)
        assert refused['status'] == 'blocked' and 'sql' not in refused
        assert any(d['code'] == 'WFT-TYPE' for d in refused['diagnostics'])
        cases.append(dict(id=path.stem+'-boolean-sum', request=altered))
        reports.append(dict(raw=refused_raw))
assert len(files) == 8
out = ROOT/'target/b005/original-embedding'; out.mkdir(parents=True, exist_ok=True)
(out/'cases.json').write_text(json.dumps(cases)+'\n')
(out/'reports.json').write_text(json.dumps(reports)+'\n')
summary = dict(cases=len(cases), originalConfigurations=38, fullResponseParity=True, deterministicRepeats=84, subprocessDisabled=True, nativeModule=weft.__file__, version=weft.__version__, scope='test-original feature; seven pinned compounds, one native uint64 relationship and two optional scalar/entity home configurations and fourteen complete scalar/recursive entity cuts and ten signed scalar configurations and two Boolean scalar configurations and one native Boolean sequence configuration and one multi-recursive entity configuration')
(out/'python-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary))
