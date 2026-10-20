# SPDX-License-Identifier: MIT OR Apache-2.0
"""Actual current owner fixture/source and AST correspondence, without authority."""
import copy, hashlib, json, os, pathlib, subprocess
root=pathlib.Path(__file__).resolve().parent
binary=os.environ['WEFT_APPLICATION_LINEAGE']
original=json.loads((root/'fixtures/current-query-source.json').read_text())
checks=[]
def sha(raw):return hashlib.sha256(raw.encode()).hexdigest()
def run(raw):return subprocess.run([binary],input=raw.encode(),capture_output=True)
def refused(raw):
 p=run(raw);assert p.returncode and not p.stdout and b'WFT-' in p.stderr
for query in original['queries']:
 name=query['id'];raw=(root/'fixtures'/f'{name}.request.json').read_text();request=json.loads(raw)
 p=run(raw);assert p.returncode==0 and not p.stderr
 assert p.stdout==(root/'fixtures'/f'{name}.report.json').read_bytes()
 report=json.loads(p.stdout);assert not report['authority'] and not report['securityReady']
 assert report['sourceRequestJson']==raw and report['sourceRequestSha256']==sha(raw)
 assert report['sqlSha256']==sha(query['sql'])
 assert report['applicationPlanSha256']==sha(report['applicationPlanJson'])
 plan=json.loads(report['applicationPlanJson'])
 assert report['modelPins']==[m['pin'] for m in request['modules']]
 for module,source in zip(request['modules'],original['closure']['documents']):
  assert json.loads(module['documentJson'])==source['document']
  assert module['pin']['sha256']==sha(module['documentJson']) and module['pin']['revision']==source['revision']
 for source,unresolved in zip(request['opaqueSources'],report['unresolvedSources']):
  assert source['sha256']==sha(source['utf8']) and unresolved['sha256']==source['sha256'] and unresolved['status']=='uninterpreted'
  if source['role'] in ['policy','catalog','ontology','owner']:assert json.loads(source['utf8'])==original[source['role']]
  if source['role']=='fixture':assert source['utf8']==(root/'fixtures/current-query-source.json').read_text()
 for use in report['uses']:
  for key in ['mode','modeSource','originalAction']:assert key not in use
  assert use['fieldIdentity']['revision']==original['closure']['documents'][0]['revision']
 for expected in query['expectedStoredUses']:
  use=next(u for u in report['uses'] if u['path']==expected['path'])
  assert use['scan']==expected['scan'] and use['field']['elementId']==expected['field']
  assert use['operator']==expected['operator'] and use['required']==expected['required']
 if name=='token-self-join':assert list(report['scans'])==['s0','s1'] and len(report['uses'])==3
 if name=='token-order-refusal':assert report['uses'][0]['required']==['order'] and plan['outputs'][0]['expression']['identity']['element']=='resourceId'
 if name=='numeric-sum-unsupported':assert report['uses'][0]['required'] is None and report['uses'][0]['propertyStatus']=='unsupported' and report['uses'][0]['outputType']['family']=='integer'
 if name=='field-free-count':assert report['uses']==[] and list(report['scans'])==['s0'] and plan['outputs'][0]['expression']['op']=='count'
 # An expectation embedded in opaque source text cannot replace actual AST uses.
 bad=copy.deepcopy(request);source=next(s for s in bad['opaqueSources'] if s['role']=='fixture');f=json.loads(source['utf8']);f['queries'][0]['expectedStoredUses']=[{'mode':'disclosed','required':[]}];source['utf8']=json.dumps(f);source['sha256']=sha(source['utf8']);r=run(json.dumps(bad));assert r.returncode==0
 altered=json.loads(r.stdout);assert altered['uses']==report['uses'] and altered['applicationPlanJson']==report['applicationPlanJson']
 for key in ['mode','uses','originalAction','required','applicationPlan','authority']:
  bad=copy.deepcopy(request);bad[key]=True;refused(json.dumps(bad))
 bad=copy.deepcopy(request);bad['modules'][0]['pin']['sha256']='0'*64;refused(json.dumps(bad))
 bad=copy.deepcopy(request);bad['opaqueSources'][0]['sha256']='0'*64;refused(json.dumps(bad))
 checks.append(name+': exact current source/hash/AST correspondence; no mode/action; opaque expectation and caller overrides cannot supply dependencies')
for raw in ['{}','[]','{"interfaceVersion":1,"interfaceVersion":2}',' '*4_000_001]:refused(raw)
print(json.dumps({'scope':'compiler derivation and unresolved current source custody only; conditional stored properties/effective disposition/native authority remain separate','checks':checks},indent=2))
