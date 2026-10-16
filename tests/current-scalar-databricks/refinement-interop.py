"""Actual candidate CQP checker over retained managed results. Offline, no authority."""
import argparse,ast,copy,hashlib,importlib,json,pathlib,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--native-dir',type=pathlib.Path,required=True);p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(exist_ok=False)
def text(v):return json.dumps(v,separators=(',',':'),ensure_ascii=False)
def sha(v):return hashlib.sha256(v).hexdigest()
root=pathlib.Path(__file__).resolve().parents[1];corpus=root/'current-scalar-databricks/native.py'
inputs=[a.native_dir/'receipt.json',a.native_dir/'statements.json',corpus,root/'current-query-designation/fixtures/mixed-original-uses.request.json',root/'current-query-designation/fixtures/live-original-integer-sum.request.json']
captured={p:p.read_bytes()for p in inputs}
before={str(p):sha(captured[p])for p in inputs}
native=pathlib.Path(importlib.import_module('weft.weft').__file__)
with zipfile.ZipFile(a.wheel)as z:
 names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1
 assert native.read_bytes()==z.read(names[0])and pathlib.Path(weft.__file__).read_bytes()==z.read('weft/__init__.py')
tree=ast.parse(captured[corpus].decode());assignments={t.id:n.value for n in tree.body if isinstance(n,ast.Assign)for t in n.targets if isinstance(t,ast.Name)}
one=[('k1',),('k3',),('k4',),('k5',)];joined=[(x,y)for group in [['k1','k4'],['k3','k5']]for x in group for y in group]
env={'maximum':9223372036854775807,'minimum':-9223372036854775808,'one':one,'joined':joined}
cases=eval(compile(ast.Expression(assignments['cases']),str(corpus),'eval'),{'__builtins__':{'str':str}},env)
fixture=eval(compile(ast.Expression(assignments['rows']),str(corpus),'eval'),{'__builtins__':{}},env)
prior=json.loads(captured[a.native_dir/'receipt.json']);assert prior['schemaRemoved'] is True
records={r['id']:r for r in prior['checks']if 'compiled'in r};assert len(records)==len(cases)==22
statements=json.loads(captured[a.native_dir/'statements.json']);by_id={s['response']['statement_id']:s for s in statements}
sources={False:json.loads(captured[inputs[-2]]),True:json.loads(captured[inputs[-1]])}
checks=[]
def refused(method,binding,frame,label):
 try:method(binding,frame)
 except ValueError as e:
  assert 'WFT-CURRENT-QUERY-REFINEMENT'in str(e),str(e)
  assert '[private]'not in str(e)and 'opaque-'not in str(e)
 else:raise AssertionError(label)
 checks.append({'id':label,'refused':True})
