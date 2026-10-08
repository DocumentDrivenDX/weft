"""Read effective SQL warehouse ANSI setting without changing configuration.
SET key reads the value: https://docs.databricks.com/aws/en/sql/language-manual/sql-ref-syntax-aux-conf-mgmt-set
"""
import hashlib,json,os
from pathlib import Path
from native_transport import Client
OUT=Path(os.environ['WEFT_ASHLAR_EVIDENCE_OUTPUT']);client=Client(OUT)
identity_sql="SELECT to_json(current_version(), map('ignoreNullFields','false')) AS warehouse"
before=client.sql('warehouse-before-settings',identity_sql)
settings=client.sql('read-ansi-mode','SET ansi_mode')
after=client.sql('warehouse-after-settings',identity_sql)
assert before==after and len(before)==1 and len(before[0])==1
warehouse=json.loads(before[0][0])
assert set(warehouse)=={'dbr_version','dbsql_version','u_build_hash','r_build_hash'} and warehouse['dbr_version'] is None
assert all(isinstance(warehouse[k],str) and warehouse[k] for k in ['dbsql_version','u_build_hash','r_build_hash'])
assert len(settings)==1 and len(settings[0])==2 and settings[0][0].lower()=='ansi_mode' and settings[0][1].lower()=='true',settings
summary=dict(status='passed',nativeStatements=3,warehouseIdentity=warehouse,observedAnsiMode=True,harnessSha256=hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),transportSha256=hashlib.sha256(Path(__file__).with_name('native_transport.py').read_bytes()).hexdigest(),statementsSha256=hashlib.sha256((OUT/'statements.jsonl').read_bytes()).hexdigest(),source='https://docs.databricks.com/aws/en/sql/language-manual/sql-ref-syntax-aux-conf-mgmt-set',scope='Actual read-only ANSI setting observation between matching warehouse build probes. Separate statements; does not retroactively identify settings of earlier corpus queries or qualify arbitrary host sessions. No configuration, schema or data changes.')
(OUT/'summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
