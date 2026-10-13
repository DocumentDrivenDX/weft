from pathlib import Path
import json,copy,subprocess,hashlib,itertools,sqlite3,re,base64
out=Path(__file__).parent;root=Path('/private/tmp/weft-security-main-integration');binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike')
truss=Path('/private/tmp/truss-security-main-integration/docs/helix/04-build/evidence/security-predicate-native-20261013')
base={k:json.loads((truss/(k+'.json')).read_bytes()) for k in ('request','assessment')}
def ref(n):return {'documentId':'domain','moduleId':'m','elementId':n}
def rule(i,disposition=None,condition=None):
 r={'id':'r'+str(i),'effect':'permit','actions':['read'],'target':[ref('Resource')],'condition':condition or {'op':'literal','value':True}}
 if disposition is not None:r['disclosure']=[{'field':ref('salary'),'disposition':disposition}]
 return r
def trans(value,field='salary'):return {'kind':'transformed','transform':'constant','version':'0.1.0','field':ref(field),'value':value}
def packet(rules,family='integer',nullable=False,protected=True):
 p=copy.deepcopy(base);pol=json.loads(p['request']['policyJson']);pol['rules']=rules;p['request']['policyJson']=json.dumps(pol,separators=(',',':'))
 ont=json.loads(p['request']['ontologyJson']);f=next(f for e in ont['entities'] if e['type']==ref('Resource') for f in e['fields'] if f['ref']==ref('salary'));f['protection']='protected' if protected else 'unprotected';f.pop('queryUse',None);p['request']['ontologyJson']=json.dumps(ont,separators=(',',':'))
 m=json.loads(p['request']['modules'][0]['documentJson']);f=next(e for e in m['modules'][0]['elements'] if e['id']=='salary');f['scalarType']=family;f['nullability']='absent-allowed' if nullable else 'required';f.pop('facets',None)
 if family=='decimal':f['facets']={'precision':20,'scale':4}
 s=json.dumps(m,separators=(',',':'));p['request']['modules'][0]['documentJson']=s;p['request']['modules'][0]['pin']['sha256']=hashlib.sha256(s.encode()).hexdigest();p['assessment']['modelPins']=[p['request']['modules'][0]['pin']]
 for prop in p['assessment']['inventory']['properties']:
  if prop['element']=='salary':prop['scalar_type']=family;prop['facets']=f.get('facets',{});prop['nullability']=f['nullability']
 return p
results=[];evaluations=0

def invoke(name,p,positive=True):
 raw=json.dumps(p,separators=(',',':')).encode();r=subprocess.run([str(binary)],input=raw,capture_output=True,timeout=30)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:1000])
 if positive:
  v=json.loads(r.stdout);assert v['version']=='weft.truss.predicate-spike/0.5';assert v['securityReady'] is False
  assert v['projectionScope']=='diagnostic_cell_envelopes_not_registered_result_contract'
  assert v['refusedProjectionStates']==['conflict','indeterminate','refuse','not-applicable']
  assert len(v['parameterTypes'])==len(v['parameters'])
 else:
  assert r.stdout==b'';v=json.loads(r.stderr);assert v['code']=='WFT-SECURITY-PREDICATE-SPIKE'
 results.append({'case':name,'passed':True,'inputBytes':len(raw),'exit':r.returncode});return v
