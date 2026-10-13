from pathlib import Path
import json,copy,subprocess,hashlib,itertools,sqlite3
out=Path(__file__).parent;root=Path('/private/tmp/weft-security-main-integration');binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike')
truss=Path('/private/tmp/truss-security-main-integration/docs/helix/04-build/evidence/security-predicate-native-20261013')
base={'request':json.loads((truss/'request.json').read_bytes()),'assessment':json.loads((truss/'assessment.json').read_bytes())}
def ref(name):return {'documentId':'domain','moduleId':'m','elementId':name}
def rule(i,slot=0,target='Resource',actions=['read']):
 r={'id':'r'+str(i),'effect':'permit','actions':actions,'target':[ref(target)],'condition':{'op':'literal','value':True}}
 if slot:r['disclosure']=[{'field':ref('salary'),'disposition':{'kind':{1:'original',2:'withheld'}[slot]}}]
 return r
def packet(rules,protected=True,extra=[]):
 p=copy.deepcopy(base);pol=json.loads(p['request']['policyJson']);pol['rules']=rules;p['request']['policyJson']=json.dumps(pol,separators=(',',':'))
 ont=json.loads(p['request']['ontologyJson']);ont['actions']+=[a for a in extra if a not in ont['actions']]
 f=next(f for e in ont['entities'] if e['type']==ref('Resource') for f in e['fields'] if f['ref']==ref('salary'))
 f['protection']='protected' if protected else 'unprotected';f.pop('queryUse',None)
 p['request']['ontologyJson']=json.dumps(ont,separators=(',',':'));return p
results=[];vectors=0

def invoke(name,p,positive=True):
 raw=json.dumps(p,separators=(',',':')).encode();r=subprocess.run([str(binary)],input=raw,capture_output=True,timeout=30)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:1500])
 if positive:
  v=json.loads(r.stdout);assert v['version']=='weft.truss.predicate-spike/0.3';assert v['securityReady'] is False
  assert v['missingEligibilityGroup']=='unsupported_refuse';assert v['indeterminateField']=='projection_must_refuse'
  assert v['disclosureScope']=='row_permit_gated_original_withheld_states_only_transformations_refuse_no_value_projection'
 else:
  assert r.stdout==b'';v=json.loads(r.stderr);assert v['code']=='WFT-SECURITY-PREDICATE-SPIKE'
 results.append({'case':name,'passed':True,'inputBytes':len(raw),'exit':r.returncode});return v
con=sqlite3.connect(':memory:')
def state(v,values,field='salary',target='Resource'):
 g=next(g for g in v['eligibility'] if g['target']==ref(target) and g['action']=='read')
 e=next(d['stateExpression'] for d in g['disclosure'] if d['field']==ref(field))
 cols=[r['truthColumn'] for r in v['rules']];args=[values[r['id']] for r in v['rules']]
 return con.execute('SELECT '+e+' FROM (SELECT '+','.join('? AS '+n for n in cols)+')',args).fetchone()[0]
corpus=json.loads((root/'crates/weft-core/tests/security-disposition-oracle.json').read_bytes())['cases']
selected=[c for c in corpus if all(s<=2 for s in c['slots'])];assert len(selected)==54
for n,c in enumerate(selected):
 rules=[rule(i,s) for i,s in enumerate(c['slots'])]
 v=invoke('originalDispositionVector'+str(n),packet(rules,c['protected']))
 expected='indeterminate' if c['decision']=='indeterminate' else {1:'original',2:'withheld'}[c['disposition']]
 assert state(v,{r['id']:True for r in rules})==expected;vectors+=1
 # Rule order has no semantic precedence.
 v=invoke('reorderedDispositionVector'+str(n),packet(list(reversed(rules)),c['protected']))
 assert state(v,{r['id']:True for r in rules})==expected;vectors+=1
for protected in (False,True):
 slots=[1,2,0];v=invoke('activePermitFold'+str(protected),packet([rule(i,s) for i,s in enumerate(slots)],protected))
 for vals in itertools.product((False,None,True),repeat=3):
  expected='not-applicable' if None in vals or not any(vals) else 'withheld' if vals[1] else 'original' if vals[0] or not protected else 'indeterminate'
  assert state(v,dict(zip(('r0','r1','r2'),vals)))==expected;vectors+=1
v=invoke('associationTargetFields',packet([rule(0,target='Ownership')]))
assert state(v,{'r0':True},'ownerId','Ownership')=='original'
assert state(v,{'r0':False},'ownerId','Ownership')=='not-applicable';vectors+=2
for active in (False,True):
 r=rule(0);r['condition']['value']=active;r['disclosure']=[{'field':ref('salary'),'disposition':{'kind':'transformed','transform':'constant','version':'0.1.0','field':ref('resourceId'),'value':{'string':'x'}}}]
 invoke('transformRefusesEvenInactive'+str(active),packet([r]),False)
# Work = 10 literal roots + 255*(10 group entries+2 fields+20 selected-rule visits)=8170.
original=json.loads(base['request']['ontologyJson'])['actions'];actions=original+['a'+str(i) for i in range(256-len(original))]
for extra,positive in [(22,True),(23,False)]:
 rules=[rule(i,actions=actions[:255]) for i in range(10)]
 for i,count in [(0,11),(1,extra-11)]:
  for _ in range(count):rules[i]['condition']={'op':'not','arg':rules[i]['condition']}
 invoke('work8192' if positive else 'work8193',packet(rules,extra=actions),positive)
# Group target=15 bytes; Resource's two field refs=17+13 bytes.
for length,positive in [(1048576-45,True),(1048576-44,False)]:
 action='x'*length;invoke('metadataExact' if positive else 'metadataOneOver',packet([rule(0,actions=[action])],extra=[action]),positive)
result={'scope':'Actual diagnostic renderer with original admitted sources. In-memory SQLite executes only emitted CASE/state SQL; this is not an independent PostgreSQL oracle. No native DB mutation.','passedControls':len(results),'stateEvaluations':vectors,'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'stateEvaluations':vectors,'path':str(out/'results.json')}))
