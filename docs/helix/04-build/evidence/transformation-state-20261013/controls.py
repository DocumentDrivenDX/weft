from pathlib import Path
import json,copy,subprocess,hashlib,itertools,sqlite3
out=Path(__file__).parent;root=Path('/private/tmp/weft-security-main-integration');binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike')
truss=Path('/private/tmp/truss-security-main-integration/docs/helix/04-build/evidence/security-predicate-native-20261013')
base={'request':json.loads((truss/'request.json').read_bytes()),'assessment':json.loads((truss/'assessment.json').read_bytes())}
def ref(name):return {'documentId':'domain','moduleId':'m','elementId':name}
def transformed(field,value):return {'kind':'transformed','transform':'constant','version':'0.1.0','field':ref(field),'value':value}
def rule(i,slot=0,actions=['read']):
 r={'id':'r'+str(i),'effect':'permit','actions':actions,'target':[ref('Resource')],'condition':{'op':'literal','value':True}}
 if slot:r['disclosure']=[{'field':ref('salary'),'disposition':({'kind':{1:'original',2:'withheld'}[slot]} if slot<3 else transformed('resourceId',{'string':'identity'+str(slot)}))}]
 return r
def packet(rules,protected=True,extra=[]):
 p=copy.deepcopy(base);pol=json.loads(p['request']['policyJson']);pol['rules']=rules;p['request']['policyJson']=json.dumps(pol,separators=(',',':'))
 ont=json.loads(p['request']['ontologyJson']);ont['actions']+=[a for a in extra if a not in ont['actions']]
 f=next(f for e in ont['entities'] if e['type']==ref('Resource') for f in e['fields'] if f['ref']==ref('salary'));f['protection']='protected' if protected else 'unprotected';f.pop('queryUse',None)
 p['request']['ontologyJson']=json.dumps(ont,separators=(',',':'));return p
results=[];vectors=0

def invoke(name,p,positive=True):
 raw=json.dumps(p,separators=(',',':')).encode();r=subprocess.run([str(binary)],input=raw,capture_output=True,timeout=30)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:1500])
 if positive:
  v=json.loads(r.stdout);assert v['version']=='weft.truss.predicate-spike/0.4';assert v['securityReady'] is False
  assert v['missingEligibilityGroup']=='unsupported_refuse';assert v['indeterminateField']==v['conflictingField']=='projection_must_refuse'
 else:
  assert r.stdout==b'';v=json.loads(r.stderr);assert v['code']=='WFT-SECURITY-PREDICATE-SPIKE'
 results.append({'case':name,'passed':True,'inputBytes':len(raw),'exit':r.returncode});return v
con=sqlite3.connect(':memory:')
def field(v,action='read'):return next(d for g in v['eligibility'] if g['target']==ref('Resource') and g['action']==action for d in g['disclosure'] if d['field']==ref('salary'))
def state_choice(v,values):
 f=field(v);cols=[r['truthColumn'] for r in v['rules']];args=[values[r['id']] for r in v['rules']]
 # This cast spelling is the only PostgreSQL-specific syntax in the generated CASE.
 exprs=[f[k].replace('NULL::pg_catalog.int4','NULL') for k in ('stateExpression','transformationChoiceExpression')]
 return con.execute('SELECT '+','.join(exprs)+' FROM (SELECT '+','.join('? AS '+n for n in cols)+')',args).fetchone()
corpus=json.loads((root/'crates/weft-core/tests/security-disposition-oracle.json').read_bytes())['cases'];assert len(corpus)==432
for n,c in enumerate(corpus):
 for reverse in (False,True):
  rules=[rule(i,s) for i,s in enumerate(c['slots'])]
  if reverse:rules.reverse()
  v=invoke('oracle'+str(n)+('Reversed' if reverse else ''),packet(rules,c['protected']))
  expected=c['decision'] if c['decision'] in ('indeterminate','conflict') else {1:'original',2:'withheld',3:'transformed',4:'transformed',5:'transformed'}[c['disposition']]
  state,choice=state_choice(v,{r['id']:True for r in rules});assert state==expected,(n,reverse,state,expected)
  if state=='transformed':
   assert choice==0
   d=field(v)['transformations'][choice]['disposition']['transformed'];assert d['literal']['string']=='identity'+str(c['disposition'])
  else:assert choice is None
  vectors+=1
# Original typed numeric semantics and choice ordering; the renderer itself calls compose.
for left,right,equal in [('0','-0',True),('100','1e2',True),('1','2',False)]:
 for reverse in (False,True):
  rules=[rule(0,3),rule(1,3),rule(2,2)]
  for r,t in zip(rules,(left,right)):r['disclosure'][0]['disposition']=transformed('salary',{'integerToken':t})
  if reverse:rules.reverse()
  v=invoke('numeric'+left+'_'+right+str(reverse),packet(rules))
  transformed_ids=[r['id'] for r in rules if r['id']!='r2']
  for vals in itertools.product((False,None,True),repeat=3):
   a,b,w=vals;state,choice=state_choice(v,dict(zip(('r0','r1','r2'),vals)))
   expected='not-applicable' if None in vals or not any(vals) else 'withheld' if w else 'conflict' if a and b and not equal else 'transformed'
   assert state==expected
   expected_choice=next(i for i,rid in enumerate(transformed_ids) if dict(zip(('r0','r1','r2'),vals))[rid]) if state=='transformed' else None
   assert choice==expected_choice;vectors+=1
# Pair work charged: per group 10 incidences+2 fields+20 rule visits+4 disclosure visits+2 metadata+1 pair=39.
# 209 groups*39+10 roots+31 extra NOT nodes=8192. Each original condition stays within depth16.
original=json.loads(base['request']['ontologyJson'])['actions'];actions=original+['a'+str(i) for i in range(256-len(original))]
for extra,positive in [(31,True),(32,False)]:
 rules=[rule(i,3 if i<2 else 0,actions[:209]) for i in range(10)]
 for i,count in [(0,15),(1,15),(2,extra-30)]:
  for _ in range(count):rules[i]['condition']={'op':'not','arg':rules[i]['condition']}
 invoke('pairWork8192' if positive else 'pairWork8193',packet(rules,extra=actions),positive)
# One transformation's serialized owner metadata plus49 bytes of target/action/field identity.
r=rule(0,3);r['disclosure'][0]['disposition']=transformed('resourceId',{'string':''})
v=invoke('metadataSizing',packet([r]));fixed=len(json.dumps(field(v)['transformations'][0]['disposition'],separators=(',',':')).encode())
for n,positive in [(1048576-49-fixed,True),(1048576-48-fixed,False)]:
 r=rule(0,3);r['disclosure'][0]['disposition']=transformed('resourceId',{'string':'x'*n})
 invoke('transformedMetadataExact' if positive else 'transformedMetadataOneOver',packet([r]),positive)
result={'scope':'Actual diagnostic renderer against full original disposition corpus; in-memory SQLite only for emitted CASE expressions with NULL::pg_catalog.int4 spelling translated to NULL. Not an independent PostgreSQL execution/refinement proof.','passedControls':len(results),'stateChoiceEvaluations':vectors,'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'stateChoiceEvaluations':vectors,'path':str(out/'results.json')}))
