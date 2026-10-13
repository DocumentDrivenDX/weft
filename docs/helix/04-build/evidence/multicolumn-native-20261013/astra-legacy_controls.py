from pathlib import Path
import copy,json,hashlib,subprocess
out=Path(__file__).parent;weft=Path('/private/tmp/weft-security-main-integration');binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike');native=Path('/private/tmp/truss-security-main-integration/docs/helix/04-build/evidence/security-predicate-native-20261013')
base={k:json.loads((native/(k+'.json')).read_bytes()) for k in ('request','assessment')}
corpus=json.loads((weft/'crates/weft-core/tests/security-evaluation-oracle.json').read_bytes());fixture=corpus['cases'][0]
def ref(n):return {'documentId':'domain','moduleId':'m','elementId':n}
p=copy.deepcopy(base);p['request']['sql']='SELECT r.salary AS amount FROM Resource r';p['request']['parameters']={}
p['resultProbe']={'target':ref('Resource'),'field':ref('salary'),'outputName':'amount','rows':[[{'outcomeId':'withheld','disposition':'withheld'}]],'factCut':copy.deepcopy(fixture['cut']),'scopedFacts':{'version':'weft.security.scoped-facts/0.1.0','rows':[{'s0':copy.deepcopy(fixture['request']['resources'][0])}]}}
(out/'baseline-input.json').write_text(json.dumps(p,indent=2)+'\n');results=[]
def invoke(name,p,positive=True,with_facts=True):
 raw=json.dumps(p,separators=(',',':')).encode();r=subprocess.run([str(binary)],input=raw,capture_output=True,timeout=30)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:2000],r.stdout[:1000])
 if positive:
  v=json.loads(r.stdout);assert v['securityReady'] is False and v['resultCellInspectionPassed'] is True
  assert v.get('conditionalFactSelectionPassed',False)==with_facts
 else:assert r.stdout==b'';v=json.loads(r.stderr)
 results.append({'case':name,'passed':True,'status':r.returncode,'diagnostic':v.get('code')});return v
invoke('originalOracleValidMembership',p)
# Preserve cell-only behavior when both optional members are absent.
q=copy.deepcopy(p);q['resultProbe'].pop('factCut');q['resultProbe'].pop('scopedFacts');invoke('legacyCellOnly',q,with_facts=False)
for key in ('factCut','scopedFacts'):
 q=copy.deepcopy(p);q['resultProbe'].pop(key);invoke('missingPaired'+key,q,False)
 q=copy.deepcopy(p);q['resultProbe'][key]=None;invoke('null'+key,q,False)
# Actual owner evaluation must reject well-shaped withheld cells when membership is false or unknown.
for name,mutate in [
 ('inactive',lambda x:x['resultProbe']['factCut']['facts'][1]['fields'][-1].update(value={'boolean':False})),
 ('unknownRequiredFact',lambda x:x['resultProbe']['factCut']['facts'][1]['fields'].pop()),
 ('noWitness',lambda x:x['resultProbe']['factCut'].update(facts=[])),
 ('untrusted',lambda x:x['resultProbe']['factCut'].update(trusted=False)),
 ('generationMismatch',lambda x:x['resultProbe']['factCut'].update(expectedGeneration='other')),
 ('policyMismatch',lambda x:x['resultProbe']['factCut'].update(policyRevision='other')),
 ('ontologyMismatch',lambda x:x['resultProbe']['factCut'].update(ontologyRevision='other')),
 ('zeroSubjects',lambda x:x['resultProbe']['factCut'].update(subjects=[])),
 ('ambiguousSubject',lambda x:x['resultProbe']['factCut']['subjects'].append(copy.deepcopy(x['resultProbe']['factCut']['subjects'][0]))),
 ('incompleteOwnership',lambda x:x['resultProbe']['factCut']['coverage'][3].update(complete=False)),
 ('duplicateWitnessIdentity',lambda x:x['resultProbe']['factCut']['facts'].append(copy.deepcopy(x['resultProbe']['factCut']['facts'][0]))),
 ('badResourceKey',lambda x:x['resultProbe']['scopedFacts']['rows'][0]['s0'].update(key=[{'string':'wrong'}])),
 ('wrongScanName',lambda x:x['resultProbe']['scopedFacts']['rows'][0].update(s1=x['resultProbe']['scopedFacts']['rows'][0].pop('s0'))),
 ('missingScan',lambda x:x['resultProbe']['scopedFacts'].update(rows=[{}])),
 ('extraScan',lambda x:x['resultProbe']['scopedFacts']['rows'][0].update(s1=copy.deepcopy(x['resultProbe']['scopedFacts']['rows'][0]['s0']))),
 ('wrongScopedType',lambda x:x['resultProbe']['scopedFacts']['rows'][0]['s0'].update(type=ref('Staff'))),
 ('scopedCountMismatch',lambda x:x['resultProbe']['scopedFacts'].update(rows=[])),
 ('badScopedVersion',lambda x:x['resultProbe']['scopedFacts'].update(version='weft.security.scoped-facts/9')),
 ('extraScopedProperty',lambda x:x['resultProbe']['scopedFacts'].update(extra=True)),
 ('extraCutProperty',lambda x:x['resultProbe']['factCut'].update(extra=True)),
 ('extraProbeProperty',lambda x:x['resultProbe'].update(extra=True)),
 ('factBudget',lambda x:x['resultProbe']['factCut'].update(maxFacts=3)),
 ('stepBudget',lambda x:x['resultProbe']['factCut'].update(maxSteps=1)),
 ]:
 q=copy.deepcopy(p);mutate(q);invoke(name,q,False)
