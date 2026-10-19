# SPDX-License-Identifier: MIT OR Apache-2.0
"""Native original-producer object custody; no native source/caller authority.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import copy,hashlib,json,pathlib,pickle
import weft
root=pathlib.Path(__file__).resolve().parents[2]
cls=weft.ApplicationLineagePreparation
checks=[]
def refuses(label,call):
 try:call()
 except (TypeError,AttributeError,ValueError):checks.append({'id':label,'refused':True})
 else:raise AssertionError(label)
for folder in ['application-lineage','lineage-v02']:
 for p in sorted((root/'tests'/folder/'fixtures').glob('*.request.json')):
  expected=p.with_name(p.name.replace('.request.','.report.'))
  if not expected.exists():continue
  prepared=weft.prepare_application_lineage(p.read_text())
  assert type(prepared) is cls and prepared.json().encode()==expected.read_bytes()
  assert prepared.source_request_json()==p.read_text()
  assert prepared.json()==weft.prepare_application_lineage_json(p.read_text())
  r=json.loads(prepared.json());assert r['authority'] is False and r['securityReady'] is False
  checks.append({'id':folder+':'+p.stem,'reportSha256':hashlib.sha256(prepared.json().encode()).hexdigest(),'passed':True})
raw=(root/'tests/lineage-v02/fixtures/projection-read.request.json').read_text()
prepared=weft.prepare_application_lineage(raw);original=prepared.json()
for i,args in enumerate([(),(original,),({'json':original},)]):refuses('no-report-constructor:'+str(i),lambda args=args:cls(*args))
refuses('no-object-new',lambda:object.__new__(cls))
refuses('no-subclass',lambda:type('ForgedPreparation',(cls,),{}))
refuses('no-inner-assignment',lambda:setattr(prepared,'inner',original))
refuses('no-report-assignment',lambda:setattr(prepared,'json',lambda:'{}'))
refuses('no-dynamic-dictionary',lambda:getattr(prepared,'__dict__'))
refuses('no-copy',lambda:copy.copy(prepared))
refuses('no-deepcopy',lambda:copy.deepcopy(prepared))
refuses('no-pickle',lambda:pickle.dumps(prepared))
class Duck:
 def json(self):return original
for value in [original,json.loads(original),Duck(),object()]:refuses('native-receiver-refuses:'+type(value).__name__,lambda value=value:cls.json(value))
assert prepared.json()==original
spaced=' \n'+raw+'\t';held=weft.prepare_application_lineage(spaced);r=json.loads(held.json());assert r['sourceRequestJson']==spaced and held.source_request_json()==spaced
checks.append({'id':'exact-original-request-whitespace','passed':True})
# A caller can invoke the producer on arbitrary opaque source bytes. Issuance is no grant.
request=json.loads(raw);source=request['opaqueSources'][0];source['utf8']='{"uninterpreted":"caller content"}';source['sha256']=hashlib.sha256(source['utf8'].encode()).hexdigest()
r=json.loads(weft.prepare_application_lineage(json.dumps(request)).json())
assert r['authority'] is False and r['securityReady'] is False and r['unresolvedSources'][0]['status']=='uninterpreted'
checks.append({'id':'genuine-producer-does-not-authenticate-opaque-source','passed':True})
source['utf8']='opaque\r\n😀\0{"duplicate":1,"duplicate":2}';source['sha256']=hashlib.sha256(source['utf8'].encode()).hexdigest();raw_unicode=json.dumps(request,ensure_ascii=False)
r=json.loads(weft.prepare_application_lineage(raw_unicode).json());retained=json.loads(r['sourceRequestJson'])
assert r['sourceRequestJson']==raw_unicode and retained['opaqueSources'][0]['utf8']==source['utf8'] and r['authority'] is False
checks.append({'id':'unicode-nul-opaque-bytes-retained-without-interpretation','passed':True})

for i,bad in enumerate(['{}',' '*4_000_001]):refuses('bad-source:'+str(i),lambda bad=bad:weft.prepare_application_lineage(bad))
assert len({r['id'] for r in checks})==len(checks)
print(json.dumps({'scope':'Actual in-process original compiler output custody only; every source/caller/current native authority check remains mandatory','module':weft.__file__,'typeModule':cls.__module__,'checks':checks},indent=2))
