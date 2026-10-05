"""Serialize the governed draft 0.2 structural schemas reproducibly."""
import json
from pathlib import Path
D=Path(__file__).resolve().parents[2]/'docs/helix/02-design/contracts'
BASE='https://github.com/DocumentDrivenDX/weft/raw/main/docs/helix/02-design/contracts/'
def obj(**properties):return dict(type='object',properties=properties,required=list(properties),additionalProperties=False)
def ref(name):return {'$ref':'#/$defs/'+name}
def arr(items,**bounds):return dict(type='array',items=items,**bounds)
def tagged(tag,**props):return obj(**{tag:props.pop(tag)},**props)
s={'type':'string','minLength':1};integer={'type':'integer','minimum':0};null={'type':'null'}
old=json.loads((D/'logical-plan.schema.json').read_text())
defs={
 'identity':obj(documentId=s,revision=s,module=s,element=s),
 'pin':json.loads((D/'compile-request.schema.json').read_text())['properties']['modules']['items']['properties']['pin'],
 'span':obj(start=integer,end=integer),
 'type':obj(family={'enum':['boolean','string','integer','decimal']},facets={'type':'object'},nullable={'type':'boolean'}),
 'member':obj(name=s,identity=ref('identity')),
 'profile':obj(version={'const':'weft-application-read/0.2.0'},subset={'enum':['entity-page','count-summary','related-entity-page']}),
 'field':obj(scan=s,identity=ref('identity'),type=ref('type'),span=ref('span')),
 'key':obj(id=s,fields=arr(ref('identity'),minItems=1),types=arr(ref('type'),minItems=1)),
 'relationshipIdentity':obj(documentId=s,revision=s,module=s,relationship=s),
 'multiplicity':dict(type='object',properties={'min':integer,'max':{'anyOf':[dict(type='integer',minimum=1),{'const':'*'}]}},required=['min','max']),
 'scan':obj(occurrence=s,record=ref('identity'),pin=ref('pin')),
}
def variant(tag,label,**props):return obj(**{tag:{'const':label}},**props)
defs['descriptor']={'oneOf':[
 variant('kind','scalar',identity=ref('identity'),availability={'enum':['required','absent-allowed']},type=ref('type')),
 *[variant('kind',kind,identity=ref('identity'),availability={'enum':['required','absent-allowed']},item=ref('identity')) for kind in ['sequence','map']],
 variant('kind','structured',identity=ref('identity'),availability={'enum':['required','absent-allowed']},record=ref('identity')),
 variant('kind','record',identity=ref('identity'),availability=null,members=arr(ref('member'))),
]}
defs['relationship']=obj(identity=ref('relationshipIdentity'),inverse={'type':'boolean'},**{'from':ref('identity'),'to':ref('identity')},sourceKey=ref('key'),targetKey=ref('key'),sourceMultiplicity=ref('multiplicity'),targetMultiplicity=ref('multiplicity'),targetLifecycle={'enum':['owned','independent']})
defs['value']={'oneOf':[
 variant('kind','literal',value={'type':'string'},type=ref('type'),span=ref('span')),
 variant('kind','parameter',name=s,value={'type':'string'},type=ref('type'),span=ref('span')),
 variant('kind','field',field=ref('field')),
]}
defs['predicate']={'oneOf':[
 variant('op','equal',left=ref('field'),right=ref('value')),
 variant('op','lexicographicGreater',columns=arr(ref('field'),minItems=1),values=arr(ref('value'),minItems=1)),
 variant('op','hasRelated',scan=s,relationship=ref('relationship'),key=arr(ref('value'),minItems=1)),
]}
defs['expression']={'oneOf':[
 variant('op','field',scan=s,identity=ref('identity')),
 variant('op','count',type=ref('type')),
 variant('op','sum',argument=ref('field'),type=ref('type')),
 variant('op','relatedKeys',scan=s,relationship=ref('relationship'),bound={'type':'integer','minimum':1,'maximum':1000}),
]}
plan=obj(irVersion={'const':'weft-ir/0.2.0'},modulePins=arr(ref('pin'),minItems=1,maxItems=32),readProfile={'anyOf':[ref('profile'),null]},requiredCapabilities=arr(s,uniqueItems=True),typeGraph=arr(ref('descriptor'),maxItems=131072),source=ref('scan'),pageKey={'anyOf':[ref('key'),null]},joins=arr(obj(right=ref('scan'),on=arr(ref('predicate'),minItems=1)),maxItems=16),filters=arr(ref('predicate')),groups=arr(ref('field')),aggregate={'type':'boolean'},outputs=arr(obj(name=s,expression=ref('expression')),minItems=1,maxItems=256),order=arr(ref('field')),limit={'anyOf':[dict(type='integer',minimum=1,maximum=1000),null]})
plan.update({'$schema':'https://json-schema.org/draft/2020-12/schema','$id':BASE+'logical-plan-v0.2.schema.json','$defs':defs})
request=json.loads((D/'compile-request.schema.json').read_text());request['$id']=BASE+'compile-request-v0.2.schema.json';request['properties']['interfaceVersion']={'const':'weft-compile/0.2.0'};request['properties']['dialect']={'const':'weft-sql/0.2.0'};request['properties']['readProfile']=defs['profile'];request['properties']['parameters']={'type':'object','maxProperties':1024,'propertyNames':{'pattern':'^[A-Za-z_][A-Za-z0-9_]*$'},'additionalProperties':obj(family={'enum':['boolean','string','integer','decimal']},value={'type':'string'})}
# Result values are typed data carriers. Binding qualification decides whether null is permitted.
presence={'oneOf':[variant('state','absent'),variant('state','null'),variant('state','value',value={})]}
related=obj(items=arr(arr({'type':'string'},minItems=1)),truncated={'type':'boolean'})
result={'$schema':'https://json-schema.org/draft/2020-12/schema','$id':BASE+'application-result-v0.2.schema.json','oneOf':[{'$ref':'#/$defs/presence'},{'$ref':'#/$defs/relatedKeys'}],'$defs':{'presence':presence,'relatedKeys':related}}
for name,value in [('logical-plan-v0.2.schema.json',plan),('compile-request-v0.2.schema.json',request),('application-result-v0.2.schema.json',result)]:
 (D/name).write_text(json.dumps(value,indent=2)+'\n')
