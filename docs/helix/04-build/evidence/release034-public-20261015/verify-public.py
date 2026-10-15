from pathlib import Path
import hashlib,json,zipfile
root=Path('/private/tmp/weft-release034-public-20261015')
release=json.loads((root/'github-release.json').read_text())
assert release['tag_name']=='v0.4.34' and release['draft'] is False and release['published_at']
assets={a['name']:a for a in release['assets']}
for name,asset in assets.items():
 if name in ['release-pins.json','SHA256SUMS'] or name.endswith(('.whl','.json')):
  p=root/name;assert p.is_file(),name
  assert asset['digest']=='sha256:'+hashlib.sha256(p.read_bytes()).hexdigest(),name
pins=json.loads((root/'release-pins.json').read_text())
assert pins['commit']=='86273dd3c2b4c2b99c8d0268c9dff8537e6ce20e' and pins['version']=='0.4.34' and pins['tag']=='v0.4.34'
assert pins['trussBackend']=='0.4.37' and pins['trussRuntime']=='truss-catalog-runtime/0.68-candidate'
assert pins['trussRuntimeSha256']=='3588cf030cb46c7d12368557e351bd767c10b4dd2e1644f7f100cec54602a2fc'
assert set(pins['features'])=={'truss-postgresql-pg18','current-postgresql-scalar-candidate','current-databricks-scalar-candidate'}
pg={r['wheel']:r for r in pins['currentQuerySmoke']}; db={r['wheel']:r for r in pins['currentDatabricksSmoke']}
wheels=sorted(root.glob('*.whl'));assert len(wheels)==len(pg)==len(db)==5
checksums=dict((name,digest)for digest,name in (s.split()for s in (root/'SHA256SUMS').read_text().splitlines()))
records=[]
for w in wheels:
 p=pg[w.name]; d=db[w.name]; assert p['pinsBeforeAfter']==d['pinsBeforeAfter']; observed=p['pinsBeforeAfter']
 assert p['sourceCommit']==d['sourceCommit']==pins['commit'] and p['version']==d['version']==pins['version']
 assert len(p['checks'])==4 and d['controls']==len(d['checks'])==32
 assert p['nativeAdmission'] is d['nativeAdmission'] is False and p['securityReady'] is d['securityReady'] is False
 digest=hashlib.sha256(w.read_bytes()).hexdigest();assert observed['wheelSha256']==checksums[w.name]==digest
 with zipfile.ZipFile(w)as z:
  native=[n for n in z.namelist()if n.endswith(('.so','.pyd'))]; assert len(native)==1
  assert observed['nativeModuleSha256']==hashlib.sha256(z.read(native[0])).hexdigest()
  assert observed['shimSha256']==hashlib.sha256(z.read('weft/__init__.py')).hexdigest()
 records.append({'wheel':w.name,'pins':observed})
result={'release':release['html_url'],'publishedAt':release['published_at'],'sourceCommit':pins['commit'],'wheels':records,'githubAssetDigestsVerified':True,'embeddedSmokePinsVerified':True,'nativeSecurityAcceptance':False,'casesPromoted':[]}
(root/'verified-public.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
