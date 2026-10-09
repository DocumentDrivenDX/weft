"""Bounded read-only native token-fidelity probe; no Delta or external endpoint."""
import argparse,json
from pathlib import Path
from pyspark.sql import SparkSession
p=argparse.ArgumentParser();p.add_argument('--output',type=Path,required=True);a=p.parse_args()
if a.output.exists():raise ValueError('Fresh receipt required')
s=SparkSession.builder.master('local[1]').appName('Exact token probe').config('spark.driver.memory','512m').config('spark.ui.enabled','false').getOrCreate()
s.sparkContext.setLogLevel('ERROR')
try:
 if s.version!='4.0.1':raise ValueError('Explicit local Spark4.0.1 required')
 tokens=['9'*38,'1'+'0'*38,'9'*50,'1e3','12.5','true','null','"123"']
 rows=s.createDataFrame([('{"q":'+t+'}',t)for t in tokens],['raw','original'])
 out=[r.asDict()for r in rows.selectExpr('raw','original',"get_json_object(raw,'$.q') AS extracted", "schema_of_variant(variant_get(parse_json(raw),'$.q')) AS nativeType").collect()]
 for r in out[:3]:
  if r['extracted']!=r['original']:raise ValueError('Integer token changed')
 if out[3]['extracted']==out[3]['original']:raise ValueError('Expected observed exponent normalization differs')
 a.output.write_text(json.dumps(out,indent=2)+'\n')
finally:s.stop()
