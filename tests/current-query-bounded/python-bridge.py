# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual native candidate ABI vectors for Chromium parity; no database admission.
@covers US-009-AC3 @covers US-009-AC4
"""
import argparse,copy,hashlib,importlib,json,pathlib,subprocess,zipfile
import weft
p=argparse.ArgumentParser();p.add_argument('--wheel',type=pathlib.Path,required=True);p.add_argument('--output',type=pathlib.Path,required=True);a=p.parse_args()
root=pathlib.Path(__file__).resolve().parents[2]
def text(v):return json.dumps(v,separators=(',',':'),ensure_ascii=False)
def sha(v):return hashlib.sha256(v).hexdigest()
def pins():
 native=pathlib.Path(importlib.import_module('weft.weft').__file__);shim=pathlib.Path(weft.__file__)
 with zipfile.ZipFile(a.wheel)as z:
  names=[n for n in z.namelist()if n.endswith(('.so','.pyd'))];assert len(names)==1 and native.read_bytes()==z.read(names[0]) and shim.read_bytes()==z.read('weft/__init__.py')
 return dict(wheelSha256=sha(a.wheel.read_bytes()),nativeModuleSha256=sha(native.read_bytes()),shimSha256=sha(shim.read_bytes()))
before=pins();checks=[];vectors=[]
base=(root/'tests/current-query-designation/fixtures/bounded-unicode-strings.request.json').read_text()
def refused(label,fn):
 try:fn()
 except (ValueError,TypeError,AttributeError,UnicodeError)as e:
  assert 'PRIVATE-SENTINEL'not in str(e);checks.append(dict(id=label,refused=True));return
 raise AssertionError(label)
def make(backend,kind,sql,values,results,needle=None):
 source=json.loads(base);source.update(sql=sql,parameters={}if needle is None else dict(needle=dict(family='string',value=needle)));raw=text(source)
 parent=weft.prepare_application_lineage(raw);child=parent.prepare_current_query();assert child.source_preparation()is parent
 packet=json.loads(child.native_request_json());bindings=[]
 for op in packet['operations']:
  for operand in op['operands']:
   v=operand['input']
   if v['kind']=='field':bindings.append(dict(path=operand['path'],scan=v['scan'],field=v['identity'],domain=v['identity'],column=v['identity']['element'],comparisonScope='a'*64,mode='original-authorized'if v['originalAction']else'disclosed',originalAction=v['originalAction'],exposure=None))
 for projection in packet['projections']:bindings.append(dict(path=projection['path'],scan=projection['scan'],field=projection['field'],domain=projection['field'],column=projection['field']['element'],comparisonScope='a'*64,mode=None,originalAction=None,exposure='original'))
 relation=dict(schema='private_stage',table='current_values',eligibleColumn='eligible',rowKeyColumn='private_key')
 if backend=='databricks':relation.update(catalog='client_dev',uuid='00000000-0000-4000-8000-000000000001',version='0',schemaSha256='e'*64)
 binding=dict(version='weft.'+backend+'.current-values/'+('0.5'if backend=='postgresql'else'0.3')+'-candidate',capabilities=['bounded-unicode-scalars'],sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],sourceBindingSha256=packet['binding']['sha256'],scans=[dict(scan=s['scan'],target=s['record'],relation=relation)for s in packet['scans']],values=bindings)
 raw_binding=text(binding);name=backend+'_bounded_current_'+kind+'_candidate_json';artifact=getattr(child,'lower_'+name)(raw_binding)
 def carrier(keys):return text(dict(version='weft.stage-row-tuple/0.1-candidate',rows=[dict(scan='s0',key='k'+str(keys[0]))]))if kind=='rows'else text(dict(version='weft.stage-contributor-bag/0.1-candidate',tuples=[[dict(scan='s0',key='k'+str(k))]for k in keys]))
 result_rows=[list(public)+[carrier(keys)]for public,keys in results]
 frame=dict(version='weft.current-query.refinement-input/0.2-candidate',capabilities=['bounded-unicode-scalars'],sourceRequestSha256=packet['sourceRequestSha256'],designationSha256=packet['designationSha256'],materializationBindingSha256=sha(raw_binding.encode()),compiledArtifactSha256=sha(artifact.encode()),scans=[dict(scan='s0',expectedRowCount=str(len(values)),rows=[dict(key='k'+str(i),cells={'token':dict(family='string',value=value)})for i,value in enumerate(values)])],resultRows=result_rows)
 raw_frame=text(frame);report=getattr(child,'check_'+name)(raw_binding,raw_frame);r=json.loads(report);assert r['matched']is True and r['authority']is False and r['securityReady']is False and r['version']=='weft.current-query.refinement-result/0.2-candidate'and r['capabilities']==['bounded-unicode-scalars']
 record=dict(id=backend+'/'+kind+'/'+str(len(vectors)),request=raw,designation=child.designation_json(),nativeRequest=child.native_request_json(),binding=raw_binding,artifact=artifact,input=raw_frame,report=report,backend=backend,kind=kind);vectors.append(record);checks.append(dict(id=record['id'],matched=True))
 for version in ['0.1-candidate','0.2-candidate','0.3-candidate','0.4-candidate']:
  bad=copy.deepcopy(binding);bad['version']='weft.'+backend+'.current-values/'+version
  if bad['version']==binding['version']:continue
  refused(record['id']+'/old-binding/'+version,lambda bad=bad:getattr(child,'lower_'+name)(text(bad)))
 for label,change in [('source-drift',lambda f:f.update(sourceRequestSha256='0'*64)),('old-frame',lambda f:f.update(version='weft.current-query.refinement-input/0.1-candidate')),('null-capability',lambda f:f.update(capabilities=None)),('duplicate-capability',lambda f:f.update(capabilities=['bounded-unicode-scalars']*2)),('invalid-hidden-cell',lambda f:f['scans'][0]['rows'][0]['cells']['token'].update(value='PRIVATE-SENTINEL'+'😀'*65))]:
  bad=copy.deepcopy(frame);change(bad);refused(record['id']+'/'+label,lambda bad=bad:getattr(child,'check_'+name)(raw_binding,text(bad)))
 old='lower_'+backend+'_current_'+kind+'_candidate_json';refused(record['id']+'/old-entrypoint',lambda:getattr(child,old)(raw_binding))
 for value in [None,{},b'{}',object()]:refused(record['id']+'/non-string/'+type(value).__name__,lambda value=value:getattr(child,'lower_'+name)(value))
for backend,kind in [('postgresql','rows'),('postgresql','contributors'),('databricks','contributors')]:
 for value in ['😀','e\u0301','😀'*64]:make(backend,kind,'SELECT r.token FROM Resource r WHERE r.token=:needle',[value],[([value],[0])],value)
 make(backend,kind,'SELECT r.token FROM Resource r WHERE FALSE AND r.token=:needle LIMIT 0',['😀'],[],'😀')
 if kind=='contributors':
  make(backend,kind,'SELECT r.token FROM Resource r ORDER BY r.token DESC',['\ue000','😀'],[(['😀'],[1]),(['\ue000'],[0])])
  make(backend,kind,'SELECT r.token, COUNT(*) AS n FROM Resource r GROUP BY r.token ORDER BY r.token',['😀','😀','e\u0301'],[(['e\u0301','1'],[2]),(['😀','2'],[0,1])])
  make(backend,kind,'SELECT COUNT(DISTINCT r.token) AS n FROM Resource r',['😀','😀','e\u0301'],[(['2'],[0,1,2])])
for value in ['', '😀'*65]:
 source=json.loads(base);source.update(sql='SELECT r.token FROM Resource r WHERE FALSE AND r.token=:needle LIMIT 0',parameters=dict(needle=dict(family='string',value=value)));refused('original-bound-before-FALSE-LIMIT0/'+str(len(value)),lambda source=source:weft.prepare_application_lineage(text(source)))
assert pins()==before
# The existing bridge uses exact expected report bytes; this adds the new explicit profile.
gold=weft.prepare_application_lineage(base).prepare_current_query().designation_json()
report_path=root/'tests/current-query-designation/fixtures/bounded-unicode-strings.report.json'
assert report_path.read_text()==gold
receipt=dict(scope='Actual isolated candidate wheel and source-object compiler/checker parity vectors. Declared cells only; no database, native authority or P03 acceptance.',baseCommit=subprocess.check_output(['git','rev-parse','HEAD'],cwd=root,text=True).strip(),pinsBeforeAfter=before,checks=checks,vectors=vectors)
a.output.write_text(json.dumps(receipt,indent=2,ensure_ascii=False)+'\n');print(json.dumps(dict(controls=len(checks),vectors=len(vectors))))
