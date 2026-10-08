"""Independent host orchestration checks; no database or production-host claim.
@covers US-003-AC4: pins/visibility/authority and preparation gate publication.
"""
import copy, hashlib, json
from pathlib import Path
from host_obligation_fixture import Refused, execute
ROOT=Path(__file__).resolve().parents[2]
transports=json.loads((ROOT/'tests/truss-postgresql/fixtures/original-entity-public-transport.json').read_text())
results=[]
for transport in transports:
 artifact=transport['response']
 admitted=dict(bindingSha256=artifact['bindingSha256'],modelPins=artifact['modelPins'],targetContext=artifact['targetContext'],completeVisibility=True,authority='admitted-authority')
 for case in ['valid','unknown-obligation','unknown-context-semantics','missing-context','missing-complete-context','guard-parameter-mismatch','changed-binding','changed-model','changed-layout','hidden-state','guard-violation','guard-null','guard-error','query-error','authority-revoked','binding-revoked','model-revoked','layout-revoked','visibility-revoked']:
  response=copy.deepcopy(artifact);snapshots=[copy.deepcopy(admitted),copy.deepcopy(admitted)];events=[]
  if case=='unknown-obligation':response['obligations'].append(dict(id='unknown.future-obligation',owner='host',parameters={}))
  if case in ['changed-binding','binding-revoked']:snapshots[0 if case=='changed-binding' else 1]['bindingSha256']='0'*64
  if case in ['changed-model','model-revoked']:snapshots[0 if case=='changed-model' else 1]['modelPins'][0]['revision']='other'
  if case in ['changed-layout','layout-revoked']:snapshots[0 if case=='changed-layout' else 1]['targetContext']['storageLayoutRevision']='other'
  if case=='unknown-context-semantics':response['obligations'][0]['parameters']['execution']['unknownObligationMeaning']='ignore'
  if case=='missing-context':response['obligations']=[o for o in response['obligations'] if o['id']!='truss.candidate.context']
  if case=='missing-complete-context':response['obligations']=[o for o in response['obligations'] if o['id']!='truss.original.complete-read-context']
  if case=='guard-parameter-mismatch':next(o for o in response['obligations'] if 'beforeQuery' in o['parameters'])['parameters']['parameters']=[]
  if case=='hidden-state':snapshots[0]['completeVisibility']=False
  if case=='visibility-revoked':snapshots[1]['completeVisibility']=False
  if case=='authority-revoked':snapshots[1]['authority']='other'
  def snapshot():
   events.append('context');return snapshots.pop(0)
  def query(sql,parameters):
   guard=sql!=response['sql'];events.append('guard' if guard else 'query')
   assert parameters==response['parameters']
   if guard:
    if case=='guard-error':raise RuntimeError('independent driver refusal')
    return [('1' if case=='guard-violation' else None if case=='guard-null' else '0',)]
   if case=='query-error':raise RuntimeError('independent data failure')
   return [('18446744073709551615','9007199254740993','0.0100')]
  try:
   rows=execute(response,snapshot,query)
   assert case=='valid',(case,events)
   assert rows==[('18446744073709551615','9007199254740993','0.0100')]
   assert events[0]==events[-1]=='context'
   assert events[-2]=='query' and all(e=='guard' for e in events[1:-2])
   published=True
  except Refused:
   assert case!='valid',(case,events)
   published=False
   if case in ['unknown-obligation','unknown-context-semantics','missing-context','missing-complete-context','guard-parameter-mismatch','changed-binding','changed-model','changed-layout','hidden-state']:assert 'query' not in events and 'guard' not in events
   if case.startswith('guard-'):assert 'query' not in events
  results.append(dict(fixture=transport['fixture'],noteHome=transport['noteHome'],case=case,events=events,published=published))
receipt=dict(scope='Test-only host orchestration using independent injected driver/context callbacks; no native driver, storage or production host qualification',cases=len(results),sourceHashes={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest() for p in [Path(__file__),Path(__file__).with_name('host_obligation_fixture.py')]},results=results)
(ROOT/'docs/helix/04-build/evidence/B-005-host-obligation-orchestration.json').write_text(json.dumps(receipt,indent=2)+'\n')
print(f'{len(results)} host orchestration cases passed')
