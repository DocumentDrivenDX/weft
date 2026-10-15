import subprocess,json,time,hashlib
from pathlib import Path
out=Path('/private/tmp/weft-databricks-current-semantics-20261015');out.mkdir(exist_ok=False)
cases=[
('exact-signed64-sum',"SELECT typeof(sum(CAST(v AS DECIMAL(38,0)))) AS native_type, CAST(sum(CAST(v AS DECIMAL(38,0))) AS STRING) AS total FROM VALUES (CAST('9223372036854775807' AS BIGINT)), (CAST('9223372036854775807' AS BIGINT)) t(v)",None),
('empty-aggregate-carrier',"SELECT CAST(sum(CAST(v AS DECIMAL(38,0))) AS STRING) AS total, to_json(collect_list(array(named_struct('scan','r','key',k)))) AS contributors, CAST(count(*) AS STRING) AS n FROM (SELECT CAST(1 AS BIGINT) v, 'key' k WHERE FALSE)",None),
('complete-duplicate-bag',"SELECT CAST(count(DISTINCT v COLLATE UTF8_BINARY) AS STRING) AS n, to_json(collect_list(array(named_struct('scan','r','key',k)))) AS contributors FROM VALUES ('same','one'),('same','two'),('same','two') t(v,k)",None),
('binary-collation',"SELECT ('A' COLLATE UTF8_BINARY = 'a' COLLATE UTF8_BINARY) AS case_equal, ('x' COLLATE UTF8_BINARY = 'x ' COLLATE UTF8_BINARY) AS space_equal, (decode(unhex('c3a9'),'UTF-8') COLLATE UTF8_BINARY = decode(unhex('65cc81'),'UTF-8') COLLATE UTF8_BINARY) AS normalized_equal",None),
('exact-bound-parameter',"SELECT CAST(CAST(:wft_1 AS BIGINT) AS STRING) AS exact_value",[{'name':'wft_1','value':'9007199254740993','type':'STRING'}]),
('qualified-builtins',"SELECT system.builtin.to_json(system.builtin.array(system.builtin.named_struct('scan','r','key','one'))) AS carrier",None),
]
def api(method,path,body=None):
 args=['databricks','api',method,path,'--profile','aidev-cus','--output','json']
 if body is not None:
  p=out/'request.json';p.write_text(json.dumps(body));args+=['--json','@'+str(p)]
 r=subprocess.run(args,capture_output=True,text=True,timeout=45)
 if r.returncode:raise RuntimeError(r.stderr[:2000])
 return json.loads(r.stdout)
records=[]
for name,sql,parameters in cases:
 request={'warehouse_id':'2439e1f2e37ac563','statement':sql,'wait_timeout':'10s','disposition':'INLINE','format':'JSON_ARRAY','catalog':'client_dev'}
 if parameters:request['parameters']=parameters
 result=api('post','/api/2.0/sql/statements',request)
 end=time.monotonic()+180
 while result['status']['state'] in ('PENDING','RUNNING'):
  if time.monotonic()>end:raise TimeoutError(result['statement_id'])
  time.sleep(1);result=api('get','/api/2.0/sql/statements/'+result['statement_id'])
 record={'id':name,'sql':sql,'parameters':parameters,'response':result};records.append(record)
 (out/(name+'.json')).write_text(json.dumps(record,indent=2)+'\n')
 print(json.dumps({'id':name,'state':result['status']['state'],'rows':result.get('result',{}).get('data_array'),'error':result['status'].get('error')}),flush=True)
receipt={'scope':'Managed Databricks constant-only dialect semantics probes. No Ashlar binding, original protected sources, second caller or security qualification. No schema/table created.','profile':'aidev-cus','warehouse':'2439e1f2e37ac563','cases':records,'runnerSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest()}
(out/'receipt.json').write_text(json.dumps(receipt,indent=2)+'\n')
