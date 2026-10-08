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
for path in files:
    fixture = json.loads(path.read_text())
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
assert len(files) == 7
out = ROOT/'target/b005/original-embedding'; out.mkdir(parents=True, exist_ok=True)
(out/'cases.json').write_text(json.dumps(cases)+'\n')
(out/'reports.json').write_text(json.dumps(reports)+'\n')
summary = dict(cases=len(cases), originalConfigurations=7, fullResponseParity=True, deterministicRepeats=7, subprocessDisabled=True, nativeModule=weft.__file__, version=weft.__version__, scope='test-original feature; seven pinned native compound configurations only')
(out/'python-summary.json').write_text(json.dumps(summary, indent=2)+'\n')
print(json.dumps(summary))
