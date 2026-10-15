import sys,json
from pathlib import Path
from dataclasses import asdict
root=Path('/private/tmp/truss066-native-frozen')
sys.path[:0]=['/private/tmp/weft-truss066-installed',str(root/'packages/python/src'),str(root/'packages/python/examples')]
import truss._query_runtime as qr
qr.BACKEND=dict(qr.BACKEND,backendVersion='0.4.36')
from truss import QueryRows,QueryRefused
checks=[]
def exercise(person,admin,documents,bundle_bytes):
 from adopter_workflow import transaction,success
 cases=[('SELECT r.token FROM Resource r',(('opaque-token',),)),('SELECT COUNT(*) FROM Resource r',((1,),)),("SELECT r.resourceId FROM Resource r WHERE r.token='opaque-token'",(('r1',),)),('SELECT r.token FROM Resource r GROUP BY r.token',(('opaque-token',),)),('SELECT r.token FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId',(('opaque-token',),))]
 for sql,expected in cases:
  with transaction(person,read_only=True) as (assembly,tx):
   result=success(assembly.query_in_transaction(tx,sql=sql,document_ids=('domain',)))
   if not isinstance(result,QueryRows) or result.rows!=expected:raise AssertionError((sql,result,expected))
   checks.append({'sql':sql,'result':asdict(result)})
 with transaction(person,read_only=True) as (assembly,tx):
  result=success(assembly.query_in_transaction(tx,sql='SELECT r.clear FROM Resource r',document_ids=('domain',)))
  assert isinstance(result,QueryRefused),result
  checks.append({'declarativeOnlyFieldRefused':asdict(result)})
 assert not person.execute("SELECT has_table_privilege(current_user,'truss.object','SELECT')").fetchone()[0]
 checks.append({'ordinaryCallerHasRawObjectSelect':False})
path=root/'scripts/export-current-security-native-binding.py'
source=path.read_text();marker="                bundle=json.loads(bundle_bytes);binding=bundle['bindingJson'].encode('utf8')"
assert source.count(marker)==1
scope={'__file__':str(path),'__name__':'frozen_native_export','exercise':exercise}
exec(compile(source.replace(marker,'                exercise(person,admin,documents,bundle_bytes)\n'+marker),str(path),'exec'),scope)
packet=root/'docs/helix/04-build/evidence/named-current-native-20261015'
out=Path(sys.argv[1]);report=scope['run'](packet/'original-source.json',packet/'original-read-scenarios.json',out)
assert report['status']=='exported',report
receipt={'status':'passed','scope':'ordinary catalog/query integration; no current protected-policy authority','trussSource':'50307788','candidateSdkMetadataOverride':{'backendVersion':'0.4.36'},'weftVersion':'0.4.30','checks':checks}
(out/'weft-query-receipt.json').write_text(json.dumps(receipt,indent=2,default=str)+'\n')
print(json.dumps(receipt,indent=2,default=str))
