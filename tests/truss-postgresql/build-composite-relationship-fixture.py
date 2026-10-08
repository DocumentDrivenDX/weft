"""Author independent composite key models and exact original composition inputs."""
import base64, copy, hashlib, json
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2];F=ROOT/'tests/truss-postgresql/fixtures'
def raw(v):return json.dumps(v,separators=(',',':'),sort_keys=True).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def decoded(a):return json.loads(base64.b64decode(a['bytesBase64']))
def reauthor(v,old,new,name,pid):
 if isinstance(v,list):return [reauthor(x,old,new,name,pid) for x in v]
 if not isinstance(v,dict):return new if v==old else v
 if 'bytesBase64' in v and 'sha256' in v:
  b=base64.b64decode(v['bytesBase64'])
  try:d=decoded(v)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=reauthor(d,old,new,name,pid)
  if changed==d:return copy.deepcopy(v)
  return artifact(v['identity']+'-'+new,raw(changed))
 out={k:reauthor(x,old,new,name,pid) for k,x in v.items()}
 if v.get('kind')=='field' and v.get('id')==old:out['name']=name
 if 'propertyCatalogId' in v:out['propertyCatalogId']=str(pid)
 return out
def selection(v,old,new,name,pid):
 out=copy.deepcopy(v)
 for kind in ['leafCodecs','recordPresence']:
  for d in out[kind].values():rewrite_definition(d,old,new,name,pid)
 rewrite_definition(out['rowJoin'],old,new,name,pid)
 return out
def rewrite_definition(d,old,new,name,pid):
 original=reauthor(json.loads(d['originalJson']),old,new,name,pid)
 d['originalJson']=raw(original).decode()
 for path in d.get('originalArtifacts',{}):
  a=original
  for part in path.split('/'):a=a[int(part)] if isinstance(a,list) else a[part]
  d['originalArtifacts'][path]=a['bytesBase64']
 if 'acceptedDefinitionBase64' in d:d['acceptedDefinitionBase64']=original['acceptedDefinition']['bytesBase64']
base=json.loads((F/'original-relationship-public-transport.json').read_text())[0]['request']
composition=json.loads((F/'original-relationship-composition.json').read_text())
binding=json.loads(base['target']['bindingJson']);document=json.loads(base['modules'][0]['documentJson'])
for owner,old,new,name,pid in [('customer','customer-id','customer-part','part',20),('orders','order-id','order-part','part',21),('orders','order-id','order-third','third',22)]:
 field=next(x for x in document['modules'][0]['elements'] if x['id']==old)
 field=reauthor(field,old,new,name,pid);document['modules'][0]['elements'].append(field)
 record=next(x for x in document['modules'][0]['elements'] if x['id']==owner)
 record['members'].append(dict(module='sales',element=new))
 index=next(i for i,p in enumerate(binding['properties']) if p['logical']['element']==old)
 prop=reauthor(binding['properties'][index],old,new,name,pid);prop['propertyId']=str(pid)
 new_index=len(binding['properties']);binding['properties'].append(prop)
 selected=selection(next(p for p in composition['properties'] if p['index']==index),old,new,name,pid)
 selected['index']=new_index;composition['properties'].append(selected)
 key,comp=next((k,v) for k,v in composition['comparators'].items() if json.loads(k)['field']['element']==old)
 identity=reauthor(json.loads(key),old,new,name,pid);comp=copy.deepcopy(comp);rewrite_definition(comp,old,new,name,pid)
 composition['comparators'][raw(identity).decode()]=comp
for owner,fields in [('customer',['customer-part','customer-id']),('orders',['order-part','order-id','order-third'])]:
 record=next(x for x in document['modules'][0]['elements'] if x['id']==owner)
 record['keys'][0]['fields']=[dict(module='sales',element=x) for x in fields]
 mapped=next(p for p in binding['entities'] if p['logical']['element']==owner)
 mapped['acceptedDefinition']=artifact('composite-record-'+owner,raw(record))
 key=next(k for k in binding['keys'] if k['ownerTypeId']==mapped['typeId'])
 key['acceptedDefinition']=artifact('composite-key-'+owner,raw(record['keys'][0]))
 ordered=[next(p['propertyId'] for p in binding['properties'] if p['logical']['element']==field) for field in fields]
 key['orderedPropertyIds']=ordered
 binding['relationships'][0]['sourceOrderedPropertyIds' if owner=='customer' else 'targetOrderedPropertyIds']=ordered
for group in ['entities','properties','relationships']:
 for p in binding[group]:p['source']=artifact('composite-source',raw(document))
base['modules'][0]['documentJson']=raw(document).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(document)).hexdigest()
base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
requests=[]
for direction,sql,parameters in [
 ('inverse','SELECT COUNT(*) AS n FROM Orders o WHERE HAS_RELATED(o.customer, KEY(:part, :id))',{'part':'7','id':'9007199254740993'}),
 ('forward','SELECT COUNT(*) AS n FROM Customer c WHERE HAS_RELATED(c.orders, KEY(:part, :id, :third))',{'part':'8','id':'100','third':'9'}),
 ('inverse','SELECT o.part, o.id, o.third, RELATED_KEYS(o.customer, 2) AS customers FROM Orders o ORDER BY o.part, o.id, o.third LIMIT 10',{}),
 ('forward','SELECT c.part, c.id, RELATED_KEYS(c.orders, 2) AS orders FROM Customer c ORDER BY c.part, c.id LIMIT 10',{}),
 ('inverse','SELECT o.part, o.id, o.third, RELATED_KEYS(o.customer, 2) AS customers FROM Orders o WHERE (o.part, o.id, o.third) > (:part, :id, :third) ORDER BY o.part, o.id, o.third LIMIT 10',{'part':'8','id':'100','third':'9'}),
 ('forward','SELECT c.part, c.id, RELATED_KEYS(c.orders, 2) AS orders FROM Customer c WHERE (c.part, c.id) > (:part, :id) ORDER BY c.part, c.id LIMIT 10',{'part':'7','id':'0'}),
]:
 request=copy.deepcopy(base);request['sql']=sql;request['parameters']={name:dict(family='integer',value=value) for name,value in parameters.items()}
 requests.append(dict(direction=direction,kind='has' if 'COUNT' in sql else ('cursor' if 'WHERE' in sql else 'keys'),request=request))
(F/'original-composite-relationship-inputs.json').write_text(json.dumps(dict(composition=composition,requests=requests),indent=2)+'\n')
print('Authored composite 2/3 endpoint key inputs (no emitted SQL expectations).')
