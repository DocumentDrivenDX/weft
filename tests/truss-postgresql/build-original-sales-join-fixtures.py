"""Author original sales-query bindings for every selected row/props home cut.
No emitted SQL or compiler result is used to author schema/binding inputs.
"""
import base64,copy,hashlib,itertools,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def decode(a):return json.loads(base64.b64decode(a['bytesBase64']))
base=json.loads((F/'original-multi-recursive-entity-inputs.json').read_text());request=copy.deepcopy(base['requests'][0]['request']);binding=json.loads(request['target']['bindingJson']);doc=json.loads(request['modules'][0]['documentJson']);configuration=base['composition']
fields={e['id']:e for e in doc['modules'][0]['elements'] if e['kind']=='field'}
for record in doc['modules'][0]['elements']:
 if record['id']=='customer':
  for element in ['customer-name']:
   if not any(m['element']==element for m in record['members']):record['members'].append(dict(module='sales',element=element))
 if record['id']=='orders':
  for element in ['order-customer','order-total']:
   if not any(m['element']==element for m in record['members']):record['members'].append(dict(module='sales',element=element))

def rebind(v,old,new):
 if isinstance(v,list):return [rebind(x,old,new) for x in v]
 if not isinstance(v,dict):return v
 if 'bytesBase64' in v and 'sha256' in v:
  try:d=decode(v)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=rebind(d,old,new)
  return artifact(v['identity']+'-sales-'+new,raw(changed)) if changed!=d else copy.deepcopy(v)
 if v.get('kind')=='field' and v.get('id')==old:return copy.deepcopy(fields[new])
 out={k:rebind(x,old,new) for k,x in v.items()}
 if out.get('element')==old:out['element']=new
 return out

def definition(v,old,new):
 v=copy.deepcopy(v);d=rebind(json.loads(v['originalJson']),old,new);v['originalJson']=raw(d).decode()
 for path in v.get('originalArtifacts',{}):
  a=d
  for part in path.split('/'):a=a[int(part)] if isinstance(a,list) else a[part]
  v['originalArtifacts'][path]=a['bytesBase64']
 return v

def comparator(c,element):return next(v for k,v in c['comparators'].items() if json.loads(k)['field']['element']==element)
templates=json.loads((F/'original-sales-decimal-templates.json').read_text())
decimal={home:(dict(properties=[None]*8+[value['property']]),dict(properties=[value['selection']],comparators=value['comparators'])) for home,value in templates.items()}
props_binding,props_configuration=decimal['props'];props_template=props_configuration['properties'][0]
entries=[]
for homes in itertools.product(['row','props'],repeat=4):
 b=copy.deepcopy(binding);c=copy.deepcopy(configuration);c['properties']=[];c['comparators']={};c['relationships']=[]
 for index,old,new,owner,template_index,home in zip([0,1,7,8],['customer-id','customer-part','customer-id','order-total'],['customer-id','customer-name','order-customer','order-total'],['customer','customer','orders','orders'],[0,11,0,8],homes):
  tb,tc=decimal[home] if index==8 else (binding,configuration)
  prop=rebind(tb['properties'][template_index],old,new);selected=copy.deepcopy(next(p for p in tc['properties'] if p['index']==template_index));selected['index']=index
  selected['leafCodecs']={k:definition(v,old,new) for k,v in selected['leafCodecs'].items()}
  prop['propertyId']=str(index);prop['ownerTypeId']='-1' if owner=='customer' else '-2';prop['home']=home
  if home=='props':
   home_definition=decode(props_binding['properties'][8]['homeDefinition']);home_definition['ownerCatalogId']=prop['ownerTypeId'];home_definition['propertyCatalogId']=str(index);home_definition['memberName']=str(index)
   prop['homeDefinition']=artifact('sales-props-home-'+new,raw(home_definition));prop['homeProfile']=props_binding['properties'][8]['homeProfile']
   for key in ['inventory','relations','columns','obligations','rowJoin']:selected[key]=copy.deepcopy(props_template[key])
  if home=='row':
   home_definition=decode(prop['homeDefinition']);home_definition['ownerCatalogId']=prop['ownerTypeId'];home_definition['propertyCatalogId']=str(index);prop['homeDefinition']=artifact('sales-row-home-'+new,raw(home_definition))
  b['properties'][index]=prop;c['properties'].append(selected)
  d=definition(comparator(tc,old),old,new)
  identity=copy.deepcopy(prop['logical']);identity['element']=owner
  key=raw(dict(owner=identity,field=prop['logical'])).decode();c['comparators'][key]=d
 for group in ['entities','properties','relationships']:
  for mapped in b[group]:mapped['source']=artifact('sales-join-source',raw(doc))
 for entity in b['entities']:
  record=next(e for e in doc['modules'][0]['elements'] if e['id']==entity['logical']['element']);entity['acceptedDefinition']=artifact('sales-join-record-'+record['id'],raw(record))
 q=copy.deepcopy(request);q['modules'][0]['documentJson']=raw(doc).decode();q['modules'][0]['pin']['sha256']=hashlib.sha256(raw(doc)).hexdigest();q['target']['bindingJson']=raw(b).decode();q['target']['bindingSha256']=hashlib.sha256(raw(b)).hexdigest();q['sql']='SELECT c.name, SUM(o.total) AS total FROM Customer c JOIN Orders o ON o.customer_id = c.id GROUP BY c.name';q['parameters']={}
 entries.append(dict(homes=list(homes),configuration=c,request=q))
(F/'original-sales-join-inputs.json').write_text(json.dumps(entries,indent=2)+'\n');print(f'Authored {len(entries)} original sales join home cuts')
