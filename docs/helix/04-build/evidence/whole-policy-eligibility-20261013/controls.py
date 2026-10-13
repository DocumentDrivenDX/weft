from pathlib import Path
import json,copy,subprocess,hashlib,itertools,sqlite3
out=Path(__file__).parent
root=Path('/private/tmp/weft-security-main-integration');binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike')
truss=Path('/private/tmp/truss-security-main-integration/docs/helix/04-build/evidence/security-predicate-native-20261013')
base={'request':json.loads((truss/'request.json').read_bytes()),'assessment':json.loads((truss/'assessment.json').read_bytes())}
def ref(name):return {'documentId':'domain','moduleId':'m','elementId':name}
def rule(identity,effect,actions=['read'],targets=[ref('Resource')]):return {'id':identity,'effect':effect,'actions':actions,'target':targets,'condition':{'op':'literal','value':True}}
def packet(rules,extra_actions=[]):
 p=copy.deepcopy(base);pol=json.loads(p['request']['policyJson']);pol['rules']=rules;p['request']['policyJson']=json.dumps(pol,separators=(',',':'))
 ont=json.loads(p['request']['ontologyJson']);ont['actions']+= [a for a in extra_actions if a not in ont['actions']];p['request']['ontologyJson']=json.dumps(ont,separators=(',',':'));return p
results=[]
def invoke(name,p,positive=True):
 raw=json.dumps(p,separators=(',',':')).encode();r=subprocess.run([str(binary)],input=raw,capture_output=True,timeout=30)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:2000])
 if positive:
  v=json.loads(r.stdout);assert v['version']=='weft.truss.predicate-spike/0.2';assert v['securityReady'] is False
  assert v['eligibilityScope']=='selected_rule_groups_empty_disclosure_only_no_missing_group_authorization'
  assert v['missingEligibilityGroup']=='unsupported_refuse'
 else:
  assert r.stdout==b'';v=json.loads(r.stderr);assert v['code']=='WFT-SECURITY-PREDICATE-SPIKE'
 results.append({'case':name,'passed':True,'inputBytes':len(raw),'exit':r.returncode});return v
con=sqlite3.connect(':memory:')
def sql_decision(v,values,target='Resource',action='read'):
 group=next(g for g in v['eligibility'] if g['target']==ref(target) and g['action']==action)
 names=[r['truthColumn'] for r in v['rules']]
 args=[values[r['id']] for r in v['rules']]
 sql='SELECT '+group['decisionExpression']+' FROM (SELECT '+','.join('? AS '+n for n in names)+')'
 return con.execute(sql,args).fetchone()[0]
three=[rule('p','permit'),rule('r','require'),rule('f','forbid')]
v=invoke('threeEffectRender',packet(three));vr=invoke('reorderedThreeEffectRender',packet(list(reversed(three))))
corpus=json.loads((root/'crates/weft-core/tests/security-composition-oracle.json').read_bytes())['decisions']
truth={'T':True,'F':False,'U':None}
for row in corpus:
 values={'p':truth[row['permit']],'r':truth[row['require']],'f':truth[row['forbid']]}
 assert sql_decision(v,values)==row['decision'];assert sql_decision(vr,values)==row['decision']
results.append({'case':'original27TruthVectorsBothRuleOrders','passed':True,'vectors':len(corpus)*2,'engine':'in-memory SQLite for emitted CASE/IS TRUE/IS FALSE/IS NULL only; not a PostgreSQL native oracle'})
four=[rule('p','permit'),rule('p2','permit'),rule('r','require'),rule('f','forbid')]
v=invoke('fourEffectRender',packet(four))
for vector in itertools.product((True,False,None),repeat=4):
 p,p2,r,f=vector;expected='indeterminate' if None in vector else 'permit' if (p or p2) and r and not f else 'deny'
 assert sql_decision(v,dict(zip(('p','p2','r','f'),vector)))==expected
results.append({'case':'multiplePermitOR81Vectors','passed':True,'vectors':81})
for effect in ('require','forbid'):
 v=invoke('noPermit_'+effect,packet([rule('r',effect)]))
 assert sql_decision(v,{'r':True})=='deny';assert sql_decision(v,{'r':False})=='deny';assert sql_decision(v,{'r':None})=='indeterminate'
v=invoke('noRulesScopedUnsupported',packet([]));assert v['eligibility']==[]
v=invoke('targetActionIsolation',packet([rule('p','permit',['read'],[ref('Resource'),ref('Staff')]),rule('f','forbid',['query-original'])]))
assert len(v['eligibility'])==3
assert all(g['target'] in [ref('Resource'),ref('Staff')] for g in v['eligibility'])
original_actions=json.loads(base['request']['ontologyJson'])['actions']
actions=original_actions+['action'+str(i) for i in range(256-len(original_actions))]
v=invoke('exact8192Work',packet([rule('r'+str(i),'permit',actions[:255]) for i in range(32)],actions))
assert len(v['eligibility'])==255
rules=[rule('r'+str(i),'permit',actions[:255]) for i in range(32)];rules[0]['actions']=actions
invoke('oneOver8192Work',packet(rules,actions),False)
# Target UTF-8 payload is domain(6)+m(1)+Resource(8)=15 bytes.
for n,positive in [(1048576-15,True),(1048576-14,False)]:
 a='x'*n;invoke('metadata'+('Exact' if positive else 'OneOver'),packet([rule('r','permit',[a])],[a]),positive)
result={'scope':'Actual built diagnostic renderer. CASE truth vectors executed in memory only, not PostgreSQL, and compared with original retained composition oracle or explicit empty-output fold. No native DB mutation.','binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'passedControls':len(results),'truthVectorEvaluations':141,'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'truthVectors':141,'result':str(out/'results.json')}))
