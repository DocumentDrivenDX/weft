"""Deterministic application fixtures; expected decisions are authored here, never compiled."""
import json,hashlib
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
OUT=Path(__file__).parent/'fixtures'
doc=json.loads((ROOT/'docs/helix/03-test/fixtures/sales.umf.json').read_text())
e=doc['modules'][0]['elements']; mid='sales'
ref=lambda id:dict(module=mid,element=id)
def field(id,name,scalar=None,availability='required',**extra):
    f=dict(id=id,name=name,kind='field',nullability=availability,cardinality='one',extensions={})
    f.update(extra)
    if scalar:f['scalarType']=scalar
    return f
for index,key,field_id in [(0,'customer-pk','customer-id'),(1,'order-pk','order-id')]:
    e[index]['keys']=[dict(id=key,name=key,fields=[ref(field_id)],primary=True)]
e[1]['members'].insert(0,ref('order-id'))
e.append(field('order-id','id','integer',facets={'integerWidth':{'bits':64,'signed':False}}))
e[0]['members'] += [ref('nickname'),ref('tags'),ref('address')]
e += [field('nickname','nickname','string','absent-allowed'),field('tags','tags',cardinality='array',itemType=ref('tag-item')),field('tag-item','item','string'),field('address','address',availability='absent-allowed',references=[dict(role='record-type',**ref('address-record'))]),dict(id='address-record',name='Address',kind='record',members=[ref('street'),ref('zip')],extensions={}),field('street','street','string'),field('zip','zip','string','absent-allowed')]
doc['modules'][0]['relationships']=[dict(id='customer-orders',name='orders',source=[ref('customer')],target=[dict(**ref('orders'),key='order-pk')],sourceMultiplicity={'min':0,'max':'*'},targetMultiplicity={'min':0,'max':'*'},targetLifecycle='independent',directed=True,inverse='customer')]
def text(d):return json.dumps(d,ensure_ascii=False,separators=(',',':'))
def module(d):
    raw=text(d)
    return dict(documentJson=raw,pin=dict(documentId=d['id'],revision='application-fixture-1',umfVersion='0.7.0',sha256=hashlib.sha256(raw.encode()).hexdigest()),selectedModuleIds=['sales'])
def request(sql,profile=None,parameters=None,d=None):
    r=dict(dialect='weft-sql/0.2.0',sql=sql,modules=[module(d or doc)])
    if profile:r['readProfile']=dict(version='weft-application-read/0.2.0',subset=profile)
    if parameters is not None:r['parameters']=parameters
    return r
cases=[]
def add(id,sql,profile=None,parameters=None,code=None,d=None):
    cases.append(dict(id=id,request=request(sql,profile,parameters,d),expected=dict(status='blocked' if code else 'resolved',**({'code':code} if code else {}))))
