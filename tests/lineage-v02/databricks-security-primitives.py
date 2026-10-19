# SPDX-License-Identifier: MIT OR Apache-2.0
"""Disposable UC/Delta platform primitives, never compiled Weft/native acceptance.
@covers US-009-AC2 @covers US-009-AC3
Uses one actual authenticated caller. A second identity and native issuer are separate.
"""
import argparse,hashlib,json,pathlib,re,subprocess,time,uuid
p=argparse.ArgumentParser();p.add_argument('--profile',required=True);p.add_argument('--warehouse',required=True);p.add_argument('--catalog',required=True);p.add_argument('--output',required=True);args=p.parse_args()
if not re.fullmatch(r'[a-z][a-z0-9_]{0,62}',args.catalog):raise ValueError('Exact bounded catalog required')
cli='/opt/homebrew/bin/databricks';schema='weft_b008_20261015_'+uuid.uuid4().hex[:8];scope='`'+args.catalog+'`.`'+schema+'`';table=scope+'.`resources`';created=False
receipt={'scope':'One real caller, disposable Unity Catalog/Delta platform primitives only; not Weft compiler/native admission, second-caller security, Ashlar layout or retention qualification','namespace':args.catalog+'.'+schema,'checks':[],'cleanup':'not-created'}
output=pathlib.Path(args.output)
def save():output.write_text(json.dumps(receipt,indent=2)+'\n')
def api(method,path,body=None):
 command=[cli,'api',method,path,'--profile',args.profile,'--output','json']
 if body is not None:command+=['--json',json.dumps(body)]
 q=subprocess.run(command,capture_output=True,timeout=40)
 if q.returncode:raise RuntimeError('DATABRICKS_API_FAILED')
 return json.loads(q.stdout)
def execute(label,statement,expected=None):
 r=api('post','/api/2.0/sql/statements',{'warehouse_id':args.warehouse,'statement':statement,'wait_timeout':'10s','on_wait_timeout':'CONTINUE','row_limit':32})
 ident=r.get('statement_id');deadline=time.monotonic()+90
 while r.get('status',{}).get('state') in ('PENDING','RUNNING'):
  if time.monotonic()>deadline:
   api('post','/api/2.0/sql/statements/'+ident+'/cancel',{});raise RuntimeError('DATABRICKS_STATEMENT_TIMEOUT')
  time.sleep(1);r=api('get','/api/2.0/sql/statements/'+ident)
 state=r.get('status',{}).get('state');row={'id':label,'statementSha256':hashlib.sha256(statement.encode()).hexdigest(),'state':state}
 if state!='SUCCEEDED':
  row['errorCode']=r.get('status',{}).get('error',{}).get('error_code');receipt['checks'].append(row);save();raise RuntimeError('DATABRICKS_STATEMENT_REFUSED:'+label)
 manifest=r.get('manifest',{});result=r.get('result',{});data=result.get('data_array',[])
 if manifest.get('truncated') or result.get('next_chunk_index') is not None:raise RuntimeError('UNBOUNDED_RESULT')
 if expected is not None:
  if data!=expected:raise RuntimeError('RESULT_MISMATCH:'+label)
  row['resultMatched']=True
 receipt['checks'].append(row);save();return data
save()
try:
 warehouse=api('get','/api/2.0/sql/warehouses/'+args.warehouse)
 if warehouse.get('state')!='RUNNING':raise RuntimeError('WAREHOUSE_NOT_ALREADY_RUNNING')
 execute('verify-fresh-namespace',"SHOW SCHEMAS IN `"+args.catalog+"` LIKE '"+schema+"'",[])
 created=True;receipt['cleanup']='pending';save()
 execute('create-disposable-schema',"CREATE SCHEMA "+scope+" COMMENT 'Disposable Weft B-008 platform test; cleanup required'")
 execute('create-managed-delta-table','CREATE TABLE '+table+' (resource_id STRING NOT NULL, owner_id STRING NOT NULL, salary BIGINT NOT NULL, token STRING NOT NULL) USING DELTA')
 execute('insert-two-synthetic-resources',"INSERT INTO "+table+" VALUES ('visible',current_user(),100,'opaque-a'),('hidden','weft-unassigned-"+uuid.uuid4().hex+"',200,'opaque-b')")
 execute('create-row-filter','CREATE FUNCTION '+scope+'.`row_filter`(resource_owner STRING) RETURNS BOOLEAN RETURN resource_owner = current_user()')
 execute('install-row-filter','ALTER TABLE '+table+' SET ROW FILTER '+scope+'.`row_filter` ON (owner_id)')
 execute('authenticated-row-filter','SELECT resource_id,salary,token FROM '+table+' ORDER BY resource_id',[['visible','100','opaque-a']])
 execute('filtered-count-sum-distinct','SELECT COUNT(*),SUM(salary),COUNT(DISTINCT token) FROM '+table,[['1','100','1']])
 execute('filtered-self-join','SELECT COUNT(*) FROM '+table+' a JOIN '+table+' b ON a.token=b.token',[['1']])
 execute('hidden-row-predicate',"SELECT COUNT(*) FROM "+table+" WHERE resource_id='hidden'",[['0']])
 execute('create-constant-mask','CREATE FUNCTION '+scope+'.`salary_mask`(value BIGINT) RETURNS BIGINT RETURN CAST(-1 AS BIGINT)')
 execute('install-column-mask','ALTER TABLE '+table+' ALTER COLUMN salary SET MASK '+scope+'.`salary_mask`')
 execute('masked-projection','SELECT salary FROM '+table,[['-1']])
 execute('masked-predicate','SELECT COUNT(*) FROM '+table+' WHERE salary=100',[['0']])
 execute('masked-group-sum','SELECT salary,COUNT(*),SUM(salary) FROM '+table+' GROUP BY salary',[['-1','1','-1']])
 receipt['status']='passed'
except Exception as error:
 receipt['status']='failed';receipt['failure']=str(error)
finally:
 if created:
  try:
   execute('drop-owned-test-schema','DROP SCHEMA IF EXISTS '+scope+' CASCADE')
   execute('verify-namespace-removed',"SHOW SCHEMAS IN `"+args.catalog+"` LIKE '"+schema+"'",[])
   receipt['cleanup']='namespace-removed-managed-object-physical-retention-not-qualified'
  except Exception as error:receipt['cleanup']='FAILED';receipt['cleanupFailure']=str(error)
 save()
print(json.dumps({'status':receipt.get('status'),'checks':len(receipt['checks']),'cleanup':receipt['cleanup']}))
if receipt.get('status')!='passed' or receipt['cleanup']=='FAILED':raise SystemExit(1)
