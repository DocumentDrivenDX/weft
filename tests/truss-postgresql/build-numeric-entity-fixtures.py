"""Author complete entities with two independent native signed/decimal roots."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,sort_keys=True,separators=(',',':')).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
base_entity=next(e for e in json.loads((F/'original-recursive-entity-inputs.json').read_text()) if e['fixture']=='numeric-address')
for tags,address in [('signed-sequence','decimal-structured'),('decimal-sequence','signed-structured'),('signed-map','decimal-structured'),('decimal-map','signed-structured')]:
 name=tags+'-'+address;request=copy.deepcopy(base_entity['requests'][0]['request']);binding=json.loads(request['target']['bindingJson']);document=json.loads(request['modules'][0]['documentJson']);composition=copy.deepcopy(base_entity['composition'])
 for source_name in [tags,address]:
  cut=json.loads((F/f'original-{source_name}-inputs.json').read_text());selected=copy.deepcopy(cut['composition']['properties'][0]);index=selected['index'];prop=json.loads(cut['request']['target']['bindingJson'])['properties'][index];graph=json.loads(base64.b64decode(prop['valueDefinition']['bytesBase64']));ids={n['authoredIdentity']['element'] for n in graph['nodes']}
  source_document=json.loads(cut['request']['modules'][0]['documentJson'])
  for field in source_document['modules'][0]['elements']:
   if field['id'] in ids:
    existing=next((i for i,e in enumerate(document['modules'][0]['elements']) if e['id']==field['id']),None)
    if existing is None:document['modules'][0]['elements'].append(copy.deepcopy(field))
    else:document['modules'][0]['elements'][existing]=copy.deepcopy(field)
  binding['properties'][index]=prop
  composition['properties']=[p for p in composition['properties'] if p['index']!=index]+[selected]
 record=next(e for e in document['modules'][0]['elements'] if e['id']=='customer');record['members']=[dict(module='sales',element=e) for e in ['tags','address','customer-note','customer-id','customer-part']]
 next(e for e in binding['entities'] if e['logical']['element']=='customer')['acceptedDefinition']=artifact(name+'-customer',raw(record))
 for group in ['entities','properties','relationships']:
  for selected in binding[group]:selected['source']=artifact(name+'-source',raw(document))
 request['modules'][0]['documentJson']=raw(document).decode();request['modules'][0]['pin']['sha256']=hashlib.sha256(raw(document)).hexdigest();request['target']['bindingJson']=raw(binding).decode();request['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
 requests=[]
 for bound in [10,1]:
  q=copy.deepcopy(request);q['sql']=f'SELECT c.* FROM Customer c ORDER BY c.part, c.id LIMIT {bound}';requests.append(dict(bound=bound,request=q))
 (F/f'original-entity-{name}-inputs.json').write_text(json.dumps(dict(tags=tags,address=address,composition=composition,requests=requests),indent=2)+'\n');print('Authored '+name)