# Emitted CASE path/JSON-envelope evaluator only. It intentionally treats native validity
# as a supplied Boolean premise; this is not a native SQL or scalar-validation oracle.
def jb(v):return b'J'+json.dumps(v,separators=(',',':'),ensure_ascii=False).encode()
def un(v):return json.loads(v[1:]) if isinstance(v,bytes) and v[:1]==b'J' else v
con=sqlite3.connect(':memory:');con.create_function('jbuild',-1,lambda *x:jb({x[i]:un(x[i+1]) for i in range(0,len(x),2)}));con.create_function('jget',2,lambda v,k:None if v is None or not isinstance(un(v),dict) or k not in un(v) else jb(un(v)[k]));con.create_function('jtext',2,lambda v,k:None if v is None else un(v).get(k));con.create_function('jb',1,jb);con.create_function('hexbase',1,lambda v:base64.b64decode(v,validate=True).hex());con.create_function('valid',3,lambda *_:True)
def field(v):return next(d for g in v['eligibility'] if g['action']=='read' for d in g['disclosure'] if d['field']==ref('salary'))
prop=str(next(p['prop_id'] for p in base['assessment']['inventory']['properties'] if p['element']=='salary'))
def cell(v,raw,truths,valid=True,missing=False):
 global evaluations
 con.create_function('valid',3,lambda *_:valid)
 sql=field(v)['cellExpression'];sql=re.sub(r"r\.props->'([0-9]+)'->>'([^']+)'",r"jtext(jget(props,'\1'),'\2')",sql);sql=re.sub(r"r\.props->'([0-9]+)'->'([^']+)'",r"jget(jget(props,'\1'),'\2')",sql);sql=re.sub(r"r\.props->'([0-9]+)'",r"jget(props,'\1')",sql)
 sql=sql.replace("pg_catalog.to_jsonb(pg_catalog.encode(pg_catalog.decode(","jb(hexbase(").replace(",'base64'),'hex'))","))")
 sql=sql.replace('pg_catalog.jsonb_build_object','jbuild').replace('truss.valid_scalar_cell','valid').replace('NULL::pg_catalog.int4','NULL').replace("'null'::pg_catalog.jsonb","jb(NULL)")
 args={'props':jb({} if missing else {prop:raw})}
 for i,(typ,value) in enumerate(zip(v['parameterTypes'],v['parameters']),1):
  if typ=='jsonb':sql=sql.replace('$'+str(i)+'::pg_catalog.jsonb',':p'+str(i));args['p'+str(i)]=jb(value)
 cols=[]
 for r in v['rules']:
  name=r['truthColumn'];args[name]=truths.get(r['id'],True);cols.append(':'+name+' AS '+name)
 actual=un(con.execute('SELECT ('+sql+') FROM (SELECT :props AS props,'+','.join(cols)+')',args).fetchone()[0]);evaluations+=1;return actual
for family,value,expected in [('string','quoted\"\\é\n',{'string':'quoted\"\\é\n'}),('boolean',False,{'boolean':False}),('boolean',True,{'boolean':True}),('integer','-0',{'integerToken':'-0'}),('decimal','1.2300e+1',{'decimalToken':'1.2300e+1'}),('binary','AQI=',{'binaryHex':'0102'}),('binary','',{'binaryHex':''})]:
 v=invoke('original_'+family+str(value),packet([rule(0,{'kind':'original'})],family));raw={'kind':family,('value' if family=='boolean' else 'text'):value}
 assert cell(v,raw,{})=={'outcomeId':'original','disposition':'original','value':expected}
 for valid in (False,None):assert cell(v,raw,{},valid=valid)=={'disposition':'refuse'}
 assert cell(v,raw,{},missing=True)=={'outcomeId':'refuse','disposition':'refuse'}
 for truth in (False,None):assert cell(v,raw,{'r0':truth})=={'disposition':'not-applicable'}
# Required and optional absence/null are separately represented, conditional on validator result.
for nullable in (False,True):
 v=invoke('nullability'+str(nullable),packet([rule(0,{'kind':'original'})],nullable=nullable))
 expected={'outcomeId':'absent' if nullable else 'refuse','disposition':'absent' if nullable else 'refuse'}
 assert cell(v,None,{},missing=True)==expected
 assert cell(v,{'kind':'null'},{},valid=nullable)==({'outcomeId':'original','disposition':'original','value':None} if nullable else {'disposition':'refuse'})
 assert cell(v,None,{},valid=False)=={'disposition':'refuse'}
