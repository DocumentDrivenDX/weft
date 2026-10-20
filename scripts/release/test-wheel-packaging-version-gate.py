# SPDX-License-Identifier: MIT OR Apache-2.0
"""Exercise the actual workflow snippets across the packaging-version boundary."""
import hashlib
import json
from pathlib import Path
import tempfile
import textwrap
from unittest.mock import patch
import zipfile

workflow=Path('.github/workflows/release.yml').read_text()
step=workflow.split('      - name: Check the exact installed wheel and notices\n',1)[1].split('      - uses:',1)[0]
source=textwrap.dedent(step.split("python - <<'PYTHON'\n",1)[1].split('          PYTHON',1)[0])
fragment=textwrap.dedent('          packaging='+workflow.split('          packaging=',1)[1].split('          features=',1)[0])
checks=[]
with patch('pathlib.Path.glob',return_value=iter([Path('dist/exact.whl')])),patch('subprocess.run')as invoke:
    exec(source,{});assert invoke.call_count==1
    assert 'scripts/release/verify-wheel-packaging.py'in invoke.call_args.args[0]
    assert invoke.call_args.kwargs['check']is True
checks.append('current package always invokes mandatory validator without existence waiver')
with tempfile.TemporaryDirectory()as directory:
    root=Path(directory);wheels=[]
    for i in range(5):
        wheel=root/f'fixture-{i}.whl'
        with zipfile.ZipFile(wheel,'w')as archive:archive.writestr('weft/fixture.so',b'fixture-native-bytes')
        wheels.append(wheel)
    def gate(version):
        exec(fragment,dict(version=version,root=root,wheels=wheels,json=json,hashlib=hashlib,zipfile=zipfile))
    try:gate('0.4.48')
    except AssertionError:pass
    else:raise AssertionError('missing mandatory reports accepted')
    checks.append('publisher refuses missing reports')
    for i,wheel in enumerate(wheels):
        (root/f'packaging-{i}.json').write_text(json.dumps({'status':'passed','version':'0.4.48','wheel':wheel.name,'wheelSha256':hashlib.sha256(wheel.read_bytes()).hexdigest(),'nativeModuleSha256':hashlib.sha256(b'fixture-native-bytes').hexdigest()}))
    gate('0.4.48');checks.append('five exact current reports accepted')
    path=root/'packaging-0.json';report=json.loads(path.read_text());report['wheelSha256']='0'*64;path.write_text(json.dumps(report))
    try:gate('0.4.48')
    except AssertionError:pass
    else:raise AssertionError('mismatched report accepted')
    checks.append('mismatched current report refused')
print(json.dumps({'status':'passed','checks':checks}))
