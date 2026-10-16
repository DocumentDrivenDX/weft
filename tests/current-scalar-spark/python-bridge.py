"""Actual local-development Spark CQP descriptors; no engine or authority admission."""
import argparse
import copy
import hashlib
import json
from pathlib import Path
import weft

p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);args=p.parse_args()
root=Path(__file__).resolve().parents[1]
text=lambda value:json.dumps(value,separators=(',',':'))
sha=lambda raw:hashlib.sha256(raw.encode()).hexdigest()
source=json.loads((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_bytes())
source.update(sql='SELECT COUNT(*) AS n FROM Resource r',parameters={})
parent=weft.prepare_application_lineage(text(source));child=parent.prepare_current_query()
assert child.source_preparation() is parent
packet=json.loads(child.native_request_json())
binding=dict(version='weft.spark.current-values/0.1-candidate',
    sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],
    sourceBindingSha256=packet['binding']['sha256'],values=[],
    scans=[dict(scan=s['scan'],target=s['record'],relation=dict(catalog='spark_catalog',schema='local_fixture',
        table='current_values',uuid='00000000-0000-4000-8000-000000000001',version='0',schemaSha256='a'*64,
        eligibleColumn='eligible',rowKeyColumn='private_key')) for s in packet['scans']])
raw_binding=text(binding);artifact=child.lower_spark_current_contributors_candidate_json(raw_binding)
compiled=json.loads(artifact)
assert compiled['version']=='weft.spark.current-scalar/0.1-candidate'
assert 'system.builtin.' not in compiled['sql']
assert compiled['authority'] is False and compiled['securityReady'] is False
carrier=text(dict(version='weft.stage-contributor-bag/0.1-candidate',tuples=[[dict(scan='s0',key=k)]for k in ['k1','k2']]))
frame=dict(version='weft.current-query.refinement-input/0.1-candidate',
    sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],
    materializationBindingSha256=sha(raw_binding),compiledArtifactSha256=sha(artifact),
    scans=[dict(scan='s0',expectedRowCount='2',rows=[dict(key=k,cells={})for k in ['k1','k2']])],
    resultRows=[['2',carrier]])
report=json.loads(child.check_spark_current_contributors_candidate_json(raw_binding,text(frame)))
assert report['matched'] is True and report['authority'] is False and report['securityReady'] is False
checks=[{'id':'same-original-spark-contributors','matched':True}]
bad=copy.deepcopy(frame);bad['resultRows'][0][0]='3'
try:child.check_spark_current_contributors_candidate_json(raw_binding,text(bad))
except ValueError as error:assert str(error)=='WFT-CURRENT-QUERY-REFINEMENT: Conditional current-query refinement refused'
else:raise AssertionError('Wrong public count accepted')
checks.append({'id':'wrong-public-result','refused':True})
for method,version in [(child.lower_spark_current_contributors_candidate_json,'weft.databricks.current-values/0.1-candidate'),
                       (child.lower_databricks_current_contributors_candidate_json,'weft.spark.current-values/0.1-candidate')]:
 wrong=dict(binding,version=version)
 try:method(text(wrong))
 except ValueError:pass
 else:raise AssertionError('Foreign dialect binding accepted')
 checks.append({'id':'closed-target:'+version,'refused':True})
assert not hasattr(child,'lower_spark_current_sum_candidate_json')
assert not hasattr(child,'check_spark_current_sum_candidate_json')
checks.append({'id':'no-spark-sum-api','refused':True})
args.output.write_text(json.dumps({'scope':'Explicit local-development Spark candidate pure native descriptors only; no engine/native permissions',
    'checks':checks,'controls':len(checks),'artifactSha256':sha(artifact),'report':report},indent=2)+'\n')
print(json.dumps({'passed':True,'controls':len(checks),'nativeAdmission':False}))
