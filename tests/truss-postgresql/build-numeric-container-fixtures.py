"""Author signed64/decimal(28,2) map and structured native fixtures by identity."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def transform(v,target,family,facets,label):
 if isinstance(v,list):return [transform(x,target,family,facets,label) for x in v]
 if not isinstance(v,dict):return v
 if 'bytesBase64' in v and 'sha256' in v:
  b=base64.b64decode(v['bytesBase64'])
  try:d=json.loads(b)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=transform(d,target,family,facets,label)
  return artifact(v['identity']+'-'+label,raw(changed)) if changed!=d else copy.deepcopy(v)
 out={k:transform(x,target,family,facets,label) for k,x in v.items()}
 if out.get('kind')=='field' and out.get('id') in target:out['scalarType']=family;out['facets']=facets
 if out.get('authoredIdentity',{}).get('element') in target and out.get('shape',{}).get('kind')=='scalar':out['shape']['family']=family
 if out.get('interfaceVersion')=='truss-jsonb-leaf-codec/0.1.0':
  field=json.loads(base64.b64decode(out['authoredDefinition']['bytesBase64']))
  if field['id'] in target:out['rule']=dict(family=family,storageRepresentation='json-string',encoding='preserve-admitted-source-token',decodedCarrierKind=family,numericAdoptionEvidence=artifact('numeric-adoption',b'{}'))
 return out
for base_name,target in [('numeric-map',{'tag-item'}),('numeric-address',{'zip'})]:
 for label,family,facets,minimum,maximum in [
  ('signed','integer',{'integerWidth':{'bits':64,'signed':True}},'-9223372036854775808','9223372036854775807'),
  ('decimal','decimal',{'precision':28,'scale':2},'-99999999999999999999999999.99','99999999999999999999999999.99')]:
  shape='map' if base_name=='numeric-map' else 'structured';name=label+'-'+shape
  request=json.loads((F/f'original-{base_name}-compile-transport.json').read_text())['request'];binding=json.loads(request['target']['bindingJson']);composition=json.loads((F/f'original-{base_name}-composition.json').read_text());index=composition['properties'][0]['index']
  document=transform(json.loads(request['modules'][0]['documentJson']),target,family,facets,name)
  binding['properties'][index]=transform(binding['properties'][index],target,family,facets,name)
  for group in ['entities','properties','relationships']:
   for selected in binding[group]:selected['source']=artifact(name+'-source',raw(document))
  selected=composition['properties'][0]
  for definition in list(selected['leafCodecs'].values())+list(selected['recordPresence'].values()):
   original=transform(json.loads(definition['originalJson']),target,family,facets,name);definition['originalJson']=raw(original).decode()
   if 'originalArtifacts' in definition:
    if 'numericAdoptionEvidence' in original.get('rule',{}):definition['originalArtifacts']['rule/numericAdoptionEvidence']=original['rule']['numericAdoptionEvidence']['bytesBase64']
    for pointer in definition['originalArtifacts']:
     node=original
     for part in pointer.split('/'):node=node[int(part)] if isinstance(node,list) else node[part]
     definition['originalArtifacts'][pointer]=node['bytesBase64']
   else:definition['acceptedDefinitionBase64']=original['acceptedDefinition']['bytesBase64']
  request['modules'][0]['documentJson']=raw(document).decode();request['modules'][0]['pin']['sha256']=hashlib.sha256(raw(document)).hexdigest();request['target']['bindingJson']=raw(binding).decode();request['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
  (F/f'original-{name}-inputs.json').write_text(json.dumps(dict(request=request,composition=composition),indent=2)+'\n')
  tree=json.loads((F/f'original-{base_name}-native-tree.json').read_text());values=iter([minimum,maximum])
  codec=next(d['originalJson'].encode().hex() for d in selected['leafCodecs'].values() if json.loads(base64.b64decode(json.loads(d['originalJson'])['authoredDefinition']['bytesBase64']))['id'] in target)
  for row in tree['cells']:
   if row[12]=='integer':row[8]=row[21]=codec;row[12]=family;row[15]=row[16]=next(values)
  tree['binding']=binding;tree['fixture']=name
  if shape=='map':tree['logical']=tree['stored']={'':minimum,'9.a':maximum}
  else:tree['logical']['zip']['value']=minimum;tree['stored']['slot.1']=minimum
  (F/f'original-{name}-tree.json').write_text(json.dumps(tree,indent=2)+'\n');print('Authored '+name)
