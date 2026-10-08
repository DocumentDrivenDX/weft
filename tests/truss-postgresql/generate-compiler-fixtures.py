"""Preserve independent corpus queries/results while changing only physical homes."""
import json,base64,hashlib,copy
from pathlib import Path
ROOT=Path(__file__).resolve().parents[2]
BASE=json.loads((Path(__file__).parent/'fixtures/binding-props.json').read_text())
source=json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_text())
def artifact(value):
    raw=json.dumps(value,ensure_ascii=False,separators=(',',':')).encode()
    h=hashlib.sha256(raw).hexdigest()
    return dict(identity='candidate-fixture-'+h,bytesBase64=base64.b64encode(raw).decode(),sha256=h)
pin=BASE['bindingProfile']
def make(request,home):
    b=copy.deepcopy(BASE);b.update(bindingProfileId='truss-postgresql-candidate/0.1.0',entities=[],properties=[],keys=[],relationships=[])
    b['basis'].update(modelBundle=artifact(request['modules']),namespace='pg_temp')
    mappings=[];prop=0
    for module_input in request['modules']:
        doc=json.loads(module_input['documentJson']);revision=module_input['pin']['revision']
        for module in doc['modules']:
            lookup={(m['id'],e['id']):e for m in doc['modules'] for e in m['elements']}
            for element in module['elements']:
                if element['kind']!='record':continue
                typeid=str(-1-len(b['entities']))
                identity=dict(documentId=doc['id'],revision=revision,module=module['id'],element=element['id'])
                b['entities'].append(dict(logical=identity,typeId=typeid,acceptedDefinition=artifact(element),source=artifact(doc)))
                for ref in element['members']:
                    field=lookup[(ref['module'],ref['element'])];propid=str(prop);prop+=1
                    logical={**identity,'module':ref['module'],'element':ref['element']}
                    p=copy.deepcopy(BASE['properties'][0]);p.update(logical=logical,ownerTypeId=typeid,propertyId=propid,home=home,acceptedDefinition=artifact(field),source=artifact(doc))
                    if home=='props':
                        h=dict(interfaceVersion='truss-property-home/0.1.0',layoutInventory=b['basis']['layoutInventory'],recordKind='object',relationPhysicalIdentity='object-table',propsColumnPhysicalIdentity='object-props',discriminatorColumnPhysicalIdentity='object-type',relationName='object',propsColumnName='props',discriminatorColumnName='type_id',ownerCatalogId=typeid,propertyCatalogId=propid,memberName=propid,accessor='jsonb-top-level-member',presenceProfile=pin,valueProfile=pin)
                    else:
                        h=dict(interfaceVersion='truss-row-home/0.1.0',layoutInventory=b['basis']['layoutInventory'],recordKind='object',ownerCatalogId=typeid,propertyCatalogId=propid,stateRelationPhysicalIdentity='state',nodeRelationPhysicalIdentity='node',scalarRelationPhysicalIdentity='scalar',joinProfile=pin,joinDefinition=artifact({}),access='scalar-root' if 'scalarType' in field else 'complete-value-tree',presenceDefinition=p['presenceDefinition'],valueDefinition=p['valueDefinition'],storedDomainObligation='truss.candidate.context')
                    p['homeDefinition']=artifact(h);b['properties'].append(p)
                    mappings.append(dict(logical=logical,record=element['name'],typeId=typeid,propertyId=propid,name=field['name'],family=field.get('scalarType','compound'),itemFamily=lookup[(field['itemType']['module'],field['itemType']['element'])].get('scalarType') if 'itemType' in field else None))
                    refs=field.get('references',[])
                    if refs and refs[0]['role']=='record-type':
                        record=lookup[(refs[0]['module'],refs[0]['element'])]
                        nested=[]
                        for child_ref in record['members']:
                            child=lookup[(child_ref['module'],child_ref['element'])]
                            nested.append(dict(name=child['name'],family=child.get('scalarType'),optional=child['nullability']=='absent-allowed',identity=dict(documentId=doc['id'],revision=revision,module=child_ref['module'],element=child_ref['element'])))
                        mappings[-1]['structuredMembers']=nested
                for number,key in enumerate(element.get('keys',[])):
                    components=[]
                    for ref in key['fields']:
                        logical={**identity,'module':ref['module'],'element':ref['element']}
                        components.append(next(p['propertyId'] for p in b['properties'] if p['ownerTypeId']==typeid and p['logical']==logical))
                    b['keys'].append(dict(ownerTypeId=typeid,keyId=key['id'],keyNumber=str(number),orderedPropertyIds=components,comparisonProfile=pin,encodingProfile=pin,acceptedDefinition=artifact(key),comparisonDefinition=artifact({}),encodingDefinition=artifact({})))
    for module_input in request['modules']:
        doc=json.loads(module_input['documentJson']);revision=module_input['pin']['revision']
        for module in doc['modules']:
            for rel in module.get('relationships',[]):
                endpoints=[]
                for side in ['source','target']:
                    ref=rel[side][0]
                    logical=dict(documentId=doc['id'],revision=revision,module=ref['module'],element=ref['element'])
                    owner=next(e for e in b['entities'] if e['logical']==logical)
                    keys=[k for k in b['keys'] if k['ownerTypeId']==owner['typeId']]
                    authored=json.loads(base64.b64decode(owner['acceptedDefinition']['bytesBase64']))
                    kid=ref.get('key') or next(k['id'] for k in authored['keys'] if k.get('primary'))
                    key=next(k for k in keys if k['keyId']==kid)
                    endpoints.append((owner['typeId'],key['keyId'],key['orderedPropertyIds']))
                source,target=endpoints
                b['relationships'].append(dict(logical=dict(documentId=doc['id'],revision=revision,module=module['id'],relationship=rel['id']),relationshipId=str(len(b['relationships'])),sourceTypeId=source[0],targetTypeId=target[0],sourceKeyId=source[1],targetKeyId=target[1],sourceOrderedPropertyIds=source[2],targetOrderedPropertyIds=target[2],relationshipProfile=pin,acceptedDefinition=artifact(rel),source=artifact(doc)))
    return b,mappings
cases=[]
for c in source:
    if 'rows' not in c['expected']:continue
    for home in ['props','row']:
        req=copy.deepcopy(c['request']);binding,mapping=make(req,home);raw=json.dumps(binding,ensure_ascii=False,separators=(',',':'))
        req['target']=dict(backendId='truss.postgresql',backendVersion='0.1.0-candidate',targetProfile='pg17.9-candidate',bindingJson=raw,bindingSha256=hashlib.sha256(raw.encode()).hexdigest());req['options']={'allowCandidate':True}
        cases.append(dict(id=c['id']+'-'+home,home=home,request=req,mapping=mapping,expected=c['expected']))
(Path(__file__).parent/'fixtures/compiler-cases.json').write_text(json.dumps(cases,ensure_ascii=False,indent=2)+'\n')
print('Candidate relational compiler cases:',len(cases))