# Same identity may appear in inventory and rows only with consistent known projections.
q=copy.deepcopy(p);q['resultProbe']['factCut']['facts'].append(copy.deepcopy(q['resultProbe']['scopedFacts']['rows'][0]['s0']));invoke('consistentInventoryResource',q)
q['resultProbe']['factCut']['facts'][-1]['fields'][-1]['value']={'integerToken':'999'};invoke('conflictingInventoryResource',q,False)
# Original cells must match the actual scoped field, not merely its domain.
original=copy.deepcopy(p);original['request']['sql']='SELECT r.resourceId AS amount FROM Resource r';original['resultProbe']['field']=ref('resourceId');original['resultProbe']['rows']=[[{'outcomeId':'original','disposition':'original','value':{'string':'r1'}}]]
invoke('scopedOriginalValue',original)
q=copy.deepcopy(original);q['resultProbe']['rows'][0][0]['value']={'string':'another-valid-string'};invoke('originalValueForgery',q,False)
q=copy.deepcopy(original);q['resultProbe']['scopedFacts']['rows'][0]['s0']['fields']=[f for f in q['resultProbe']['scopedFacts']['rows'][0]['s0']['fields'] if f['field']!=ref('resourceId')];invoke('missingOriginalValue',q,False)
q=copy.deepcopy(original);q['resultProbe']['factCut']['coverage'][2]['fields']=[ref('salary')];invoke('missingOriginalCoverage',q,False)
# Projection consistency is not an enrichment mechanism.
q=copy.deepcopy(original);q['resultProbe']['factCut']['facts'].append(copy.deepcopy(q['resultProbe']['scopedFacts']['rows'][0]['s0']));q['resultProbe']['scopedFacts']['rows'][0]['s0']['fields']=[f for f in q['resultProbe']['scopedFacts']['rows'][0]['s0']['fields'] if f['field']!=ref('resourceId')];invoke('noInventoryEnrichment',q,False)
# Empty bags still require coverage of all actual rule dependencies.
q=copy.deepcopy(p);q['resultProbe']['rows']=[];q['resultProbe']['scopedFacts']['rows']=[];invoke('emptyRowsConditional',q)
q['resultProbe']['factCut']['coverage'][3]['complete']=False;invoke('emptyRowsIncompleteDependency',q,False)
# Joint cut/scoped JSON bound is charged before parsing in the owner checker.
q=copy.deepcopy(p);q['resultProbe']['factCut']['generation']='g'*4000000;q['resultProbe']['factCut']['expectedGeneration']='g'*4000000;invoke('combinedFactPayloadOverflow',q,False)
# Closed source binding remains enforced for the facts path.
q=copy.deepcopy(p);q['request']['policyJson']+=' ';invoke('sourceHashDrift',q,False)
q=copy.deepcopy(p);q['request']['callerTruths']={};invoke('callerTruthOverride',q,False)
result={'scope':'Actual original diagnostic/fact inspector with original admitted source plus caller-declared oracle facts. No native execution, facts authentication, native SQL lineage, complete result enumeration or release proof.','passedControls':len(results),'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'path':str(out/'results.json')}))
