"""Verify native harness annotation targets the outer SELECT, not nested/quoted text."""
import importlib.util,json
from pathlib import Path
source=Path(__file__).resolve().parents[1]/'ashlar-databricks/warehouse_capture.py'
spec=importlib.util.spec_from_file_location('capture',source);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
prefix=module.PROJECTION
cases=[
 ('SELECT 1','SELECT'+prefix+' 1'),
 ("WITH a AS (SELECT 'SELECT' AS x) SELECT x FROM a", "WITH a AS (SELECT 'SELECT' AS x) SELECT"+prefix+' x FROM a'),
 ("select COALESCE((SELECT 'it''s SELECT'), 'x')",'select'+prefix+" COALESCE((SELECT 'it''s SELECT'), 'x')"),
 ('WITH `SELECT``name` AS (SELECT 1) SELECT "SELECT" FROM `SELECT``name`','WITH `SELECT``name` AS (SELECT 1) SELECT'+prefix+' "SELECT" FROM `SELECT``name`'),
]
for sql,expected in cases:assert module.capture(sql)==expected
refused=0
for sql in ['SELECT 1 UNION ALL SELECT 2',"SELECT 'unterminated",'SELECT (1','SELECT 1)','UPDATE t SET x=1']:
 try:module.capture(sql)
 except AssertionError:refused+=1
 else:raise AssertionError('unsupported harness SQL accepted')
print(json.dumps({'status':'passed','controls':len(cases)+refused,'positive':len(cases),'refused':refused,'scope':'Harness annotation controls only; no native or backend support claim.'}))
