"""Native Graph requirements bridge custody and feature isolation; no admission."""
import copy
import gc
import hashlib
import json
import pickle
import sys
from pathlib import Path
sys.path.insert(0,sys.argv[1])
import weft
mode=sys.argv[2]
root=Path(__file__).resolve().parents[2]
checks=[]
def ok(name,condition):
 assert condition,name
 checks.append(name)
def refuses(name,call):
 try: call()
 except (TypeError,ValueError,AttributeError): checks.append(name);return
 raise AssertionError(name)
def digest(raw):return hashlib.sha256(raw.encode()).hexdigest()
if mode=='default':
 ok('default class absent',not hasattr(weft,'CurrentPresencePreparation'))
elif mode=='old-presence':
 ok('old presence class present',hasattr(weft,'CurrentPresencePreparation'))
 ok('graph method absent',not hasattr(weft.CurrentPresencePreparation,'graph_requirements_json'))
elif mode=='old-model':
 ok('old model method present',hasattr(weft.CurrentPresencePreparation,'model_requirements_json'))
 ok('graph method absent',not hasattr(weft.CurrentPresencePreparation,'graph_requirements_json'))
else:
 raw=(root/'tests/current-query-presence/fixtures/unadmitted-context.request.json').read_text()
 parent=weft.prepare_application_lineage(raw);child=parent.prepare_current_presence()
 other=weft.prepare_application_lineage(raw);other_child=other.prepare_current_presence()
 encoded=child.graph_requirements_json();report=json.loads(encoded)
 ok('same original parent',child.source_preparation() is parent)
 ok('equal source remains different parent',other is not parent and other_child.source_preparation() is other)
 ok('deterministic data does not replace custody',other_child.graph_requirements_json()==encoded and child.source_preparation() is not other)
 ok('exact original source',report['sourceRequestJson']==raw)
 ok('exact source hash',report['sourceRequestSha256']==digest(raw))
 ok('actual preparation hash',report['preparationSha256']==digest(child.json()))
 ok('same-context model hash',report['modelRequirementsSha256']==digest(child.model_requirements_json()))
 ok('version',report['version']=='weft.current-presence.graph-requirements/0.1-candidate')
 for flag in ['complete','authority','securityReady']:ok(flag+' false',report[flag] is False)
 ok('native admission required',report['nativeAdmission']=='required')
 expected=[('Ownership',['resource','project'],['Resource','Project']),('Assignment',['staff','project'],['Staff','Project'])]
 actual=[(a['owner']['element'],[e['role'] for e in a['endpoints']],[e['target']['element'] for e in a['endpoints']]) for a in report['associations']]
 ok('independent association mapping',actual==expected)
 for a in report['associations']:
  ok('record namespace',a['reference']['elementId']==a['owner']['element'] and 'relationshipId' not in a['reference'])
  for e in a['endpoints']:
   ok('target Key original owner',e['key']['record']==e['target'])
   ok('member original descriptor',e['members'][0]['field']==e['members'][0]['descriptors'][0]['identity'])
 del parent;gc.collect();ok('parent retained',child.source_preparation() is not None)
 ok('data stable after parent drop',child.graph_requirements_json()==encoded)
 callbacks=[]
 class Trap:
  def __str__(self):callbacks.append('str');raise AssertionError('coercion')
  def __len__(self):callbacks.append('len');raise AssertionError('inspection')
 for value in [None,{},encoded,report,other,Trap()]:
  refuses('wrong receiver',lambda value=value:weft.CurrentPresencePreparation.graph_requirements_json(value))
 refuses('extra argument',lambda:child.graph_requirements_json(Trap()))
 ok('no coercion callbacks',callbacks==[])
 for name,call in [('no constructor',lambda:weft.CurrentPresencePreparation(encoded)),('no copy',lambda:copy.copy(child)),('no pickle',lambda:pickle.dumps(child)),('frozen parent',lambda:setattr(child,'original',other))]:refuses(name,call)
 ok('report not compiler request',json.loads(weft.compile_json(encoded)).get('status')!='compiled')
print(json.dumps({'mode':mode,'checks':checks,'passed':len(checks),'nativeAdmission':False}))
