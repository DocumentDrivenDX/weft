"""Public original named model acceptance/create/read query on frozen native70.
The explicit compiler/backend SDK adapter is a test prototype, not a released Truss.
No protected policy, ownership or aggregate security qualification.
"""
import argparse,hashlib,json,pathlib,sys
p=argparse.ArgumentParser();p.add_argument('--truss',type=pathlib.Path,required=True);p.add_argument('--model',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(parents=True,exist_ok=False)
sys.path.insert(0,str(a.truss/'packages/python/src'));sys.path.insert(0,str(a.truss/'packages/python/examples'))
import weft
import truss._query_runtime as query_runtime
assert weft.__version__=="0.4.38"
query_runtime.BACKEND=dict(query_runtime.BACKEND,backendVersion="0.4.40")
query_runtime.compiler=lambda:weft.compile_truss_runtime_json
from adopter_workflow import transaction,success
from truss.query import QueryRows
checks=[]
def exercise(runtime,person,admin,documents,identities,bundle_bytes):
 bundle=json.loads(bundle_bytes);assert bundle['binding']['releaseId']=='truss-catalog-runtime/0.70-candidate';assert bundle['binding']['releaseSha256']=='0d292c3681986b3664ff3af9927039d10cf20ef029511e33ccd7d76c566041e3'
 with transaction(person,read_only=True) as(assembly,tx):
  result=success(assembly.query_in_transaction(tx,sql="SELECT p.name FROM products p WHERE p.id='bar'",document_ids=('product-action-demo',)))
  assert isinstance(result,QueryRows),result;assert result.rows==(('old',),),result
  assert person.execute("SELECT has_table_privilege(current_user,'truss.object','SELECT')").fetchone()==(False,)
  checks.append({'id':'public-named-products-query-after-native70-create','rows':result.rows,'ordinaryDirectObjectSelect':False})
 with transaction(person,read_only=True) as(assembly,tx):
  result=success(assembly.query_in_transaction(tx,sql='SELECT COUNT(*) AS n FROM products p',document_ids=('product-action-demo',)));assert isinstance(result,QueryRows) and result.rows==((2,),),result;checks.append({'id':'public-ordinary-count','rows':result.rows})
 (a.output/'products-native-query-bundle.json').write_bytes(bundle_bytes)
source=a.truss/'scripts/export-products-native-binding.py';raw=source.read_text();needle='                    bundle_bytes=success(assembly._host._run_operation(tx,export))';assert raw.count(needle)==1
namespace={'__name__':'original_exporter_callback','__file__':str(source),'exercise':exercise}
exec(compile(raw.replace(needle,needle+'\n                exercise(runtime,person,admin,documents,identities,bundle_bytes)'),str(source),'exec'),namespace)
namespace['run'](a.model,a.output/'original-export','/opt/homebrew/bin/bun')
native=next(pathlib.Path(weft.__file__).parent.glob('*.so'));manifest=(a.truss/'packages/python/src/truss/_catalog_runtime/manifest.json').read_bytes();assert hashlib.sha256(manifest).hexdigest()=='0d292c3681986b3664ff3af9927039d10cf20ef029511e33ccd7d76c566041e3'
report={'scope':'Actual public native70 acceptance/create/query over unchanged named original Core08 products; frozen unpublished Truss source plus explicit version38/backend40 constants. No current-policy/owned/aggregate-security acceptance.','checks':checks,'weftVersion':weft.__version__,'moduleSha256':hashlib.sha256(native.read_bytes()).hexdigest(),'exporterSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'runtimeManifestSha256':hashlib.sha256(manifest).hexdigest(),'cleanup':'Original LocalPostgres exporter context exited without cleanup error'}
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
