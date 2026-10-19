# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual Python native descriptor and old/new ABI separation controls.
@covers US-009-AC1 @covers US-009-AC2
"""
import gc,json,pathlib,weft
root=pathlib.Path(__file__).resolve().parents[1]
parent=weft.prepare_application_lineage((root/'current-query-designation/fixtures/mixed-original-uses.request.json').read_text());child=parent.prepare_current_query();assert child.source_preparation() is parent
checks=[]
for name in ['lower_databricks_current_contributors_candidate_json','lower_databricks_current_sum_candidate_json']:
 method=getattr(child,name)
 for raw in ['{}','[]','null',child.designation_json(),child.native_request_json(),' '*4_000_001]:
  try:method(raw)
  except ValueError as e:assert str(e).startswith('WFT-CURRENT-DATABRICKS-SQL:');checks.append({'id':name+':refuse-import:'+str(len(checks)),'refused':True})
  else:raise AssertionError('Imported declaration accepted')
 descriptor=getattr(type(child),name)
 for foreign in [None,'child',{},parent,object()]:
  try:descriptor(foreign,'{}')
  except TypeError:checks.append({'id':name+':foreign-self:'+type(foreign).__name__,'refused':True})
  else:raise AssertionError('Foreign native receiver accepted')
 for kwargs in [{'binding':'{}','authority':True},{'binding':'{}','mode':'original-authorized'}]:
  try:method(**kwargs)
  except TypeError:checks.append({'id':name+':no-authority-override:'+str(len(checks)),'refused':True})
  else:raise AssertionError('Unknown override accepted')
 del method
packet=json.loads(child.native_request_json());values=[]
for operation in packet['operations']:
 for operand in operation['operands']:
  v=operand['input']
  if v['kind']=='field':values.append({'path':operand['path'],'scan':v['scan'],'field':v['identity'],'column':v['identity']['element'],'domain':v['identity'],'comparisonScope':'a'*64,'mode':'original-authorized' if v['originalAction'] else 'disclosed','originalAction':v['originalAction'],'exposure':None})
for projection in packet['projections']:
 field=projection['field'];salary=field['element']=='salary';domain=dict(field,element='salaryMaskLabel') if salary else field
 values.append({'path':projection['path'],'scan':projection['scan'],'field':field,'column':'salary_public' if salary else field['element'],'domain':domain,'comparisonScope':'a'*64,'mode':None,'originalAction':None,'exposure':'transformed' if salary else 'original'})
binding={'version':'weft.databricks.current-values/0.1-candidate','sourceRequestSha256':packet['sourceRequestSha256'],'designationSha256':packet['designationSha256'],'sourceBindingSha256':packet['binding']['sha256'],'scans':[{'scan':v['scan'],'target':v['record'],'relation':{'catalog':'client_dev','schema':'synthetic','table':'values','uuid':'00000000-0000-4000-8000-000000000000','version':'0','schemaSha256':'a'*64,'eligibleColumn':'eligible','rowKeyColumn':'row_key'}} for v in packet['scans']],'values':values}
compiled=json.loads(child.lower_databricks_current_contributors_candidate_json(json.dumps(binding)));assert compiled['authority'] is False and compiled['securityReady'] is False;checks.append({'id':'actual-same-original-candidate-emits-conditional-sql','passed':True})
for name in ['lower_postgresql_scalar_candidate_json','lower_postgresql_current_rows_candidate_json','lower_postgresql_current_contributors_candidate_json','lower_postgresql_current_sum_candidate_json']:
 try:getattr(child,name)(json.dumps(binding))
 except ValueError as e:assert str(e).startswith('WFT-CURRENT-SCALAR-SQL:');checks.append({'id':'old-postgresql-consumer-refuses:'+name,'refused':True})
 else:raise AssertionError('Databricks binding admitted by PostgreSQL')
original=child.source_preparation();del parent;gc.collect();assert child.source_preparation() is original;checks.append({'id':'original-parent-retained','passed':True})
print(json.dumps({'scope':'Actual candidate native Python descriptors/custody only; no database permissions or native security admission','controls':len(checks),'checks':checks},indent=2))