add('whole-entity','SELECT c.* FROM Customer c ORDER BY c.id LIMIT 10','entity-page')
add('scalar-page','SELECT c.id,c.nickname,c.tags,c.address FROM Customer c ORDER BY c.id LIMIT 10','entity-page')
add('single-cursor','SELECT c.* FROM Customer c WHERE c.id > :cursor ORDER BY c.id ASC LIMIT 2','entity-page',{'cursor':{'family':'integer','value':'9007199254740993'}})
add('injection-text','SELECT c.id FROM Customer c WHERE c.name = :name ORDER BY c.id LIMIT 20','entity-page',{'name':{'family':'string','value':"'; DROP TABLE Objects; --"}})
add('global-count','SELECT COUNT(*) AS total FROM Customer c','count-summary')
add('grouped-count','SELECT c.name,COUNT(*) AS total FROM Customer c GROUP BY c.name ORDER BY c.name LIMIT 1000','count-summary')
add('join-count','SELECT COUNT(*) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id','count-summary')
add('global-sum','SELECT SUM(o.total) FROM Orders o')
add('related-page','SELECT c.id,RELATED_KEYS(c.orders,2) AS orders FROM Customer c ORDER BY c.id LIMIT 20','related-entity-page')
add('related-filter','SELECT c.* FROM Customer c WHERE HAS_RELATED(c.orders,KEY(:order_id)) ORDER BY c.id LIMIT 5','related-entity-page',{'order_id':{'family':'integer','value':'1'}})
add('inverse-page','SELECT o.id,RELATED_KEYS(o.customer,1) AS customer FROM Orders o ORDER BY o.id LIMIT 5','related-entity-page')
add('case-folded-param','SELECT c.id FROM Customer c WHERE c.id = :ID ORDER BY c.id LIMIT 10','entity-page',{'Id':{'family':'integer','value':'1'}})
for id,sql,profile,code in [
('missing-order','SELECT c.* FROM Customer c LIMIT 10','entity-page','WFT-PROFILE'),
('missing-limit','SELECT c.* FROM Customer c ORDER BY c.id','entity-page','WFT-PROFILE'),
('wrong-key-order','SELECT c.* FROM Customer c ORDER BY c.name LIMIT 10','entity-page','WFT-PROFILE'),
('related-in-entity','SELECT c.id,RELATED_KEYS(c.orders,2) AS orders FROM Customer c ORDER BY c.id LIMIT 10','entity-page','WFT-PROFILE'),
('missing-related','SELECT c.id FROM Customer c ORDER BY c.id LIMIT 10','related-entity-page','WFT-PROFILE'),
('count-unbounded-groups','SELECT c.name,COUNT(*) FROM Customer c GROUP BY c.name','count-summary','WFT-PROFILE'),
('count-nongrouped-order','SELECT COUNT(*) FROM Customer c ORDER BY c.id LIMIT 10','count-summary','WFT-GROUPING'),
('count-ungrouped-field','SELECT c.id,COUNT(*) FROM Customer c','count-summary','WFT-GROUPING'),
('entity-with-count','SELECT c.*,COUNT(*) FROM Customer c',None,'WFT-GROUPING'),
('related-with-count','SELECT RELATED_KEYS(c.orders,1),COUNT(*) FROM Customer c',None,'WFT-GROUPING'),
('duplicate-expanded-name','SELECT c.*,c.id FROM Customer c',None,'WFT-OUTPUT-NAME'),
('duplicate-group','SELECT c.id,COUNT(*) FROM Customer c GROUP BY c.id,c.id',None,'WFT-GROUPING'),
('boolean-sum','SELECT SUM(c.active) FROM Customer c',None,'WFT-TYPE'),
('forward-alias','SELECT COUNT(*) FROM Customer c JOIN Orders o ON o.id=x.id JOIN Orders x ON x.id=c.id',None,'WFT-NAME-MISSING'),
('join-literal','SELECT COUNT(*) FROM Customer c JOIN Orders o ON o.id=1',None,'WFT-UNSUPPORTED'),
('quoted-limit',"SELECT c.id FROM Customer c LIMIT '10'",None,'WFT-LIMIT'),
('related-key-arity','SELECT c.id FROM Customer c WHERE HAS_RELATED(c.orders,KEY(1,2))',None,'WFT-TYPE'),
('independent-property-join','SELECT c.id,RELATED_KEYS(c.customer_id,1) FROM Customer c',None,'WFT-NAME-MISSING'),
]:add(id,sql,profile,code=code)
param_sql='SELECT c.id FROM Customer c WHERE c.id > :cursor ORDER BY c.id LIMIT 10'
for id,params,code in [
('missing-param',{},'WFT-PARAMETER'),('surplus-param',{'cursor':{'family':'integer','value':'1'},'extra':{'family':'integer','value':'2'}},'WFT-PARAMETER'),
('wrong-family',{'cursor':{'family':'string','value':'1'}},'WFT-PARAMETER'),
('exponent-param',{'cursor':{'family':'integer','value':'1e2'}},'WFT-PARAMETER'),
('plus-param',{'cursor':{'family':'integer','value':'+1'}},'WFT-PARAMETER'),
('duplicate-fold-param',{'cursor':{'family':'integer','value':'1'},'CURSOR':{'family':'integer','value':'1'}},'WFT-PARAMETER'),
('numeric-injection-param',{'cursor':{'family':'integer','value':'1; SELECT 2'}},'WFT-PARAMETER'),
('unsigned-negative-param',{'cursor':{'family':'integer','value':'-1'}},'WFT-NUMERIC-DOMAIN'),
('uint64-overflow-param',{'cursor':{'family':'integer','value':'18446744073709551616'}},'WFT-NUMERIC-DOMAIN'),
]:add(id,param_sql,'entity-page',params,code=code)
# Composite keys and lexicographic continuation: order of authored components matters.
composite=json.loads(text(doc));composite['modules'][0]['elements'][0]['keys'][0]['fields']=[ref('customer-name'),ref('customer-id')]
add('composite-cursor',"SELECT c.id,c.name FROM Customer c WHERE (c.name,c.id)>('Alice',:cursor) ORDER BY c.name,c.id LIMIT 10",'entity-page',{'cursor':{'family':'integer','value':'1'}},d=composite)
add('composite-conjunction-refused',"SELECT c.id FROM Customer c WHERE c.name>'Alice' AND c.id>1 ORDER BY c.name,c.id LIMIT 10",'entity-page',code='WFT-PROFILE',d=composite)
add('composite-wrong-order',"SELECT c.id FROM Customer c WHERE (c.id,c.name)>(1,'Alice') ORDER BY c.name,c.id LIMIT 10",'entity-page',code='WFT-PROFILE',d=composite)
add('unicode-key-order','SELECT c.name,c.id FROM Customer c ORDER BY c.name,c.id LIMIT 10','entity-page',d=composite)
add('unicode-key-cursor',"SELECT c.name,c.id FROM Customer c WHERE (c.name,c.id)>('trail',4) ORDER BY c.name,c.id LIMIT 10",'entity-page',d=composite)
add('exact-large-key-order','SELECT c.id FROM Customer c ORDER BY c.id LIMIT 10','entity-page')
# Reuse every numeric boundary as a named parameter; expected outcomes remain independently authored.
original=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
for c in original:
    if c.get('category')!='numeric':continue
    r=c['request'];sql=r['sql'];left,right=sql.rsplit('=',1)
    right=right.strip().rstrip(';').strip()
    model=json.loads(r['modules'][0]['documentJson']);field_name=left.rsplit('.',1)[1].strip()
    scalar=next(e['scalarType'] for m in model['modules'] for e in m['elements'] if e.get('name','').lower()==field_name.lower())
    params={'boundary':{'family':scalar,'value':right}}
    expected=c['expected'];add('parameter-'+c['id'],left+'= :boundary',parameters=params,code=expected.get('code') if expected['status']=='blocked' else None,d=model)
OUT.mkdir(exist_ok=True)
(OUT/'application.umf.json').write_text(json.dumps(doc,ensure_ascii=False,indent=2)+'\n')
(OUT/'cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
print('authored application cases:',len(cases))
