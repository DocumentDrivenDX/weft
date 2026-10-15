"""Verify actual downloaded public assets against retained tag-run artifacts."""
import hashlib,json,zipfile
from pathlib import Path
root=Path('/private/tmp/weft-release033-published-20261015');original=Path('/private/tmp/weft-release033-tag-ci-all-20261015')
def sha(b):return hashlib.sha256(b).hexdigest()
pins=json.loads((root/'release-pins.json').read_text())
assert pins['commit']=='cc943a2f0ad87e3742cccdc171184bad7d4dbf60'
assert pins['workflowCommit']=='e98bd1966c0cc902c09ade0cb6f30d73ff7b365a'
assert pins['version']=='0.4.33' and pins['tag']=='v0.4.33'
reports=[json.loads(p.read_text()) for p in root.glob('current-query-smoke-*.json')]
assert len(reports)==len(pins['currentQuerySmoke'])==len(list(root.glob('*.whl')))==5
assert {p['wheel']:p for p in reports}=={p['wheel']:p for p in pins['currentQuerySmoke']}
for r in reports:
 p=root/r['wheel'];matches=list(original.rglob(p.name));assert len(matches)==1
 assert p.read_bytes()==matches[0].read_bytes()
 assert r['sourceCommit']==pins['commit'] and r['version']==pins['version']
 assert sha(p.read_bytes())==r['pinsBeforeAfter']['wheelSha256']
 assert r['checks']==reports[0]['checks'] and len(r['checks'])==4
 assert r['securityReady'] is False and r['nativeAdmission'] is False
 with zipfile.ZipFile(p) as z:
  modules=[n for n in z.namelist() if n.endswith(('.so','.pyd'))];assert len(modules)==1
  assert sha(z.read(modules[0]))==r['pinsBeforeAfter']['nativeModuleSha256']
  assert sha(z.read('weft/__init__.py'))==r['pinsBeforeAfter']['shimSha256']
sums={n:h for h,n in (line.split() for line in (root/'SHA256SUMS').read_text().splitlines())}
assert sums=={p.name:sha(p.read_bytes()) for p in root.glob('*.whl')}
print(json.dumps({'release':'https://github.com/DocumentDrivenDX/weft/releases/tag/v0.4.33','sourceCommit':pins['commit'],'workflowCommit':pins['workflowCommit'],'publishedWheelsVerified':5,'byteIdenticalToTagRun':38807410042,'promotionRun':38808676764,'securityReady':False},indent=2))
