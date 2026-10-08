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
 if out.get('kind')=='field' and out.get('id')=='tag-item':out['scalarType']='boolean';out.pop('facets',None)
 if out.get('kind')=='scalar' and out.get('family')=='string':out['family']='boolean';out['storageRepresentation']='json-boolean'
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
base=json.loads((F/'original-tags-compile-transport.json').read_text())['request'];binding=json.loads(base['target']['bindingJson']);doc=boolean(json.loads(base['modules'][0]['documentJson']));composition=json.loads((F/'original-tags-composition.json').read_text());index=composition['properties'][0]['index'];binding['properties'][index]=boolean(binding['properties'][index])
for group in ['entities','properties','relationships']:
 for p in binding[group]:p['source']=artifact('boolean-sequence-source',raw(doc))
for leaf in composition['properties'][0]['leafCodecs'].values():definition(leaf)
base['modules'][0]['documentJson']=raw(doc).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(doc)).hexdigest();base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
(F/'original-boolean-sequence-inputs.json').write_text(json.dumps(dict(request=base,composition=composition),indent=2)+'\n')
tree=json.loads((F/'original-tags-native-tree.json').read_text());leaf=next(iter(composition['properties'][0]['leafCodecs'].values()))['originalJson'].encode().hex()
for row,value in zip(tree['cells'][1:],[False,True,False]):
 row[8]=row[21]=leaf;row[12]='boolean';row[13]=None;row[14]=str(value).lower()
tree['binding']=binding;tree['fixture']='boolean-sequence';tree['logical']=[False,True,False];tree['stored']=[False,True,False]
(F/'original-boolean-sequence-tree.json').write_text(json.dumps(tree,indent=2)+'\n')
print('Authored original Boolean sequence and independent native cells.')
