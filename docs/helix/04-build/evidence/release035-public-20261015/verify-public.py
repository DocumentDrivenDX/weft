from pathlib import Path
import hashlib,importlib.util,json,zipfile
root=Path('/private/tmp/weft-release035-public-20261015');commit='ff3695af5ece9d386b3dcbf266cf23d632997733'
release=json.loads((root/'github-release.json').read_text());assert release['tag_name']=='v0.4.35'and release['draft']is False and release['published_at']
assets={a['name']:a for a in release['assets']};assert len(assets)==22
for name,asset in assets.items():
 p=root/name;assert p.is_file(),name;assert asset['digest']=='sha256:'+hashlib.sha256(p.read_bytes()).hexdigest(),name
pins=json.loads((root/'release-pins.json').read_text());assert pins['commit']==pins['workflowCommit']==commit and pins['version']=='0.4.35'and pins['tag']=='v0.4.35'
assert pins['trussBackend']=='0.4.37'and pins['trussBackendSelection']=='closed-runtime-hash'and pins['trussRuntime']=='truss-catalog-runtime/0.68-candidate'
assert pins['trussRuntimeSha256']=='3588cf030cb46c7d12368557e351bd767c10b4dd2e1644f7f100cec54602a2fc'
assert set(pins['features'])=={'truss-postgresql-pg18','current-postgresql-scalar-candidate','current-databricks-scalar-candidate','current-query-refinement-candidate'}
assert pins['trussQueryAbiBackend']=={'backendVersion':'0.4.38','targetProfile':'truss-runtime-pg18.6/0.5','bindingProfile':'truss-runtime-binding/0.5','queryAbi':{'identity':'truss-postgresql-logical-query','version':'1.0','sha256':'9fce2c91eff11ae2d8e6c0e3759b425efb1457e2c9675dc4383fc28430909806'},'nativeAdmission':False}
pg={r['wheel']:r for r in pins['currentQuerySmoke']};db={r['wheel']:r for r in pins['currentDatabricksSmoke']};refine={r['wheel']:r for r in pins['currentQueryRefinementSmoke']}
wheels=sorted(root.glob('*.whl'));assert len(wheels)==len(pg)==len(db)==len(refine)==5
spec=importlib.util.spec_from_file_location('gate','/private/tmp/weft-refinement-20261015/scripts/release/verify-refinement-smoke.py');gate=importlib.util.module_from_spec(spec);spec.loader.exec_module(gate);gate.validate(list(refine.values()),list(pg.values()),commit,'0.4.35')
checksums=dict((name,digest)for digest,name in(s.split()for s in(root/'SHA256SUMS').read_text().splitlines()));assert set(checksums)=={w.name for w in wheels}
records=[]
for w in wheels:
 p=pg[w.name];d=db[w.name];r=refine[w.name];assert p['pinsBeforeAfter']==d['pinsBeforeAfter']==r['pinsBeforeAfter'];observed=p['pinsBeforeAfter']
 assert p['sourceCommit']==d['sourceCommit']==r['sourceCommit']==commit and p['version']==d['version']==r['version']=='0.4.35'
 assert len(p['checks'])==4 and d['controls']==len(d['checks'])==32 and r['controls']==len(r['checks'])==30
 assert all(report['nativeAdmission']is False and report['securityReady']is False for report in[p,d,r])
 digest=hashlib.sha256(w.read_bytes()).hexdigest();assert observed['wheelSha256']==checksums[w.name]==digest
 for label,embedded in [('current-query-smoke',p),('current-databricks-smoke',d),('current-refinement-smoke',r)]:
  report=next(json.loads(f.read_text())for f in root.glob(label+'-*.json')if json.loads(f.read_text())['wheel']==w.name);assert report==embedded
 with zipfile.ZipFile(w)as z:
  native=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(native)==1
  assert observed['nativeModuleSha256']==hashlib.sha256(z.read(native[0])).hexdigest()
  assert observed['shimSha256']==hashlib.sha256(z.read('weft/__init__.py')).hexdigest()=='cd0fa27c9580742333d14e24d5ebfd74babb47d31720bb4c577278d05486f11d'
 records.append({'wheel':w.name,'pins':observed})
result={'release':release['html_url'],'publishedAt':release['published_at'],'sourceCommit':commit,'wheels':records,'githubAssetDigestsVerified':True,'embeddedSmokePinsVerified':True,'nativeSecurityAcceptance':False,'casesPromoted':[]}
(root/'verified-public.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
