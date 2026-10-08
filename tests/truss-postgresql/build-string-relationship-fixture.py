"""Author string/uint64 composite keys; SQL/results are authored separately."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,separators=(',',':'),sort_keys=True).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def convert(v):
 if isinstance(v,list):return [convert(x) for x in v]
 if not isinstance(v,dict):return v
 if 'bytesBase64' in v and 'sha256' in v:
  b=base64.b64decode(v['bytesBase64'])
  try:d=json.loads(b)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=convert(d)
  return artifact(v['identity']+'-string',raw(changed)) if changed!=d else copy.deepcopy(v)
 out={k:convert(x) for k,x in v.items()}
 if out.get('kind')=='field' and out.get('id') in ['customer-part','order-part']:
  out['scalarType']='string';out['facets']={}
 if out.get('family')=='integer':out['family']='string'
 if out.get('decodedCarrierKind')=='integer':out['decodedCarrierKind']='string'
 if out.get('decodedCarrierKind')=='string':
  out['encoding']='preserve-unicode-scalars';out.pop('numericAdoptionEvidence',None)
 if out.get('kind')=='unsigned-integer':out=dict(kind='unicode-text-C',nativeType='pg_catalog.text',encoding='UTF8',collation='pg_catalog.C',normalization='none')
 return out
def definition(d):
 original=convert(json.loads(d['originalJson']));d['originalJson']=raw(original).decode()
 d.get('originalArtifacts',{}).pop('rule/numericAdoptionEvidence',None)
 for path in d.get('originalArtifacts',{}):
  a=original
  for part in path.split('/'):a=a[int(part)] if isinstance(a,list) else a[part]
  d['originalArtifacts'][path]=a['bytesBase64']
inputs=json.loads((F/'original-composite-relationship-inputs.json').read_text());comp=inputs['composition']
base=inputs['requests'][0]['request'];binding=json.loads(base['target']['bindingJson']);document=json.loads(base['modules'][0]['documentJson'])
for field in document['modules'][0]['elements']:
 if field['id'] in ['customer-part','order-part']:field['scalarType']='string';field['facets']={}
for i,p in enumerate(binding['properties']):
 if p['logical']['element'] in ['customer-part','order-part']:
  binding['properties'][i]=convert(p)
  selected=next(x for x in comp['properties'] if x['index']==i)
  for d in selected['leafCodecs'].values():definition(d)
  definition(selected['rowJoin'])
for key,d in comp['comparators'].items():
 if json.loads(key)['field']['element'] in ['customer-part','order-part']:definition(d)
for group in ['entities','properties','relationships']:
 for p in binding[group]:p['source']=artifact('heterogeneous-source',raw(document))
for case in inputs['requests']:
 request=case['request'];request['modules'][0]['documentJson']=raw(document).decode();request['modules'][0]['pin']['sha256']=hashlib.sha256(raw(document)).hexdigest()
 request['target']['bindingJson']=raw(binding).decode();request['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
 if 'part' in request['parameters']:request['parameters']['part']=dict(family='string',value='e\u0301' if request['parameters']['part']['value']=='8' else '\u00e9')
(F/'original-string-relationship-inputs.json').write_text(json.dumps(inputs,indent=2)+'\n')
print('Authored string/uint64 composite inputs.')
