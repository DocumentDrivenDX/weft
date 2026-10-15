from pathlib import Path
import subprocess,hashlib,json,zipfile
root=Path('/private/tmp/weft-truss033-independent-20261015');repo='/Users/erik/Projects/truss';revision='edebb67f5f56d6c54ac879246c0c035e0a39638a'
def sha(b):return hashlib.sha256(b).hexdigest()
files=subprocess.check_output(['git','-C',repo,'ls-tree','-r','--name-only',revision,'packages/python']).decode().splitlines()
pins={}
for name in files:
 data=(root/name).read_bytes();original=subprocess.check_output(['git','-C',repo,'show',revision+':'+name]);assert data==original,name
 pins[name]=sha(data)
wheel=Path('/private/tmp/weft-release033-ci-arm/weft_sql-0.4.33-cp39-abi3-macosx_11_0_arm64.whl');proof=json.loads(wheel.with_name('current-query-smoke-aarch64-apple-darwin.json').read_text())
assert sha(wheel.read_bytes())==proof['pinsBeforeAfter']['wheelSha256']
installed=Path('/private/tmp/weft-release033-ci-arm-installed/weft')
with zipfile.ZipFile(wheel) as z:
 modules=[n for n in z.namelist() if n.endswith('.so')];assert len(modules)==1
 assert (installed/Path(modules[0]).name).read_bytes()==z.read(modules[0])
 assert (installed/'__init__.py').read_bytes()==z.read('weft/__init__.py')
receipt={'scope':'Independent Weft-hosted replay of original committed Truss ABI2 workflow tests against exact standard CI wheel; finite native evidence only','trussCommit':revision,'weftSourceCommit':proof['sourceCommit'],'wheel':proof['wheel'],'pins':proof['pinsBeforeAfter'],'sourceFilesVerifiedAgainstCommit':len(pins),'sourceSha256':pins,'command':'env PYTHONPATH=packages/python/src:packages/python/tests:/private/tmp/weft-release033-ci-arm-installed TRUSS_BUN_EXECUTABLE=/opt/homebrew/bin/bun /private/tmp/truss-clean-install-20261013/bin/python -m unittest test_current_query_runtime_native -v','workingDirectory':str(root),'session':2546,'exitCode':0,'tests':4,'seconds':36.057,'results':{'test_all_denied_population_returns_empty_rows':'ok','test_empty_population_returns_empty_rows':'ok','test_protected_activation_closes_legacy_data_facades':'ok','test_public_masked_projection_with_authorized_exact_raw_predicate':'ok'},'limitations':['Original four tests reused without changes; no independent expansion of the test oracle','Before execution a trimmed export failed import for a missing test fixture; no native tests ran in that attempt','All original 132 B-008 cases remain required; no count promoted by this replay','No ABI3/4 native aggregate, managed Databricks, concurrent revocation or production all-path qualification','No release tag or publication performed']}
Path('/private/tmp/weft033-native-replay.json').write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'tests':4,'sourceFiles':len(pins),'exactWheelVerified':True}))
