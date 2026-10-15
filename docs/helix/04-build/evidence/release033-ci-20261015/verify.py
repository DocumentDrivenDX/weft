import hashlib,json,zipfile
from pathlib import Path
root=Path('/private/tmp/weft-release033-ci-all-20261015')
def sha(b):return hashlib.sha256(b).hexdigest()
proofs=[]
for directory in sorted(root.iterdir()):
 wheels=list(directory.glob('*.whl'));reports=list(directory.glob('current-query-smoke-*.json'))
 assert len(wheels)==len(reports)==1,directory
 wheel=wheels[0];report=json.loads(reports[0].read_text())
 assert report['sourceCommit']=='cc943a2f0ad87e3742cccdc171184bad7d4dbf60'
 assert report['version']=='0.4.33' and report['wheel']==wheel.name
 assert report['securityReady'] is False and report['nativeAdmission'] is False
 pins=report['pinsBeforeAfter'];assert sha(wheel.read_bytes())==pins['wheelSha256']
 with zipfile.ZipFile(wheel) as archive:
  modules=[n for n in archive.namelist() if n.endswith(('.so','.pyd'))];assert len(modules)==1
  assert sha(archive.read(modules[0]))==pins['nativeModuleSha256']
  assert sha(archive.read('weft/__init__.py'))==pins['shimSha256']
 assert len(report['checks'])==4 and report['checks'][-1]['refused'] is True
 proofs.append(report)
assert len(proofs)==5
assert all(p['checks']==proofs[0]['checks'] for p in proofs)
receipt={'runId':38804888838,'sourceCommit':proofs[0]['sourceCommit'],'version':'0.4.33','platforms':proofs,'verifiedWheelCount':5,'checksIdenticalAcrossPlatforms':True,'published':False,'securityReady':False,'scope':'Exact CI artifacts and compiler smoke only. No native backend security acceptance.'}
Path('/private/tmp/weft-release033-ci-verified.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(json.dumps({'verifiedWheels':5,'sameChecks':True,'pins':[{'wheel':p['wheel'],**p['pinsBeforeAfter']} for p in proofs]},indent=2))
