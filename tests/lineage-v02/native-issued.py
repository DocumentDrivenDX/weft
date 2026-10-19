# SPDX-License-Identifier: MIT OR Apache-2.0
"""Core-issued object + frozen public Truss query profile, no protected admission.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import argparse,hashlib,json,sys
from dataclasses import asdict
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--truss',type=Path,required=True);p.add_argument('--output',type=Path,required=True);a=p.parse_args();root=a.truss.resolve()
sys.path[:0]=[str(root/'packages/python/src'),str(root/'packages/python/examples')]
import weft
import truss._query_runtime as qr
from truss import QueryRows,QueryRefused
from adopter_workflow import transaction,success
assert qr.BACKEND['backendVersion']=='0.4.36' and weft.__version__=='0.4.30'
assert callable(weft.compile_truss_runtime_json)
issued_json=weft.ApplicationLineagePreparation.json
checks=[]
def exercise(person,admin,documents,bundle_bytes):
 bundle=json.loads(bundle_bytes)
 assert bundle['binding']['releaseSha256']=='99ae7218e0e3e68e0a3682e43949089185d967eb21bbf31deb0b8f8f6fc2de9a'
 def issue(sql):
  raw=json.dumps({'interfaceVersion':'weft-application-lineage/0.2-candidate','modules':bundle['modules'],'sql':sql,'parameters':{},'opaqueSources':[]},ensure_ascii=False,separators=(',',':'))
  prepared=weft.prepare_application_lineage(raw);report=issued_json(prepared);r=json.loads(report)
  assert r['sourceRequestJson']==raw and r['authority'] is False and r['securityReady'] is False
  assert r['modelPins']==[m['pin'] for m in bundle['modules']]
  return prepared,report,r
 cases=[('SELECT r.token FROM Resource r',(('opaque-token',),)),('SELECT COUNT(*) FROM Resource r',((1,),)),("SELECT r.resourceId FROM Resource r WHERE r.token='opaque-token'",(('r1',),)),('SELECT r.token FROM Resource r GROUP BY r.token',(('opaque-token',),)),('SELECT r.token FROM Resource r JOIN Resource s ON r.resourceId=s.resourceId',(('opaque-token',),))]
 for sql,expected in cases:
  prepared,report,r=issue(sql)
  with transaction(person,read_only=True) as (assembly,tx):result=success(assembly.query_in_transaction(tx,sql=sql,document_ids=('domain',)))
  assert isinstance(result,QueryRows) and result.rows==expected,(sql,result)
  blocked=json.loads(weft.compile_truss_runtime_json(report));assert blocked['status']=='blocked' and 'sql' not in blocked
  checks.append({'sql':sql,'native':asdict(result),'reportSha256':hashlib.sha256(report.encode()).hexdigest(),'reportNotNativeCompileRequest':True})
 sql='SELECT COUNT(DISTINCT r.token) FROM Resource r WHERE FALSE LIMIT 0';prepared,report,r=issue(sql)
 assert len(r['readRequirements']['scans'])==1 and len(r['readRequirements']['fields'])==1
 with transaction(person,read_only=True) as (assembly,tx):result=success(assembly.query_in_transaction(tx,sql=sql,document_ids=('domain',)))
 assert isinstance(result,QueryRefused),result
 checks.append({'newLineageSyntaxRefusedByOriginalNativeProfile':asdict(result),'sourceDutiesRetained':True})
 with transaction(person,read_only=True) as (assembly,tx):result=success(assembly.query_in_transaction(tx,sql='SELECT r.clear FROM Resource r',document_ids=('domain',)))
 assert isinstance(result,QueryRefused),result
 checks.append({'declarationOnlyFieldRefused':asdict(result)})
 assert not person.execute("SELECT has_table_privilege(current_user,'truss.object','SELECT')").fetchone()[0]
 checks.append({'ordinaryCallerHasRawObjectSelect':False})
path=root/'scripts/export-current-security-native-binding.py';original=path.read_bytes();source=original.decode();marker="                bundle=json.loads(bundle_bytes);binding=bundle['bindingJson'].encode('utf8')";assert source.count(marker)==1
scope={'__file__':str(path),'__name__':'original_native_fixture','exercise':exercise}
exec(compile(source.replace(marker,'                exercise(person,admin,documents,bundle_bytes)\n'+marker),str(path),'exec'),scope)
packet=root/'docs/helix/04-build/evidence/named-current-native-20261015';out=a.output.resolve()
report=scope['run'](packet/'original-source.json',packet/'original-read-scenarios.json',out);assert report['status']=='exported',report
assert path.read_bytes()==original
receipt={'status':'passed','scope':'Actual in-process core issuer and ordinary native066 public SDK compatibility. No installed policy/protection/Subject mapping or current protected authority; no B008 case promotion.','originalFixtureRunnerSha256':hashlib.sha256(original).hexdigest(),'querySdkSha256':hashlib.sha256((root/'packages/python/src/truss/_query_runtime.py').read_bytes()).hexdigest(),'weftModule':weft.__file__,'checks':checks}
(out/'issued-native-receipt.json').write_text(json.dumps(receipt,indent=2,default=str)+'\n');print(json.dumps({'status':'passed','checks':len(checks),'scope':receipt['scope']}))
