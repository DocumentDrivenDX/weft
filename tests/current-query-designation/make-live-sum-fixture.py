"""Author Weft's independent opt-in profile over the pinned original SUM source.
No action, route or capability is imported from owner intent/expected metadata.
"""
from pathlib import Path
import hashlib,json
root=Path(__file__).resolve().parent
source=(root/'fixtures/original-live-sum-owner-source.json').read_text()
assert hashlib.sha256(source.encode()).hexdigest()=='d9a87aa6b230eacf38b7b56451725cdf226a492610c2872c585008badd8c7b15'
# Slice exact authored role JSON bytes, preserving extension tokens/whitespace.
decoder=json.JSONDecoder();i=source.index('{')+1;slices={}
while True:
 while source[i].isspace() or source[i]==',':i+=1
 if source[i]=='}':break
 key,i=decoder.raw_decode(source,i)
 while source[i].isspace():i+=1
 assert source[i]==':';i+=1
 while source[i].isspace():i+=1
 start=i;_,i=decoder.raw_decode(source,i);assert key not in slices;slices[key]=source[start:i]
owner=json.loads(source);request=json.loads((root/'fixtures/mixed-original-uses.request.json').read_text())
strict=lambda text:json.loads(text,parse_int=lambda token:('integer-token',token),parse_float=lambda token:('number-token',token))
assert len(request['modules'])==1 and strict(request['modules'][0]['documentJson'])==strict(source)['closure']['documents'][0]['document']
assert owner['closure']['pins']==[{'documentId':'domain','revision':'query-value-core-r1'}]
sha=lambda text:hashlib.sha256(text.encode()).hexdigest()
opaque=[]
for role in ['policy','ontology','catalog','owner']:
 value=owner[role];identity={'documentId':value.get('documentId',value.get('id')),'revision':value['revision']}
 opaque.append({'role':role,'identity':identity,'mediaType':'application/json','utf8':slices[role],'sha256':sha(slices[role])})
ref=lambda element:{'documentId':'domain','moduleId':'m','elementId':element}
# Weft-owned declaration; separate from original policy permissions or author intent.
binding={'version':'weft.pg.current-read-binding/0.4-candidate','fixture':True,'scope':'declared original scalar stage; not installed'}
binding_raw=json.dumps(binding,separators=(',',':'));binding_identity={'documentId':'fixture-live-original-integer-sum-pg','revision':'sum-materialization-r1'}
opaque.append({'role':'native-binding','identity':binding_identity,'mediaType':'application/json','utf8':binding_raw,'sha256':sha(binding_raw)})
profile={'version':'weft.current-query.designation-profile/0.2-candidate','id':'weft-live-original-integer-sum-actions','revision':'live-original-integer-sum-designation-r1','modelPins':[m['pin'] for m in request['modules']],'sources':[{k:v for k,v in item.items() if k in ['role','identity','sha256']} for item in opaque if item['role']!='native-binding'],'binding':{'backendId':'weft.postgresql.current-read-candidate','backendVersion':'0.4-candidate','targetProfile':'pg18.6-current-sum/0.4-candidate','source':binding_identity,'sha256':sha(binding_raw)},'targets':[ref('Resource')],'primaryAction':'read','bindings':[{'target':ref('Resource'),'field':ref('salary'),'operator':'predicate','operation':'equal','originalAction':'query-original-salary-predicate'},{'target':ref('Resource'),'field':ref('token'),'operator':'order','operation':'order','originalAction':'query-original-token-order'},{'target':ref('Resource'),'field':ref('salary'),'operator':'aggregate','operation':'sum','originalAction':'query-original-salary-sum'}],'capabilities':['live-original-integer-sum']}
profile_raw=json.dumps(profile,separators=(',',':'))
opaque.append({'role':'query-designation','identity':{'documentId':profile['id'],'revision':profile['revision']},'mediaType':'application/json','utf8':profile_raw,'sha256':sha(profile_raw)})
request['opaqueSources']=opaque;request['sql']='SELECT SUM(r.salary) AS total FROM Resource r';request['parameters']={}
raw=json.dumps(request,separators=(',',':'),ensure_ascii=False)+'\n';(root/'fixtures/live-original-integer-sum.request.json').write_text(raw)
(root/'fixtures/live-original-integer-sum-source-pins.json').write_text(json.dumps({'originalOwnerCommit':'1bb8ec4768a720c88c4436cd2f1b75272661096c','originalSourceSha256':sha(source),'requestSha256':sha(raw),'roles':[{k:v for k,v in entry.items() if k!='utf8'} for entry in opaque],'qualification':'Source/profile custody only; no action permission/native installation or execution'},indent=2)+'\n')
