"""Authored candidate admission inputs, not accepted deployment evidence."""
import json,base64,hashlib
from pathlib import Path
OUT=Path(__file__).parent/'fixtures';OUT.mkdir(exist_ok=True)
def artifact(value):
    raw=json.dumps(value,separators=(',',':')).encode()
    return dict(identity='candidate-test-artifact',bytesBase64=base64.b64encode(raw).decode(),sha256=hashlib.sha256(raw).hexdigest())
a=artifact({});pin=dict(identity='candidate-test-profile',version='0.1.0',sha256=hashlib.sha256(b'{}').hexdigest())
logical=dict(documentId='d',revision='r',module='m',element='entity')
home=dict(interfaceVersion='truss-property-home/0.1.0',layoutInventory=a,recordKind='object',relationPhysicalIdentity='object-table',propsColumnPhysicalIdentity='object-props',discriminatorColumnPhysicalIdentity='object-type',relationName='object',propsColumnName='props',discriminatorColumnName='type_id',ownerCatalogId='-1',propertyCatalogId='0',memberName='0',accessor='jsonb-top-level-member',presenceProfile=pin,valueProfile=pin)
binding=dict(interfaceVersion='truss-postgresql-binding/0.1.0',bindingProfile=pin,bindingProfileId='candidate-test-binding/0.1.0',basis=dict(modelBundle=a,catalogRevision='0',acceptedCatalog=a,layoutProfile=pin,layoutInventory=a,layoutSql=a,exporterProfile=pin,namespace='truss',identityProfile=pin,valueProfile=pin,keyProfile=pin,readContextProfile=pin,readContextDefinition=a),entities=[dict(logical=logical,typeId='-1',acceptedDefinition=a,source=a)],properties=[dict(logical={**logical,'element':'field'},ownerTypeId='-1',propertyId='0',home='props',homeProfile=pin,homeDefinition=artifact(home),valueProfile=pin,valueDefinition=a,presenceProfile=pin,presenceDefinition=a,acceptedDefinition=a,source=a)],keys=[],relationships=[],executionObligations=[],qualification=[])
(OUT/'binding-props.json').write_text(json.dumps(binding,indent=2)+'\n')
row=dict(interfaceVersion='truss-row-home/0.1.0',layoutInventory=a,recordKind='object',ownerCatalogId='-1',propertyCatalogId='0',stateRelationPhysicalIdentity='state',nodeRelationPhysicalIdentity='node',scalarRelationPhysicalIdentity='scalar',joinProfile=pin,joinDefinition=a,access='scalar-root',presenceDefinition=a,valueDefinition=a,storedDomainObligation='candidate-stored-domain')
binding['properties'][0].update(home='row',homeDefinition=artifact(row))
(OUT/'binding-row.json').write_text(json.dumps(binding,indent=2)+'\n')
