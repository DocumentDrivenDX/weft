from pathlib import Path
import hashlib,importlib,json,sys,time,unittest,zipfile
root=Path('/private/tmp/truss034-qualified-source-20261015');out=Path('/private/tmp/weft-truss034-protected-qualified-20261015');out.mkdir(exist_ok=False)
def sha(b):return hashlib.sha256(b).hexdigest()
def inventory():return {str(p.relative_to(root)):sha(p.read_bytes())for p in root.rglob('*')if p.is_file()and '__pycache__'not in p.parts and p.suffix!='.pyc'}
before=inventory();manifest=root/'packages/python/src/truss/_catalog_runtime/manifest.json';assert sha(manifest.read_bytes())=='3588cf030cb46c7d12368557e351bd767c10b4dd2e1644f7f100cec54602a2fc'
sys.path[:0]=[str(root/'packages/python/src'),str(root/'packages/python/tests')]
import weft
native=Path(importlib.import_module('weft.weft').__file__);shim=Path(weft.__file__);wheel=Path('/private/tmp/weft-release034-public-20261015/weft_sql-0.4.34-cp39-abi3-macosx_11_0_arm64.whl')
assert weft.__version__=='0.4.34';assert sha(wheel.read_bytes())=='fb0d79679c158208418199df144cfc21917b3955c382ef933980bb89645eed37'
with zipfile.ZipFile(wheel)as z:
 names=[n for n in z.namelist()if n.endswith('.so')];assert len(names)==1
 assert native.read_bytes()==z.read(names[0])and shim.read_bytes()==z.read('weft/__init__.py')
import test_current_query_runtime_native as original
cls=original.CurrentQueryRuntimeNativeTests
names=['test_public_masked_projection_with_authorized_exact_raw_predicate','test_all_denied_population_returns_empty_rows','test_empty_population_returns_empty_rows','test_protected_activation_closes_legacy_data_facades']
suite=unittest.TestSuite(cls(n)for n in names);started=time.monotonic()
with (out/'native.log').open('w')as log:result=unittest.TextTestRunner(stream=log,verbosity=2).run(suite)
after=inventory();assert before==after,'Frozen source changed'
assert result.testsRun==4 and not result.skipped and result.wasSuccessful(),(out/'native.log').read_text()
assert cls.runtime._server is None and cls.runtime._lease is None and not cls.root.exists()
loaded={str(Path(m.__file__).relative_to(root)):sha(Path(m.__file__).read_bytes())for n,m in tuple(sys.modules.items())if n.startswith('truss')and getattr(m,'__file__',None)and Path(m.__file__).is_relative_to(root)}
report={'scope':'Independent original four protected-query tests against frozen Truss source6074e761 and actual installed publicWeft034. Original fixture decisions/negative controls only; not complete B-008/all132/nativeB10/ownership/aggregate/history/revocation qualification. Truss release wheel not installed.','trussSourceCommit':'6074e761f42aecff75464c777e77c7d8d71dff2d','weftSourceCommit':'86273dd3c2b4c2b99c8d0268c9dff8537e6ce20e','weftVersion':weft.__version__,'wheelSha256':sha(wheel.read_bytes()),'moduleSha256':sha(native.read_bytes()),'shimSha256':sha(shim.read_bytes()),'runtimeManifestSha256':sha(manifest.read_bytes()),'postgresVersion':cls.info.server_version,'tests':names,'testsRun':result.testsRun,'failures':len(result.failures),'errors':len(result.errors),'skipped':len(result.skipped),'seconds':round(time.monotonic()-started,3),'sourceFiles':len(before),'sourceUnchanged':True,'loadedTrussModules':len(loaded),'serverStopped':True,'leaseReleased':True,'runtimeDirectoryRemoved':True,'nativeSecurityAcceptance':False,'casesPromoted':[],'priorAttempts':[{'status':'import failed before tests','reason':'initial package-only export omitted tracked fixture'},{'status':'four tests passed but not credited','reason':'input export was not confirmed terminal before first snapshot'}]}
(out/'source-pins.json').write_text(json.dumps(before,indent=2)+'\n');(out/'loaded-source-pins.json').write_text(json.dumps(loaded,indent=2)+'\n');(out/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
