"""Author recursive signed64/decimal(28,2) fixtures from original sequence definitions.
Original model/codec bytes are reauthored and pinned before compiler admission.
"""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def transform(v,family,facets,label):
 if isinstance(v,list):return [transform(x,family,facets,label) for x in v]
 if not isinstance(v,dict):return v
 if 'bytesBase64' in v and 'sha256' in v:
  b=base64.b64decode(v['bytesBase64'])
  try:d=json.loads(b)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=transform(d,family,facets,label)
  return artifact(v['identity']+'-'+label,raw(changed)) if changed!=d else copy.deepcopy(v)
 out={k:transform(x,family,facets,label) for k,x in v.items()}
 if out.get('kind')=='field' and out.get('id')=='tag-item':out['scalarType']=family;out['facets']=facets
 if out.get('kind')=='scalar' and out.get('family')=='string':out['family']=family;out['storageRepresentation']='json-string'
 if out.get('interfaceVersion')=='truss-jsonb-leaf-codec/0.1.0':
  out['rule']=dict(family=family,storageRepresentation='json-string',encoding='preserve-admitted-source-token',decodedCarrierKind=family,numericAdoptionEvidence=artifact('numeric-adoption',b'{}'))
 return out
for label,family,facets,values in [
 ('signed','integer',{'integerWidth':{'bits':64,'signed':True}},['-9223372036854775808','9223372036854775807','-9223372036854775808']),
 ('decimal','decimal',{'precision':28,'scale':2},['-99999999999999999999999999.99','99999999999999999999999999.99','-99999999999999999999999999.99']),
]:
 base=json.loads((F/'original-tags-compile-transport.json').read_text())['request'];binding=json.loads(base['target']['bindingJson'])
 doc=transform(json.loads(base['modules'][0]['documentJson']),family,facets,label)
 composition=json.loads((F/'original-tags-composition.json').read_text());index=composition['properties'][0]['index'];binding['properties'][index]=transform(binding['properties'][index],family,facets,label)
 for group in ['entities','properties','relationships']:
  for p in binding[group]:p['source']=artifact(label+'-sequence-source',raw(doc))
 for leaf in composition['properties'][0]['leafCodecs'].values():
  original=transform(json.loads(leaf['originalJson']),family,facets,label);leaf['originalJson']=raw(original).decode()
  leaf['originalArtifacts']['rule/numericAdoptionEvidence']=original['rule']['numericAdoptionEvidence']['bytesBase64']
  for pointer in leaf['originalArtifacts']:
   selected=original
   for part in pointer.split('/'):selected=selected[int(part)] if isinstance(selected,list) else selected[part]
   leaf['originalArtifacts'][pointer]=selected['bytesBase64']
 base['modules'][0]['documentJson']=raw(doc).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(doc)).hexdigest();base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
 (F/f'original-{label}-sequence-inputs.json').write_text(json.dumps(dict(request=base,composition=composition),indent=2)+'\n')
 tree=json.loads((F/'original-tags-native-tree.json').read_text());codec=next(iter(composition['properties'][0]['leafCodecs'].values()))['originalJson'].encode().hex()
 for row,value in zip(tree['cells'][1:],values):
  row[8]=row[21]=codec;row[12]=family;row[13]=None;row[15]=row[16]=value
 tree['binding']=binding;tree['fixture']=label+'-sequence';tree['logical']=tree['stored']=values
 (F/f'original-{label}-sequence-tree.json').write_text(json.dumps(tree,indent=2)+'\n')
 print('Authored original '+label+' sequence and independent native cells.')
