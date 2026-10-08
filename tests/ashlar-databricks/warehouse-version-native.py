"""Read-only warehouse identity probe, distinct from Spark version().
Official semantics: https://docs.databricks.com/aws/en/sql/language-manual/functions/current_version
No corpus execution, production qualification, or provisioning.
"""
import hashlib,json,os
from pathlib import Path
from native_transport import Client
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT'])
client=Client(OUT)
rows=client.sql('warehouse-version',"SELECT version() AS spark_version, current_version().dbsql_version AS dbsql_version, current_version().u_build_hash AS u_build_hash, current_version().r_build_hash AS r_build_hash, current_version().dbr_version AS dbr_version")
assert len(rows)==1 and len(rows[0])==5,rows
spark,warehouse,u_build,r_build,runtime=rows[0]
assert all(isinstance(v,str) and v for v in [spark,warehouse,u_build,r_build]),rows
assert runtime is None,rows
summary={'status':'passed','nativeStatements':1,'sparkVersion':spark,'dbsqlVersion':warehouse,'uBuildHash':u_build,'rBuildHash':r_build,'dbrVersion':runtime,'harnessSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'transportSha256':hashlib.sha256((ROOT/'tests/ashlar-databricks/native_transport.py').read_bytes()).hexdigest(),'statementsSha256':hashlib.sha256((OUT/'statements.jsonl').read_bytes()).hexdigest(),'sources':['https://docs.databricks.com/aws/en/sql/language-manual/functions/version','https://docs.databricks.com/aws/en/sql/language-manual/functions/current_version'],'scope':'Actual read-only current warehouse identity. Does not retroactively pin earlier corpus receipts, capture their settings, or qualify support.'}
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n')
print(json.dumps(summary))
