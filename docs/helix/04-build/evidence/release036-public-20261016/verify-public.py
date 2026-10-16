from pathlib import Path
import json,hashlib,zipfile,runpy
r=Path('/private/tmp/weft036-public-20261016');sha=lambda b:hashlib.sha256(b).hexdigest();meta=json.loads((r/'release-metadata.json').read_text());assert meta['tag_name']=='v0.4.36'and meta['draft']is False and len(meta['assets'])==27
for a in meta['assets']:
 p=r/a['name'];assert p.is_file()and p.stat().st_size==a['size'];assert a['digest']=='sha256:'+sha(p.read_bytes()),a['name']
for line in(r/'SHA256SUMS').read_text().splitlines():
 digest,name=line.split(maxsplit=1);assert sha((r/name.lstrip('*')).read_bytes())==digest
commit='a14640f2bb288e58de0c9846fb10162e3e4030fc';pins=json.loads((r/'release-pins.json').read_text());assert pins['tag']=='v0.4.36'and pins['commit']==commit and pins['version']=='0.4.36';assert pins['trussDecimal12Backend']['backendVersion']=='0.4.39';assert pins['trussQueryAbiBackend']['backendVersion']=='0.4.38'
run=json.loads((r/'tagged-run.json').read_text());assert run['headSha']==commit and run['status']=='completed'and run['conclusion']=='success';assert len(run['jobs'])==6 and all(j['conclusion']=='success'for j in run['jobs'])
wheels=list(r.glob('*.whl'));assert len(wheels)==5;payloads={}
for w in wheels:
 with zipfile.ZipFile(w)as z:
  native=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(native)==1
  payloads[w.name]={'wheelSha256':sha(w.read_bytes()),'nativeModuleSha256':sha(z.read(native[0])),'shimSha256':sha(z.read('weft/__init__.py'))}
reports={}
for prefix,total in [('current-query-smoke-',4),('current-databricks-smoke-',32),('current-refinement-smoke-',30),('truss-decimal12-smoke-',22)]:
 items=[json.loads(p.read_text())for p in r.glob(prefix+'*.json')];assert len(items)==5
 for x in items:
  assert x['version']=='0.4.36'and x['sourceCommit']==commit and x['pinsBeforeAfter']==payloads[x['wheel']];assert x.get('controls',len(x.get('checks',[])))==total
 reports[prefix]=items
runpy.run_path('/private/tmp/weft-wheel-naming-gate-20261016/scripts/release/verify-decimal12-smoke.py')['validate'](reports['truss-decimal12-smoke-'],reports['current-query-smoke-'],commit,'0.4.36')
runpy.run_path('/private/tmp/weft-decimal12-reconciled-20261016/scripts/release/verify-refinement-smoke.py')['validate'](reports['current-refinement-smoke-'],reports['current-query-smoke-'],commit,'0.4.36')
out={'tag':'v0.4.36','sourceCommit':commit,'publishedAt':meta['published_at'],'assetsVerified':27,'wheels':payloads,'platformJobsPassed':5,'publishPassed':True,'reportsVerified':20,'nativeCasePromotions':0,'scope':'Actual published standard wheel cohort; no bounded String opt-in feature or native B008 acceptance implied'};(r/'verified-public.json').write_text(json.dumps(out,indent=2)+'\n');print(json.dumps(out,indent=2))
