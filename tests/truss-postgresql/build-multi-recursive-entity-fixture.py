"""Author a complete entity with independently admitted Boolean/structured roots."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
entity=next(e for e in json.loads((F/'original-recursive-entity-inputs.json').read_text()) if e['fixture']=='numeric-address');base=copy.deepcopy(entity['requests'][0]['request']);binding=json.loads(base['target']['bindingJson']);doc=json.loads(base['modules'][0]['documentJson']);composition=copy.deepcopy(entity['composition'])
sequence=json.loads((F/'original-boolean-sequence-inputs.json').read_text());sequence_binding=json.loads(sequence['request']['target']['bindingJson']);sequence_doc=json.loads(sequence['request']['modules'][0]['documentJson']);selected=copy.deepcopy(sequence['composition']['properties'][0]);index=selected['index'];prop=copy.deepcopy(sequence_binding['properties'][index]);graph=json.loads(base64.b64decode(prop['valueDefinition']['bytesBase64']));ids={n['authoredIdentity']['element'] for n in graph['nodes']}
for field in sequence_doc['modules'][0]['elements']:
 if field['id'] in ids:
  existing=next((i for i,e in enumerate(doc['modules'][0]['elements']) if e['id']==field['id']),None)
  if existing is None:doc['modules'][0]['elements'].append(copy.deepcopy(field))
  else:doc['modules'][0]['elements'][existing]=copy.deepcopy(field)
record=next(e for e in doc['modules'][0]['elements'] if e['id']=='customer');record['members']=[dict(module='sales',element=e) for e in ['tags','address','customer-note','customer-id','customer-part']]
next(e for e in binding['entities'] if e['logical']['element']=='customer')['acceptedDefinition']=artifact('multi-recursive-customer',raw(record));binding['properties'][index]=prop;composition['properties'].append(selected)
for group in ['entities','properties','relationships']:
 for mapped in binding[group]:mapped['source']=artifact('multi-recursive-source',raw(doc))
base['modules'][0]['documentJson']=raw(doc).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(doc)).hexdigest();base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
requests=[]
for bound in [10,1]:
 r=copy.deepcopy(base);r['sql']=f'SELECT c.* FROM Customer c ORDER BY c.part, c.id LIMIT {bound}';requests.append(dict(bound=bound,request=r))
(F/'original-multi-recursive-entity-inputs.json').write_text(json.dumps(dict(composition=composition,requests=requests),indent=2)+'\n');print('Authored complete Boolean sequence/numeric address entity cut.')
