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
  v=json.loads(r.stdout);assert v['version']=='weft.truss.predicate-spike/0.6';assert v['securityReady'] is False
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
# Interleaved classes: equality must reuse the minimum original class while preserving each rule and token.
from decimal import Decimal
cases=[('threeEquivalent',['0','-0','0e100']),('interleaved',['0','1','-0','1.0','0e9']),('allDifferent',['1','2','3']),('negativeEquivalent',['-100','-1e2','-100.0']),('exponentEquivalent',['100','1e2','100.00'])]
for name,tokens in cases:
 orders=[list(range(len(tokens))),list(reversed(range(len(tokens))))]
 if len(tokens)>3:orders.extend([[2,3,4,0,1],[1,0,3,2,4]])
 for order_index,order in enumerate(orders):
  rules=[rule(i,trans({'integerToken':tokens[i]})) for i in order]
  v=invoke(name+str(order_index),packet(rules));meta=field(v)['transformations'];values=[Decimal(tokens[i]) for i in order];classes=[values.index(x) for x in values]
  assert [m['outcomeId'] for m in meta]==['transform'+str(c) for c in classes]
  assert [m['ruleId'] for m in meta]==['r'+str(i) for i in order]
  assert [m['slot'] for m in meta]==list(range(len(tokens)))
  assert [m['disposition']['transformed']['literal'] for m in meta]==[{'integerToken':tokens[i]} for i in order]
  assert v['parameters']==[{'integerToken':tokens[i]} for i in order] and v['parameterTypes']==['jsonb']*len(tokens)
  for truths in itertools.product((False,None,True),repeat=len(tokens)):
   actual=cell(v,{'kind':'integer','text':'999'},dict(zip(['r'+str(i) for i in order],truths)))
   active=[i for i,t in enumerate(truths) if t]
   state='not-applicable' if None in truths or not active else 'conflict' if len({values[i] for i in active})>1 else 'transformed'
   assert actual['disposition']==state
   if state=='transformed':
    chosen=active[0];assert actual=={'outcomeId':'transform'+str(classes[chosen]),'disposition':'transformed','value':{'integerToken':tokens[order[chosen]]}}
   else:assert 'value' not in actual
# Same scalar/domain value with distinct output-field identity must remain separate.
p=packet([rule(0,trans({'string':'x'},'resourceId')),rule(1,trans({'string':'x'},'staffId'))])
v=invoke('sameValueDifferentOutputField',p);assert [m['outcomeId'] for m in field(v)['transformations']]==['transform0','transform1'];assert cell(v,None,{})=={'disposition':'conflict'}
# Original string and nullable-null equivalence also delegates to owner composition.
for name,values in [('string',[{'string':'a'},{'string':'b'},{'string':'a'}]),('null',[None,None])]:
 rules=[rule(i,trans(x)) for i,x in enumerate(values)];v=invoke(name+'Classes',packet(rules,family='string',nullable=name=='null'));expected=['transform0','transform1','transform0'] if name=='string' else ['transform0','transform0'];assert [m['outcomeId'] for m in field(v)['transformations']]==expected
 for i,x in enumerate(values):assert cell(v,None,{r['id']:j==i for j,r in enumerate(rules)})=={'outcomeId':expected[i],'disposition':'transformed','value':x}
# A withholding rule does not join a transformed class or erase any transformed provenance.
rules=[rule(0,trans({'integerToken':'0'})),rule(1,{'kind':'withheld'}),rule(2,trans({'integerToken':'-0'}))]
v=invoke('withheldProvenance',packet(rules));assert [m['ruleId'] for m in field(v)['transformations']]==['r0','r2'];assert [m['outcomeId'] for m in field(v)['transformations']]==['transform0','transform0'];assert cell(v,None,{})=={'outcomeId':'withheld','disposition':'withheld'}
# New retained ruleId bytes share metadata budget before clone, including UTF8 bytes.
for name,rid in [('ascii','r0'),('unicode','é'*64)]:
 r=rule(0,trans({'string':''},'resourceId'));r['id']=rid;v=invoke(name+'MetadataSizing',packet([r]));fixed=len(json.dumps(field(v)['transformations'][0]['disposition'],separators=(',',':'),ensure_ascii=False).encode());base_charge=49+len(rid.encode())+fixed
 for delta,positive in [(0,True),(1,False)]:
  r=rule(0,trans({'string':'x'*(1048576-base_charge+delta)},'resourceId'));r['id']=rid;invoke(name+'Metadata'+str(1048576+delta),packet([r]),positive)
result={'scope':'Actual admitted diagnostic renderer, original owner pair composer, selected class/provenance/parameter checks and emitted CASE/JSON-envelope branch controls in SQLite. Native validity supplied as Boolean premise; not PostgreSQL or ResultContract checker integration.','passedControls':len(results),'cellEvaluations':evaluations,'binary':str(binary),'binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps({'passedControls':len(results),'cellEvaluations':evaluations,'path':str(out/'results.json')}))
