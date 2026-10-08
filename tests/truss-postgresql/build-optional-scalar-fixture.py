"""Author an optional Unicode scalar and complete ordered entity projection."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,separators=(',',':'),sort_keys=True).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
def reauthor(v):
 if isinstance(v,list):return [reauthor(x) for x in v]
 if not isinstance(v,dict):return 'customer-note' if v=='customer-part' else v
 if 'bytesBase64' in v and 'sha256' in v:
  b=base64.b64decode(v['bytesBase64'])
  try:d=json.loads(b)
  except (ValueError,UnicodeError):return copy.deepcopy(v)
  changed=reauthor(d)
  return artifact(v['identity']+'-optional',raw(changed)) if changed!=d else copy.deepcopy(v)
 out={k:reauthor(x) for k,x in v.items()}
 if out.get('kind')=='field' and out.get('id')=='customer-note':out['name']='note';out['nullability']='absent-allowed'
 if 'propertyCatalogId' in out:out['propertyCatalogId']='23'
 return out
def definition(d):
 original=reauthor(json.loads(d['originalJson']));d['originalJson']=raw(original).decode()
 for path in d.get('originalArtifacts',{}):
  a=original
  for part in path.split('/'):a=a[int(part)] if isinstance(a,list) else a[part]
  d['originalArtifacts'][path]=a['bytesBase64']
inputs=json.loads((F/'original-string-relationship-inputs.json').read_text());composition=inputs['composition'];base=copy.deepcopy(inputs['requests'][0]['request']);binding=json.loads(base['target']['bindingJson']);document=json.loads(base['modules'][0]['documentJson'])
field=reauthor(next(f for f in document['modules'][0]['elements'] if f['id']=='customer-part'));document['modules'][0]['elements'].append(field)
record=next(f for f in document['modules'][0]['elements'] if f['id']=='customer');record['members']=[dict(module='sales',element=id) for id in ['customer-note','customer-id','customer-part']]
entity=next(e for e in binding['entities'] if e['logical']['element']=='customer');entity['acceptedDefinition']=artifact('optional-customer-record',raw(record))
index=next(i for i,p in enumerate(binding['properties']) if p['logical']['element']=='customer-part');prop=reauthor(binding['properties'][index]);prop['propertyId']='23';new_index=len(binding['properties']);binding['properties'].append(prop)
selected=copy.deepcopy(next(p for p in composition['properties'] if p['index']==index));selected['index']=new_index
for leaf in selected['leafCodecs'].values():definition(leaf)
definition(selected['rowJoin']);composition['properties'].append(selected)
for group in ['entities','properties','relationships']:
 for p in binding[group]:p['source']=artifact('optional-source',raw(document))
base['modules'][0]['documentJson']=raw(document).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(document)).hexdigest();base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
requests=[]
for kind,sql in [('entity','SELECT c.* FROM Customer c ORDER BY c.part, c.id LIMIT 10'),('scalar','SELECT c.id, c.note FROM Customer c ORDER BY c.part, c.id LIMIT 10'),('entity-bounded','SELECT c.* FROM Customer c ORDER BY c.part, c.id LIMIT 2')]:
 request=copy.deepcopy(base);request['sql']=sql;request['parameters']={};requests.append(dict(kind=kind,request=request))
(F/'original-optional-scalar-inputs.json').write_text(json.dumps(dict(composition=composition,requests=requests),indent=2)+'\n')
print('Authored optional string scalar and complete three-member entity inputs.')
