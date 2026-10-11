"""Author independent protocol migration and source-semantic checks, never compiler outputs."""
from pathlib import Path
import copy,gzip,json,hashlib
OLD=Path('/private/tmp/weft-042-integration-b/tests/distribution-count-star-corpus/fixtures/count-star-having-041-corpus.json.gz');OUT=Path('/private/tmp/weft-042-corpus-authored-a');OUT.mkdir(mode=0o700,exist_ok=True)
old=json.loads(gzip.decompress(OLD.read_bytes()));cases=[];ledger=[]
def request_migrate(raw,duplicate=False):
 if duplicate:
  assert raw.count(b'"interfaceVersion":"weft-compile/0.4.1"')==2
  return raw.replace(b'"interfaceVersion":"weft-compile/0.4.1"',b'"interfaceVersion":"weft-compile/0.4.2"').replace(b'"dialect":"weft-sql/0.4.1"',b'"dialect":"weft-sql/0.4.2"').replace(b'"backendVersion":"0.4.1-count-star-having-candidate"',b'"backendVersion":"0.4.2-implicit-arithmetic-labels-candidate"')
 q=json.loads(raw);m=copy.deepcopy(q)
 if m.get('interfaceVersion')=='weft-compile/0.4.1':m['interfaceVersion']='weft-compile/0.4.2'
 if m.get('dialect')=='weft-sql/0.4.1':m['dialect']='weft-sql/0.4.2'
 if m.get('target',{}).get('backendVersion')=='0.4.1-count-star-having-candidate':m['target']['backendVersion']='0.4.2-implicit-arithmetic-labels-candidate'
 for k in ('sql','modules'):assert m.get(k)==q.get(k)
 for k in ('bindingJson','bindingSha256'):assert m.get('target',{}).get(k)==q.get('target',{}).get(k)
 return json.dumps(m,separators=(',',':'),ensure_ascii=False).encode()
def expected_migrate(r,duplicate,identity):
 r=copy.deepcopy(r)
 assert r['interfaceVersion'] in ('weft-compile/0.4.0','weft-compile/0.4.1');r['interfaceVersion']='weft-compile/0.4.2'
 if identity in ('count-star:new-dialect-old-interface','count-star:old-language-new-backend'):
  r['diagnostics']=[{'code':'WFT-VERSION','message':'This entrypoint requires the exact 0.4.2 compile and dialect pair','phase':'input','recoverability':'correct-input','severity':'error'}]
 if r.get('dialect')=='weft-sql/0.4.1':r['dialect']='weft-sql/0.4.2'
 if 'backend'in r:assert r['backend']['backendVersion']=='0.4.1-count-star-having-candidate';r['backend']['backendVersion']='0.4.2-implicit-arithmetic-labels-candidate'
 if 'logicalPlan'in r:assert r['logicalPlan']['irVersion']=='weft-ir/0.4.1';r['logicalPlan']['irVersion']='weft-ir/0.4.2'
 for op in r.get('qualification',{}).get('operations',[]):
  for language in op['declaration']['languageProfiles']:
   assert language=={'dialectProfile':'weft-sql/0.4.1','irVersion':'weft-ir/0.4.1'};language.update(dialectProfile='weft-sql/0.4.2',irVersion='weft-ir/0.4.2')
 for d in r.get('diagnostics',[]):
  if d['message']=='Request is outside the closed 0.4.1 envelope':d['message']='Request is outside the closed 0.4.2 envelope'
  elif d['message']=='This entrypoint requires the exact 0.4.1 compile and dialect pair':d['message']='This entrypoint requires the exact 0.4.2 compile and dialect pair'
 return r
for c in old['cases']:
 duplicate=c['id']=='controls:duplicate-envelope-member';raw=bytes.fromhex(c['requestHex']);m=request_migrate(raw,duplicate);r=json.loads(bytes.fromhex(c['responseHex'])) if 'responseHex'in c else c['expectedResponse'];expected=expected_migrate(r,duplicate,c['id'])
 cases.append({'id':'inherited042:'+c['id'],'role':c['role'],'requestHex':m.hex(),'expectedResponse':expected,'expectation':'independent-historical-response-marker-migration'})
 ledger.append({'id':c['id'],'originalRequestSha256':hashlib.sha256(raw).hexdigest(),'newRequestSha256':hashlib.sha256(m).hexdigest(),'expectedException':('direct042 entrypoint fixes duplicate JSON refusal envelope to042 before selection' if duplicate else ('direct042 exact-version admission precedes syntax parsing for old/mixed language request' if c['id'] in ('count-star:new-dialect-old-interface','count-star:old-language-new-backend') else None))})
