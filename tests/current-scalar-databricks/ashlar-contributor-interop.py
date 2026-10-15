"""Actual public Weft producer + retained managed wire rows + original Ashlar decoder.
Offline correspondence; neither source/authority admission nor a fresh native run.
"""
import argparse,ast,hashlib,importlib,importlib.util,json,pathlib,sys,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--ashlar-source',type=pathlib.Path,required=True);p.add_argument('--native-dir',type=pathlib.Path,required=True);p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args();a.output.mkdir(exist_ok=False)
def sha(b):return hashlib.sha256(b).hexdigest()
root=pathlib.Path(__file__).resolve().parents[1];corpus=root/'current-scalar-databricks/native.py'
inputs=[a.ashlar_source,a.native_dir/'receipt.json',a.native_dir/'statements.json',corpus,root/'current-query-designation/fixtures/mixed-original-uses.request.json',root/'current-query-designation/fixtures/live-original-integer-sum.request.json']
before={str(p):sha(p.read_bytes())for p in inputs}
spec=importlib.util.spec_from_file_location('original_ashlar_contributors',a.ashlar_source);decoder=importlib.util.module_from_spec(spec);sys.modules[spec.name]=decoder;spec.loader.exec_module(decoder)
native=pathlib.Path(importlib.import_module('weft.weft').__file__);shim=pathlib.Path(weft.__file__)
with zipfile.ZipFile(a.wheel)as z:
 modules=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(modules)==1
 assert native.read_bytes()==z.read(modules[0])and shim.read_bytes()==z.read('weft/__init__.py')
tree=ast.parse(corpus.read_text());node=next(n for n in tree.body if isinstance(n,ast.Assign)and any(isinstance(t,ast.Name)and t.id=='cases'for t in n.targets))
one=[('k1',),('k3',),('k4',),('k5',)];joined=[(x,y)for group in [['k1','k4'],['k3','k5']]for x in group for y in group]
# These are the original predeclared fixture oracle expressions, not observed rows.
cases=eval(compile(ast.Expression(node.value),str(corpus),'eval'),{'__builtins__':{'str':str}},{'maximum':9223372036854775807,'minimum':-9223372036854775808,'one':one,'joined':joined})
prior=json.loads((a.native_dir/'receipt.json').read_text());assert prior['schemaRemoved'] is True
records={r['id']:r for r in prior['checks']if 'compiled'in r};assert len(records)==len(cases)==22
statements=json.loads((a.native_dir/'statements.json').read_text());by_id={s['response']['statement_id']:s for s in statements}
sources={False:json.loads(inputs[-2].read_text()),True:json.loads(inputs[-1].read_text())};checks=[];decoded_rows=0

def decode_expected(keys,scans):
 raw=json.dumps({'version':decoder.VERSION,'tuples':[[{'scan':s,'key':k}for s,k in zip(scans,row)]for row in keys]},separators=(',',':')).encode()
 assert all(len(row)==len(scans)for row in keys)
 return decoder.decode_current_contributors(raw,expected_scans=scans)

def native_rows(record,width):
 response=record['response'];assert response['status']['state']=='SUCCEEDED'
 manifest=response['manifest'];result=response.get('result',{});assert not manifest.get('truncated',False)
 assert not result.get('next_chunk_internal_link')and not result.get('next_chunk_external_link')
 assert result.get('row_count',0)==manifest.get('total_row_count',0)
 assert manifest['schema']['column_count']==width+1 and all(c['type_name']=='STRING'for c in manifest['schema']['columns'])
 rows=result.get('data_array',[]);assert all(len(row)==width+1 for row in rows);return rows

for name,context,sql,parameters,expected,unordered in cases:
 parent=weft.prepare_application_lineage(json.dumps(dict(sources[context],sql=sql,parameters=parameters),separators=(',',':')));child=parent.prepare_current_query();assert child.source_preparation()is parent
 packet=json.loads(child.native_request_json());scans=tuple(s['scan']for s in packet['scans'])
 old=records[name];method=child.lower_databricks_current_sum_candidate_json if context else child.lower_databricks_current_contributors_candidate_json
 compiled=json.loads(method(json.dumps(old['materializationBinding'],separators=(',',':'))));assert compiled==old['compiled']
 record=by_id[old['statementId']];assert record['statement']==compiled['sql']
 assert record['parameters']==[{'name':v['marker'],'value':v['value'],'type':'STRING'}for v in compiled['parameters']]
 width=len(compiled['resultColumns']);rows=native_rows(record,width)
 assert len(rows)==len(expected);remaining=[(tuple(public),decode_expected(bag,scans))for public,bag in expected]
 for row in rows:
  actual=decoder.decode_current_contributors(row[-1].encode('utf-8'),expected_scans=scans);public=tuple(row[:width]);match=None
  for i,(wanted,independent)in enumerate(remaining if unordered else remaining[:1]):
   if public!=wanted:continue
   try:decoder.require_matching_contributors(actual,independent)
   except ValueError:continue
   match=i;break
  assert match is not None,(name,public);remaining.pop(match);decoded_rows+=1
 assert not remaining
 checks.append({'id':name,'identicalCompiledArtifact':True,'managedStatementId':old['statementId'],'rows':len(rows),'matchedOriginalCompleteOracle':True})
 if name=='joined-distinct':
  wrong=compiled['sql'].replace('`wft_s1`.`private_key`','`wft_s0`.`private_key`');bad=next(s for s in statements if s['statement']==wrong)
  bad_rows=native_rows(bad,width);assert len(bad_rows)==len(rows)==1 and bad_rows[0][:width]==rows[0][:width]
  bad_bag=decoder.decode_current_contributors(bad_rows[0][-1].encode('utf-8'),expected_scans=scans);independent=decode_expected(expected[0][1],scans)
  try:decoder.require_matching_contributors(bad_bag,independent)
  except ValueError:checks.append({'id':'actual-managed-same-count-wrong-contributor-refused','samePublicResult':True,'refused':True})
  else:raise AssertionError('Actual wrong managed contributor bag accepted')
assert before=={str(p):sha(p.read_bytes())for p in inputs}
report={'scope':'Offline original Ashlar decoder consumes retained actual managed SQL JSON carriers; actual public Weft34 recompiles all22 complete artifacts identically. Expected bags come from original predeclared independent fixture oracles, not actual returned bags. No fresh database run, genuine publication/native admission, caller/current-policy authority or guarded release.','weftVersion':weft.__version__,'wheelSha256':sha(a.wheel.read_bytes()),'moduleSha256':sha(native.read_bytes()),'shimSha256':sha(shim.read_bytes()),'inputPins':before,'decodedManagedRows':decoded_rows,'checks':checks,'nativeSecurityAcceptance':False,'casesPromoted':[]}
(a.output/'receipt.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'checks':len(checks),'decodedManagedRows':decoded_rows,'sameCountWrongBagRefused':True}))
