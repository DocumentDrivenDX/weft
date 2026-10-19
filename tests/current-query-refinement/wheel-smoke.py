# SPDX-License-Identifier: MIT OR Apache-2.0
"""Exact-wheel conditional checker surface. Declared fixture values, no native admission."""
import argparse,copy,hashlib,importlib,json,pathlib,subprocess,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[2]
def text(v):return json.dumps(v,separators=(',',':'))
def sha(b):return hashlib.sha256(b).hexdigest()
def pins():
 native=pathlib.Path(importlib.import_module('weft.weft').__file__)
 with zipfile.ZipFile(a.wheel)as z:
  names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1
  assert native.read_bytes()==z.read(names[0])and pathlib.Path(weft.__file__).read_bytes()==z.read('weft/__init__.py')
 return dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(pathlib.Path(weft.__file__).read_bytes()))
before=pins();checks=[]
for backend,kind,version in [('postgresql','rows','0.2'),('postgresql','contributors','0.3'),('postgresql','sum','0.4'),('databricks','contributors','0.1'),('databricks','sum','0.2')]:
 source=json.loads((root/'tests/current-query-designation/fixtures'/('live-original-integer-sum.request.json'if kind=='sum'else'mixed-original-uses.request.json')).read_text())
 source.update(sql='SELECT r.token FROM Resource r'if kind=='rows'else'SELECT SUM(r.salary) AS n FROM Resource r'if kind=='sum'else'SELECT COUNT(*) AS n FROM Resource r',parameters={})
 parent=weft.prepare_application_lineage(text(source));child=parent.prepare_current_query();assert child.source_preparation()is parent
 packet=json.loads(child.native_request_json());values=[]
 for operation in packet['operations']:
  for operand in operation['operands']:
   v=operand['input']
   if v['kind']=='field':values.append(dict(path=operand['path'],scan=v['scan'],field=v['identity'],domain=v['identity'],column=v['identity']['element'],comparisonScope='a'*64,mode='original-authorized'if v['originalAction']else'disclosed',originalAction=v['originalAction'],exposure=None))
 for projection in packet['projections']:
  values.append(dict(path=projection['path'],scan=projection['scan'],field=projection['field'],domain=projection['field'],column=projection['field']['element'],comparisonScope='a'*64,mode=None,originalAction=None,exposure='original'))
 relation=dict(schema='private_stage',table='current_values',eligibleColumn='eligible',rowKeyColumn='private_key')
 if backend=='databricks':relation.update(catalog='client_dev',uuid='00000000-0000-4000-8000-000000000001',version='0',schemaSha256='e'*64)
 binding=dict(version='weft.'+backend+'.current-values/'+version+'-candidate',sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],sourceBindingSha256=packet['binding']['sha256'],scans=[dict(scan=s['scan'],target=s['record'],relation=relation)for s in packet['scans']],values=values)
 raw_binding=text(binding);artifact=getattr(child,'lower_'+backend+'_current_'+kind+'_candidate_json')(raw_binding)
 carrier=lambda keys:text(dict(version='weft.stage-row-tuple/0.1-candidate',rows=[dict(scan='s0',key=keys[0])]))if kind=='rows'else text(dict(version='weft.stage-contributor-bag/0.1-candidate',tuples=[[dict(scan='s0',key=k)]for k in keys]))
 rows=[]
 for key,token,salary in [('k1','x',9223372036854775807),('k2','y',-9223372036854775808)]:
  cells={v['column']:dict(family='integer'if v['column']=='salary'else'string',value=str(salary)if v['column']=='salary'else token)for v in values}
  rows.append(dict(key=key,cells=cells))
 results=[['x',carrier(['k1'])],['y',carrier(['k2'])]]if kind=='rows'else[['-1'if kind=='sum'else'2',carrier(['k1','k2'])]]
 frame=dict(version='weft.current-query.refinement-input/0.1-candidate',sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],materializationBindingSha256=sha(raw_binding.encode()),compiledArtifactSha256=sha(artifact.encode()),scans=[dict(scan='s0',expectedRowCount='2',rows=rows)],resultRows=results)
 method=getattr(child,'check_'+backend+'_current_'+kind+'_candidate_json');raw=text(frame);report=json.loads(method(raw_binding,raw))
 assert set(report)=={'version','matched','authority','securityReady','nativeAdmission','sourceRequestSha256','designationSha256','materializationBindingSha256','compiledArtifactSha256','refinementInputSha256'}
 assert report['version']=='weft.current-query.refinement-result/0.1-candidate'and report['matched']is True and report['authority']is False and report['securityReady']is False and report['nativeAdmission']=='required'
 assert report['refinementInputSha256']==sha(raw.encode())
 checks.append({'id':backend+'/'+kind,'matched':True})
 for label,change in [('foreign-source',lambda f:f.update(sourceRequestSha256='0'*64)),('wrong-public',lambda f:f['resultRows'][0].__setitem__(0,'wrong')),('duplicate-key',lambda f:f['scans'][0]['rows'][1].update(key='k1')),('missing-input',lambda f:f['scans'][0]['rows'].pop()),('unknown-frame',lambda f:f.update(extra=True))]:
  bad=copy.deepcopy(frame);change(bad)
  try:method(raw_binding,text(bad))
  except ValueError as e:assert str(e)=='WFT-CURRENT-QUERY-REFINEMENT: Conditional current-query refinement refused'
  else:raise AssertionError(label)
  checks.append({'id':backend+'/'+kind+'/'+label,'refused':True})
assert pins()==before
commit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip()if(root/'.git').exists()else None
report=dict(version=weft.__version__,sourceCommit=commit,wheel=a.wheel.name,pinsBeforeAfter=before,checks=checks,controls=len(checks),nativeAdmission=False,securityReady=False)
a.output.write_text(json.dumps(report,indent=2)+'\n');print(json.dumps({'controls':len(checks),'nativeAdmission':False}))
