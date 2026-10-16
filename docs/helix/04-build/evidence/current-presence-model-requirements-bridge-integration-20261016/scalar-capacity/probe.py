from pathlib import Path
import sys,json,hashlib,gzip,time
sys.path.insert(0,'/private/tmp/weft-model-requirements-main-combined-installed-20261016')
import weft
root=Path('/private/tmp/weft-model-requirements-scalar-capacity-r1-20261016');root.mkdir(exist_ok=False)
base=Path('/private/tmp/weft-model-requirements-bridge-main-integration-20261016/tests/current-query-presence/fixtures/unadmitted-context.request.json');request=json.loads(base.read_text());names=[f'capacity{i:04}' for i in range(1050)];revision='capacity-model-r1-'+('r'*4078);assert len(revision)==4096
D=lambda value:json.dumps(value,separators=(',',':'),ensure_ascii=False)
H=lambda value:hashlib.sha256(value.encode()).hexdigest()
ref=lambda field:{'documentId':'domain','moduleId':'m','elementId':field}
module=request['modules'][0];model=json.loads(module['documentJson']);elements=model['modules'][0]['elements'];record=next(e for e in elements if e['id']=='Resource')
for name in names:
 elements.append({'id':name,'name':name,'kind':'field','scalarType':'string','cardinality':'one','nullability':'required','extensions':{}});record['members'].append({'module':'m','element':name})
module['documentJson']=D(model);module['pin']['sha256']=H(module['documentJson']);module['pin']['revision']=revision
sources={s['role']:s for s in request['opaqueSources']};ontology=json.loads(sources['ontology']['utf8']);ontology['revision']='capacity-ontology-r1'
for pin in ontology['documents']:
 if pin['documentId']=='domain':pin['revision']=revision
entity=next(e for e in ontology['entities'] if e['type']['elementId']=='Resource');entity['fields'].extend({'ref':ref(name),'protection':'unprotected'} for name in names)
owner=json.loads(sources['owner']['utf8']);owner['revision']='capacity-owner-r1';owner['storage'].extend({'type':ref('Resource'),'field':ref(name),'storageEffect':'canonical'} for name in names)
policy=json.loads(sources['policy']['utf8']);policy['revision']='capacity-policy-r1';policy['ontology']['revision']=ontology['revision']
for role,value in [('ontology',ontology),('owner',owner),('policy',policy)]:sources[role]['utf8']=D(value);sources[role]['sha256']=H(sources[role]['utf8']);sources[role]['identity']['revision']=value['revision']
profile=json.loads(sources['presence-designation']['utf8']);profile['revision']='capacity-presence-r1';profile['modelPins']=[m['pin'] for m in request['modules']]
for pin in profile['sources']:
 s=sources[pin['role']];pin['identity']=s['identity'].copy();pin['sha256']=s['sha256']
sources['presence-designation']['utf8']=D(profile);sources['presence-designation']['sha256']=H(sources['presence-designation']['utf8']);sources['presence-designation']['identity']['revision']=profile['revision']
original=D(request);assert len(original.encode())<4_000_000;(root/'original-request.json.gz').write_bytes(gzip.compress(original.encode(),mtime=0));start=time.monotonic();stage='parent'
try:
 parent=weft.prepare_application_lineage(original);stage='presence';current=parent.prepare_current_presence();assert current.source_preparation() is parent;stage='requirements';raw=current.model_requirements_json();parsed=json.loads(raw)
 assert 8_388_608<len(raw.encode())<=16_000_000,len(raw.encode());assert parsed['sourceRequestJson']==original;assert parsed['sourceRequestSha256']==H(original);assert parsed['modelPins']==[m['pin'] for m in request['modules']]
 fields=[f for r in parsed['records'] for f in r['fields']];assert len(fields)==1062,len(fields);assert all(len(f['descriptors'])==1 and f['descriptors'][0]['kind']=='scalar' and f['descriptors'][0]['type']['nullable'] is False for f in fields)
 resource=next(r for r in parsed['records'] if r['record']['element']=='Resource');actual=[f['field']['element'] for f in resource['fields']];assert actual==['resourceId','salary','token']+names;assert all(f['field']['revision']==revision and f['descriptors'][0]['identity']==f['field'] for f in fields)
 (root/'actual-requirements.json.gz').write_bytes(gzip.compress(raw.encode(),mtime=0));result={'version':'weft-actual-scalar-requirements-capacity/0.1','passed':True,'requestBytes':len(original.encode()),'requirementsBytes':len(raw.encode()),'requestSha256':H(original),'requirementsSha256':H(raw),'recordCount':len(parsed['records']),'fieldCount':len(fields),'singleScalarDescriptorPerField':True,'allLogicalTypesNonnullable':True,'modelRevisionCharacters':len(revision),'additionalScalarFields':len(names),'sameActualParent':True,'seconds':time.monotonic()-start,'modulePath':weft.weft.__file__,'moduleSha256':hashlib.sha256(Path(weft.weft.__file__).read_bytes()).hexdigest(),'originalSourceSemanticAdmission':False,'nativeAuthority':False,'scope':'Actual compiler-issued parent/presence/model requirements; pure metadata capacity, no native admission or full UMF owner evaluation'}
except Exception as error:
 result={'passed':False,'stage':stage,'errorType':type(error).__name__,'error':str(error),'requestBytes':len(original.encode()),'requestSha256':H(original)};(root/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result));raise
(root/'receipt.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result))
