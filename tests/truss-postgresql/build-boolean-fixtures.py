"""Author independent original Boolean property/comparator cuts from signed scaffolding."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def boolean(v):
 if isinstance(v,list):return [boolean(x) for x in v]
 if not isinstance(v,dict):return v
 if 'bytesBase64' in v and 'sha256' in v:
  b=base64.b64decode(v['bytesBase64'])
  try:d=json.loads(b)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=boolean(d)
  return artifact(v['identity']+'-boolean',raw(changed)) if changed!=d else copy.deepcopy(v)
 out={k:boolean(x) for k,x in v.items()}
 if out.get('kind')=='field' and out.get('id')=='customer-id':out['scalarType']='boolean';out.pop('facets',None)
 if out.get('kind')=='scalar' and out.get('family')=='integer':out['family']='boolean';out['storageRepresentation']='json-boolean'
 if out.get('interfaceVersion')=='truss-jsonb-leaf-codec/0.1.0':out['rule']=dict(family='boolean',storageRepresentation='json-boolean',encoding='preserve-boolean',decodedCarrierKind='boolean')
 if 'strategy' in out:out['strategy']=dict(kind='native-boolean',nativeType='pg_catalog.bool')
 return out
def definition(d):
 original=boolean(json.loads(d['originalJson']));d['originalJson']=raw(original).decode()
 for path in list(d.get('originalArtifacts',{})):
  a=original
  try:
   for part in path.split('/'):a=a[int(part)] if isinstance(a,list) else a[part]
  except KeyError:del d['originalArtifacts'][path];continue
  d['originalArtifacts'][path]=a['bytesBase64']
entries=json.loads((F/'original-signed-public-transport.json').read_text());outputs=[]
for home in ['row','props']:
 base=copy.deepcopy(next(e['request'] for e in entries if e['bits']==64 and e['home']==home and e['kind']=='page'));binding=json.loads(base['target']['bindingJson']);doc=boolean(json.loads(base['modules'][0]['documentJson']));composition=json.loads((F/f'original-signed-64-{home}-composition.json').read_text());index=composition['properties'][0]['index'];binding['properties'][index]=boolean(binding['properties'][index])
 for group in ['entities','properties','relationships']:
  for p in binding[group]:p['source']=artifact('boolean-source',raw(doc))
 for leaf in composition['properties'][0]['leafCodecs'].values():definition(leaf)
 for comparator in composition['comparators'].values():definition(comparator)
 base['modules'][0]['documentJson']=raw(doc).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(doc)).hexdigest();base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
 requests=[]
 for kind,sql,params in [('page','SELECT c.id FROM Customer c ORDER BY c.id LIMIT 2',{}),('true','SELECT c.id FROM Customer c WHERE c.id = :flag ORDER BY c.id LIMIT 2',{'flag':{'family':'boolean','value':'true'}}),('false','SELECT c.id FROM Customer c WHERE c.id = :flag ORDER BY c.id LIMIT 2',{'flag':{'family':'boolean','value':'false'}})]:
  r=copy.deepcopy(base);r['sql']=sql;r['parameters']=params;requests.append(dict(kind=kind,request=r))
 outputs.append(dict(home=home,composition=composition,requests=requests))
(F/'original-boolean-inputs.json').write_text(json.dumps(outputs,indent=2)+'\n');print('Authored two original Boolean storage cuts.')
