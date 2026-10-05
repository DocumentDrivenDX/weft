"""Direct extension call with process spawning disabled and no executables on PATH."""
import json, pathlib, subprocess
import weft_spike

def forbid(*args, **kwargs):
    raise AssertionError('Python extension attempted a subprocess')
subprocess.Popen = forbid
request=json.loads((pathlib.Path(__file__).resolve().parents[2]/'target/b001/cases.json').read_text())[575]['request']
response=json.loads(weft_spike.compile_json(request))
assert response['status']=='compiled'
assert [p['value'] for p in response['parameters']]==['18446744073709551615','9007199254740993.12']
print('Native Python direct-call smoke passed with empty executable PATH and subprocesses disabled.')