for name in ('partial-return','fulfillment','settlement','refund'):
 raw=(Path('/private/tmp/ashlar-commerce-original-installed-compile-c/compile-output')/('commerce-'+name+'-0.request')).read_bytes();q=request_migrate(raw)
 expected={'status':'compiled','outputNames':['quantity','quantity','l.quantity-f.quantity+r.quantity'] if name=='partial-return' else ['id'],'newCapability':name=='partial-return','originalSql':json.loads(raw)['sql'],'independentOriginalLexicalBag':{'partial-return':[['6','2','6']],'fulfillment':[['[42,0,"F2"]']],'settlement':[['[42,0,"PAY1"]']],'refund':[['[42,0,"RF1"]']]}[name]}
 if name=='partial-return':expected.update(expressionLabels=[{'position':2,'text':'l.quantity-f.quantity+r.quantity','sourceElements':['order_lines.quantity','fulfillments.quantity','returns.quantity'],'operators':['-','+']}])
 cases.append({'id':'original-commerce:'+name,'role':'original-semantic','requestHex':q.hex(),'expectedSemantics':expected,'expectation':'independent-source-semantic (bag is deferred native obligation, not compiler result)'})
base=json.loads(bytes.fromhex(cases[80]['requestHex']));tail=' FROM order_lines l JOIN fulfillments f ON f.line_id=l.id JOIN returns r ON r.line_id=l.id WHERE f.quantity<l.quantity';expr='l.quantity-f.quantity+r.quantity'
variants=[('parentheses','SELECT ('+expr+')'+tail,'('+expr+')',True),('internal-spaces','SELECT (l.quantity - f.quantity + r.quantity)'+tail,'(l.quantity - f.quantity + r.quantity)',True),('exterior-whitespace',' \n SELECT   '+expr+'   '+tail,expr,True),('unicode-quoted-alias','SELECT ("lé".quantity-f.quantity+r.quantity)'+tail.replace('order_lines l','order_lines "lé"').replace('l.id','"lé".id').replace('l.quantity','"lé".quantity'),'("lé".quantity-f.quantity+r.quantity)',True),('explicit-alias','SELECT '+expr+' AS remaining'+tail,'remaining',False),('duplicate-computed','SELECT '+expr+','+expr+tail,None,False),('explicit-collision','SELECT '+expr+', l.quantity AS "'+expr+'"'+tail,None,False),('comment-refusal','SELECT l.quantity/*comment*/-f.quantity+r.quantity'+tail,None,False)]
for name,sql,label,newcap in variants:
 q=copy.deepcopy(base);q['sql']=sql;e={'status':'compiled' if label else 'blocked','newCapability':newcap}
 if label:
  e['outputNames']=[label]
  if newcap:e['expressionLabels']=[{'position':0,'text':label,'sourceElements':['order_lines.quantity','fulfillments.quantity','returns.quantity'],'operators':['-','+']}]
 else:e['diagnosticCode']='WFT-OUTPUT-NAME' if name!='comment-refusal' else 'WFT-UNSUPPORTED'
 cases.append({'id':'labels:'+name,'role':'authored-semantic-control','requestHex':json.dumps(q,separators=(',',':'),ensure_ascii=False).encode().hex(),'expectedSemantics':e,'expectation':'independent-source-semantic'})
for name,iface,dialect in [('old041-pair','weft-compile/0.4.1','weft-sql/0.4.1'),('mixed042-041','weft-compile/0.4.2','weft-sql/0.4.1')]:
 q=copy.deepcopy(base);q.update(interfaceVersion=iface,dialect=dialect);cases.append({'id':'fence:'+name,'role':'authored-semantic-control','requestHex':json.dumps(q,separators=(',',':')).encode().hex(),'expectedSemantics':{'status':'blocked','diagnosticCode':'WFT-VERSION'},'expectation':'independent-version-fence'})
assert len(cases)==94 and len({c['id']for c in cases})==94
v={'sourceCommit':'f36f206eef70f69f2386a2acb466814d8302b016','historicalCorpusSha256':hashlib.sha256(OLD.read_bytes()).hexdigest(),'cases':cases,'inheritedCoverage':old['coverage'],'newCapabilityWitness':['original-commerce:partial-return','labels:parentheses','labels:internal-spaces','labels:exterior-whitespace','labels:unicode-quoted-alias'],'qualification':'Authored expected protocol/source semantics only; not executed/native/index. New cases do not use actualcompiler response as independentexpected; original lexical bags deferred to native host.'};raw=json.dumps(v,separators=(',',':'),ensure_ascii=False).encode();(OUT/'corpus.json.gz').write_bytes(gzip.compress(raw,mtime=0));(OUT/'migration-ledger.json').write_text(json.dumps(ledger,indent=2)+'\n');print(len(cases),len(raw),len((OUT/'corpus.json.gz').read_bytes()))
