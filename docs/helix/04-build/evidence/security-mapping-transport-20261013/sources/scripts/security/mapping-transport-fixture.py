"""Authored transport fixture; no native model/profile admission."""
import json,hashlib,sys
from pathlib import Path
root=Path(__file__).resolve().parents[2]
def text(value):return json.dumps(value,separators=(',',':'),ensure_ascii=False)
def digest(value):return hashlib.sha256(value.encode()).hexdigest()
corpus=json.loads((root/'crates/weft-core/tests/security-evaluation-oracle.json').read_text())
case=next(c for c in corpus['cases'] if c['name']=='query-predicate-original-grant')
original=json.loads((root/'crates/weft-core/tests/security-source-fixture.json').read_text())
source=original['resolution']['documents'][0];document=source['document']
for field in document['modules'][0]['elements']:
 field['name']=field['id']
 if field.get('scalarType')=='integer':field['facets']={'integerWidth':{'bits':64,'signed':True}}
model=text(document);pin={'documentId':document['id'],'revision':source['revision'],'umfVersion':'0.8.0','sha256':digest(model)}
operators=['predicate','order','group','join','aggregate']
resource=next(e for e in case['ontology']['entities'] if e['type']['elementId']=='Resource')
salary=next(f for f in resource['fields'] if f['ref']['elementId']=='salary');salary['queryUse']={o:'original-authorized' for o in operators}
policy,ontology=text(case['policy']),text(case['ontology']);binding=text({'version':'fixture-only','layout':'opaque-unqualified-storage'})
profile={'version':'weft.security.query-profile/0.1.0','id':'fixture-profile','revision':'p1','action':'read','modelPins':[pin],
 'policySha256':digest(policy),'ontologySha256':digest(ontology),'binding':{'backendId':'fixture','backendVersion':'unqualified','targetProfile':'fixture-only','sha256':digest(binding)},
 'targets':case['request']['targets'],'bindings':[{'target':resource['type'],'field':salary['ref'],'operator':o,'originalAction':'query-original'} for o in operators]}
request={'interfaceVersion':'weft-security-mapping/0.1.0','modules':[{'documentJson':model,'pin':pin,'selectedModuleIds':['m']}],
 'policyJson':policy,'ontologyJson':ontology,'queryProfileJson':text(profile),'bindingJson':binding,'backendId':'fixture','backendVersion':'unqualified','targetProfile':'fixture-only',
 'sql':'SELECT r.salary AS amount,SUM(r.salary) AS total FROM Resource r JOIN Resource s ON r.salary=s.salary WHERE r.salary=100 GROUP BY r.salary ORDER BY r.salary','parameters':{}}
print(text(request))
