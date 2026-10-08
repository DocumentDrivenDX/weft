"""Independent host phase/refusal assertions; @covers US-004-AC4."""
import copy,json
from pathlib import Path
from host_obligation_fixture import execute,Refused
ROOT=Path(__file__).resolve().parents[2]
artifact=json.loads((ROOT/'docs/helix/04-build/evidence/B-006-compound-native-initial/compile-artifacts.jsonl').read_text().splitlines()[0])['response']
pub=next(o['parameters'] for o in artifact['obligations'] if o['id']=='ashlar.candidate.publication')
proof=dict(effectiveCaller='fixture-caller',authenticatedCaller='fixture-caller',policyRevision='policy-1',completeVisibility=True,publication=pub['publication'],modelPins=artifact['modelPins'],bindingSha256=artifact['bindingSha256'],layoutRevision=pub['layoutRevision'],layoutSha256=pub['layoutSha256'],targetContext=artifact['targetContext'],manifestVerified=True,schemasVerified=True,retainedDataFiles=True,sourceIdentityVerified=True,endpointIntegrityVerified=True,projectionCoverageVerified=True,payloadValidated=True,mappingVerified=True,lifecycleVerified=True)
checks=[x for o in artifact['obligations'] if 'checks' in o['parameters'] for x in o['parameters']['checks']]
def result(rows):return dict(rows=rows,types=['STRING'],complete=True,truncated=False)
cases=[]
def probe(name,change=None,phase='before',artifact_change=None,transport=None,expect=False):
 a=copy.deepcopy(artifact)
 if artifact_change:artifact_change(a)
 view=copy.deepcopy(proof);calls=[];snapshots=[]
 def snapshot():
  snapshots.append(True);v=copy.deepcopy(view)
  if change and (phase=='before' or len(snapshots)>1):change(v)
  return v
 def query(sql,params):
  assert params==a['parameters'];calls.append(sql)
  r=result([['0']]) if sql!=a['sql'] else result([['{"state":"value","value":["1"]}']])
  if transport:transport(r,sql==a['sql'])
  return r
 try:
  rows=execute(a,snapshot,query,dict(caller="fixture-caller",policyRevision="policy-1"))
  assert expect,name;assert rows==[['{"state":"value","value":["1"]}']]
  assert calls==[x['sql'] for x in checks]+[a['sql']] and len(snapshots)==2
 except Refused:
  assert not expect,name
  if phase=='before' and change:assert calls==[],name
 cases.append(name)
probe('success',expect=True)
for key in ['completeVisibility','manifestVerified','schemasVerified','retainedDataFiles','sourceIdentityVerified','endpointIntegrityVerified','projectionCoverageVerified','payloadValidated','mappingVerified','lifecycleVerified']:
 for phase in ['before','after']:probe(key+'-'+phase,lambda v,k=key:v.update({k:False}),phase)
for key in ['effectiveCaller','authenticatedCaller','policyRevision','bindingSha256','layoutRevision','layoutSha256']:
 for phase in ['before','after']:probe(key+'-'+phase,lambda v,k=key:v.update({k:'changed'}),phase)
for key in ['publication','modelPins','targetContext']:
 for phase in ['before','after']:probe(key+'-'+phase,lambda v,k=key:v.update({k:{}}),phase)
probe('unknown-obligation',artifact_change=lambda a:a['obligations'].append(dict(id='unknown',owner='host',parameters={},failureCode='WFT-OBLIGATION')))
probe('unknown-publication-meaning',artifact_change=lambda a:next(o for o in a['obligations'] if o['id']=='ashlar.candidate.publication')['parameters'].update(unknown=True))
probe('unknown-guard-meaning',artifact_change=lambda a:next(o for o in a['obligations'] if 'checks' in o['parameters'])['parameters'].update(unknown=True))
probe('guard-violation',transport=lambda r,is_data:r.update(rows=[['1']]) if not is_data else None)
probe('numeric-zero-is-not-exact-text',transport=lambda r,is_data:r.update(rows=[[0]]) if not is_data else None)
probe('partial-transport',transport=lambda r,is_data:r.update(complete=False))
probe('truncated-transport',transport=lambda r,is_data:r.update(truncated=True))
probe('native-type-change',transport=lambda r,is_data:r.update(types=['DOUBLE']))
probe('native-null-value',transport=lambda r,is_data:r.update(rows=[[None]]) if is_data else None)
probe('duplicate-json-member',transport=lambda r,is_data:r.update(rows=[['{"state":"absent","state":"value","value":[]}']]) if is_data else None)
print(json.dumps(dict(state='passed',cases=len(cases),names=cases,qualification='Host fixture phase/control evidence; synthetic callback trust, native execution tested separately.')))