# Row-gated precedence/transform literal custody across every trivalent truth assignment.
for left,right,equal in [('0','-0',True),('100','1e2',True),('1','2',False)]:
 for reverse in (False,True):
  rules=[rule(0,trans({'integerToken':left})),rule(1,trans({'integerToken':right})),rule(2,{'kind':'withheld'})]
  if reverse:rules.reverse()
  v=invoke('transform_'+left+'_'+right+str(reverse),packet(rules));assert v['parameterTypes']==['jsonb','jsonb']
  for a,b,w in itertools.product((False,None,True),repeat=3):
   actual=cell(v,{'kind':'integer','text':'999'},{'r0':a,'r1':b,'r2':w},valid=False)
   expected='not-applicable' if None in (a,b,w) or not any((a,b,w)) else 'withheld' if w else 'conflict' if a and b and not equal else 'transformed'
   assert actual['disposition']==expected,(left,right,a,b,w,actual)
   if expected=='transformed':
    active={'r0':a,'r1':b};chosen=next(r for r in rules if r['id']!='r2' and active[r['id']]);slot=[r['id'] for r in rules if r['id']!='r2'].index(chosen['id']);token=left if chosen['id']=='r0' else right
    assert actual=={'outcomeId':'transform'+str(slot),'disposition':'transformed','value':{'integerToken':token}}
   else:assert 'value' not in actual
# Explicit transformed null is a JSONB null parameter, never absence.
v=invoke('transformExplicitNull',packet([rule(0,trans(None))],nullable=True));assert v['parameters']==[None] and v['parameterTypes']==['jsonb'];assert cell(v,None,{})=={'outcomeId':'transform0','disposition':'transformed','value':None}
# Preserve original string as a bound scalar; transformation follows predicate ordinals.
secret="x'); SELECT unknown; --\\\n\"é"
def eq(value):return {'op':'eq','left':{'kind':'resource','field':ref('resourceId')},'right':{'kind':'constant','field':ref('resourceId'),'value':{'string':value}}}
v=invoke('mixedOrdinalTypes',packet([rule(0,trans({'string':secret},'resourceId'),eq(secret))]));assert v['parameters']==[secret,{'string':secret}] and v['parameterTypes']==['text','jsonb'];assert '$1::pg_catalog.text' in v['rules'][0]['expression'] and '$2::pg_catalog.jsonb' in field(v)['cellExpression'];assert secret not in field(v)['cellExpression']
# Shared1MiB budget, accounting for existing predicate plus original JSON literal bytes.
for delta,positive in [(0,True),(1,False)]:
 transformed={'string':'X'*100};cost=len(json.dumps(transformed,separators=(',',':')).encode());n=1048576-cost-2+delta
 invoke('sharedParameterBytes'+str(1048576+delta),packet([rule(0,trans(transformed,'resourceId'),eq('s'*n))]),positive)
# Predicate constants and transformation constants share1024 slots.
def many_equals(n):
 leaves=[eq('a') for _ in range(n)]
 while len(leaves)>1:leaves=[{'op':'and','args':leaves[i:i+64]} if len(leaves[i:i+64])>1 else leaves[i] for i in range(0,len(leaves),64)]
 return leaves[0]
for count,positive in [(1023,True),(1024,False)]:
 invoke('sharedParameterCount'+str(count+1),packet([rule(0,trans({'integerToken':'1'}),many_equals(count))]),positive)
result={'scope':'Actual admitted diagnostic renderer plus emitted CASE/JSON-envelope branch controls in SQLite; JSON operations translated explicitly, native validity supplied as a Boolean premise. Not native PostgreSQL execution, scalar-validator proof, fact admission or result-contract qualification.','passedControls':len(results),'cellEvaluations':evaluations,'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'cellEvaluations':evaluations,'path':str(out/'results.json')}))
