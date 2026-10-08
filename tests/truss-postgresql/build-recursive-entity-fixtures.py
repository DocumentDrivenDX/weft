"""Author complete entities combining independently admitted scalar/recursive roots."""
import base64,copy,hashlib,json
from pathlib import Path
F=Path(__file__).resolve().parent/'fixtures'
def raw(v):return json.dumps(v,separators=(',',':'),sort_keys=True).encode()
def artifact(identity,b):return dict(identity=identity,bytesBase64=base64.b64encode(b).decode(),sha256=hashlib.sha256(b).hexdigest())
optional=json.loads((F/'original-optional-scalar-inputs.json').read_text());outputs=[]
for fixture in ['address','cyclic','map','nested-sequence','numeric-address','numeric-map','tags']:
 base=copy.deepcopy(optional['requests'][0]['request']);binding=json.loads(base['target']['bindingJson']);document=json.loads(base['modules'][0]['documentJson']);composition=copy.deepcopy(optional['composition'])
 original=json.loads((F/f'original-{fixture}-compile-transport.json').read_text());compound_binding=json.loads(original['request']['target']['bindingJson']);compound_document=json.loads(original['request']['modules'][0]['documentJson']);compound_composition=json.loads((F/f'original-{fixture}-composition.json').read_text());selected=copy.deepcopy(compound_composition['properties'][0]);index=selected['index'];prop=copy.deepcopy(compound_binding['properties'][index]);root=prop['logical']['element']
 graph=json.loads(base64.b64decode(prop['valueDefinition']['bytesBase64']));identities={n['authoredIdentity']['element'] for n in graph['nodes']}
 for field in compound_document['modules'][0]['elements']:
  if field['id'] in identities:
   existing=next((i for i,e in enumerate(document['modules'][0]['elements']) if e['id']==field['id']),None)
   if existing is None:document['modules'][0]['elements'].append(copy.deepcopy(field))
   else:document['modules'][0]['elements'][existing]=copy.deepcopy(field)
 record=next(e for e in document['modules'][0]['elements'] if e['id']=='customer');record['members']=[dict(module='sales',element=id) for id in [root,'customer-note','customer-id','customer-part']]
 entity=next(e for e in binding['entities'] if e['logical']['element']=='customer');entity['acceptedDefinition']=artifact(f'recursive-entity-{fixture}',raw(record))
 binding['properties'][index]=prop;composition['properties'].append(selected)
 for group in ['entities','properties','relationships']:
  for mapped in binding[group]:mapped['source']=artifact(f'recursive-entity-source-{fixture}',raw(document))
 base['modules'][0]['documentJson']=raw(document).decode();base['modules'][0]['pin']['sha256']=hashlib.sha256(raw(document)).hexdigest();base['target']['bindingJson']=raw(binding).decode();base['target']['bindingSha256']=hashlib.sha256(raw(binding)).hexdigest()
 requests=[]
 for bound in [10,1]:
  request=copy.deepcopy(base);request['sql']=f'SELECT c.* FROM Customer c ORDER BY c.part, c.id LIMIT {bound}';request['parameters']={};requests.append(dict(bound=bound,request=request))
 outputs.append(dict(fixture=fixture,root=root,index=index,composition=composition,requests=requests))
(F/'original-recursive-entity-inputs.json').write_text(json.dumps(outputs,indent=2)+'\n')
print('Authored seven complete scalar/recursive entity model cuts.')
