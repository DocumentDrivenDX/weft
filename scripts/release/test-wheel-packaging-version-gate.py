# SPDX-License-Identifier: MIT OR Apache-2.0
"""Exercise the actual workflow snippets across the packaging-version boundary."""
import hashlib
import json
import os
from pathlib import Path
import tempfile
import textwrap
from unittest.mock import MagicMock, patch
import zipfile

workflow=Path('.github/workflows/release.yml').read_text()
identity_step=workflow.split('      - name: Check distribution and crate version against tag\n',1)[1].split('      - uses: PyO3/maturin-action',1)[0]
identity_source=textwrap.dedent(identity_step.split("python - <<'PYTHON'\n",1)[1].split('          PYTHON',1)[0])
step=workflow.split('      - name: Check the exact installed wheel and notices\n',1)[1].split('      - uses:',1)[0]
source=textwrap.dedent(step.split("python - <<'PYTHON'\n",1)[1].split('          PYTHON',1)[0])
fragment=textwrap.dedent('          packaging='+workflow.split('          packaging=',1)[1].split('          features=',1)[0])
checks=[]
with tempfile.TemporaryDirectory() as environment:
    with patch.dict(os.environ,{'GITHUB_ENV':str(Path(environment)/'github-env'),'RELEASE_REQUESTED':'false','RELEASE_TAG':''}):
        exec(identity_source,{})
    recorded=(Path(environment)/'github-env').read_text()
    assert 'DISTRIBUTION=umf-weft\n' in recorded and 'WHEEL_PREFIX=umf_weft-0.4.55-\n' in recorded
    fake_archive=MagicMock();fake_archive.__enter__.return_value.namelist.return_value=['weft/fixture.so']
    with patch('pathlib.Path.glob',return_value=iter([Path('dist/exact.whl')])),patch('pathlib.Path.read_text',return_value='{}'),patch('pathlib.Path.write_text'),patch('zipfile.ZipFile',return_value=fake_archive),patch('subprocess.run')as invoke,patch('subprocess.check_output',return_value='a'*40),patch.dict(os.environ,{'DISTRIBUTION':'umf-weft'}):
        exec(source,{});assert invoke.call_count==1
    assert 'scripts/release/verify-wheel-packaging.py'in invoke.call_args.args[0]
    assert '--distribution' in invoke.call_args.args[0]
    assert invoke.call_args.kwargs['check']is True
checks.append('current package always invokes mandatory validator without existence waiver')
with tempfile.TemporaryDirectory()as directory:
    root=Path(directory);wheels=[]
    for i in range(3):
        wheel=root/f'fixture-{i}.whl'
        with zipfile.ZipFile(wheel,'w')as archive:archive.writestr('weft/fixture.so',b'fixture-native-bytes')
        wheels.append(wheel)
    mac=wheels[0];source_freeze='a'*40;packaging_commit='b'*40
    def gate(version):
        with patch.dict(os.environ,{'PACKAGING_COMMIT':packaging_commit}):
            exec(fragment,dict(version=version,root=root,wheels=wheels,mac=mac,source_freeze=source_freeze,json=json,hashlib=hashlib,zipfile=zipfile,os=os))
    try:gate('0.4.48')
    except AssertionError:pass
    else:raise AssertionError('missing mandatory reports accepted')
    checks.append('publisher refuses missing reports')
    for i,wheel in enumerate(wheels):
        (root/f'packaging-{i}.json').write_text(json.dumps({'status':'passed','version':'0.4.48','wheel':wheel.name,'wheelSha256':hashlib.sha256(wheel.read_bytes()).hexdigest(),'nativeModuleSha256':hashlib.sha256(b'fixture-native-bytes').hexdigest(),'sourceCommit':source_freeze if wheel==mac else packaging_commit}))
    gate('0.4.48');checks.append('three exact current reports accepted')
    path=root/'packaging-0.json';report=json.loads(path.read_text());report['wheelSha256']='0'*64;path.write_text(json.dumps(report))
    try:gate('0.4.48')
    except AssertionError:pass
    else:raise AssertionError('mismatched report accepted')
    checks.append('mismatched current report refused')
print(json.dumps({'status':'passed','checks':checks}))