for name,context,sql,parameters,expected,unordered in cases:
 parent=weft.prepare_application_lineage(text(dict(sources[context],sql=sql,parameters=parameters)));child=parent.prepare_current_query();assert child.source_preparation()is parent
 packet=json.loads(child.native_request_json());old=records[name];binding=old['materializationBinding'];binding_text=text(binding)
 lower=child.lower_databricks_current_sum_candidate_json if context else child.lower_databricks_current_contributors_candidate_json
 check=child.check_databricks_current_sum_candidate_json if context else child.check_databricks_current_contributors_candidate_json
 artifact_text=lower(binding_text);compiled=json.loads(artifact_text);assert compiled==old['compiled']
 statement=by_id[old['statementId']];assert statement['statement']==compiled['sql']
 assert statement['parameters']==[{'name':v['marker'],'value':v['value'],'type':'STRING'}for v in compiled['parameters']]
 response=statement['response'];assert response['status']['state']=='SUCCEEDED'
 manifest=response['manifest'];result=response.get('result',{});assert not manifest.get('truncated',False)
 assert not result.get('next_chunk_internal_link')and not result.get('next_chunk_external_link')
 assert result.get('row_count',0)==manifest.get('total_row_count',0)
 captured=[]
 for scan in packet['scans']:
  columns={v['column']for v in binding['values']if v['scan']==scan['scan']}
  rows=[]
  for eligible,key,rid,salary,masked,token in fixture:
   if not eligible:continue
   original=dict(resourceId=rid,salary=salary,salary_public=masked,token=token)
   rows.append(dict(key=key,cells={c:dict(family='integer'if c=='salary'else'string',value=str(original[c]))for c in columns}))
  captured.append(dict(scan=scan['scan'],expectedRowCount=str(len(rows)),rows=rows))
 frame=dict(version='weft.current-query.refinement-input/0.1-candidate',sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],materializationBindingSha256=sha(binding_text.encode()),compiledArtifactSha256=sha(artifact_text.encode()),scans=captured,resultRows=result.get('data_array',[]))
 raw=text(frame);report=json.loads(check(binding_text,raw))
 assert report['matched']is True and report['authority']is False and report['securityReady']is False and report['nativeAdmission']=='required'
 assert report['refinementInputSha256']==sha(raw.encode())
 for anchor in ['sourceRequestSha256','designationSha256','materializationBindingSha256','compiledArtifactSha256']:assert report[anchor]==frame[anchor]
 checks.append({'id':name,'matchedRetainedManagedResult':True})
 for anchor in ['sourceRequestSha256','designationSha256','materializationBindingSha256','compiledArtifactSha256']:
  bad=copy.deepcopy(frame);bad[anchor]='0'*64;refused(check,binding_text,text(bad),name+'-'+anchor)
 bad=copy.deepcopy(frame);bad['scans'][0]['expectedRowCount']='3';refused(check,binding_text,text(bad),name+'-declared-count')
 bad=copy.deepcopy(frame);bad['scans'][0]['rows'][0]['cells']['unexpected']={'family':'string','value':'secret'};refused(check,binding_text,text(bad),name+'-unknown-cell')
 refused(check,binding_text,raw[:-1]+',"version":"duplicate"}',name+'-duplicate-json')
 refused(check,binding_text+' ',raw,name+'-binding-byte-anchor')
 if frame['resultRows']:
  bad=copy.deepcopy(frame);bad['resultRows'][0][0]='wrong';refused(check,binding_text,text(bad),name+'-wrong-public')
 if name=='joined-distinct':
  wrong_sql=compiled['sql'].replace('`wft_s1`.`private_key`','`wft_s0`.`private_key`');wrong=next(s for s in statements if s['statement']==wrong_sql)
  bad=copy.deepcopy(frame);bad['resultRows']=wrong['response']['result']['data_array'];assert bad['resultRows'][0][0]==frame['resultRows'][0][0]
  refused(check,binding_text,text(bad),'actual-native-same-public-count-wrong-contributor')
 # A forged empty population/count/result can conditionally match. It proves
 # neither native completeness nor authority and is deliberately not refused.
 if name=='count':
  class StrSubclass(str):pass
  class Coercion:
   def __str__(self):raise AssertionError('Unexpected coercion')
  for i,value in enumerate([None,{},b'{}',StrSubclass(raw),Coercion()]):
   refused(check,binding_text,value,'exact-input-string-'+str(i))
   refused(check,value,raw,'exact-binding-string-'+str(i))
  empty=copy.deepcopy(frame);empty['scans'][0]['rows']=[];empty['scans'][0]['expectedRowCount']='0';empty['resultRows']=[['0',text(dict(version='weft.stage-contributor-bag/0.1-candidate',tuples=[]))]]
  r=json.loads(check(binding_text,text(empty)));assert r['matched']and not r['authority'];checks.append({'id':'consistent-forged-world-conditional-only','authority':False})
assert before=={str(p):sha(p.read_bytes())for p in inputs}
report={'inputPinsBeforeAfter':before,'scope':'Actual installed candidate CQP checker over retained managed results and independently captured original fixture populations. Offline correspondence only; no fresh native execution, completeness/authority admission or guarded release.','version':weft.__version__,'wheelSha256':sha(a.wheel.read_bytes()),'nativeModuleSha256':sha(native.read_bytes()),'runnerSha256':sha(pathlib.Path(__file__).read_bytes()),'checks':checks,'nativeSecurityAcceptance':False,'casesPromoted':[]}
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'checks':len(checks),'managedCases':len(cases),'nativeSecurityAcceptance':False}))
