"""Actual issuer/source/action correspondence, never native authorization.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import hashlib,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parent;binary=pathlib.Path(os.environ['WEFT_CURRENT_DESIGNATION'])
expected={
'disclosed-token-equality':[('filters/0/left','s0',None)],
'disclosed-token-selfjoin':[('joins/0/on/0/left','s0',None),('joins/0/on/0/right','s1',None),('outputs/0/argument','s1',None)],
'original-token-order':[('order/0','s0','query-original-token-order')],
'read-dto-hidden-original':[('filters/0/left','s0','query-original-salary-predicate')],
'mixed-original-uses':[('filters/0/left','s0','query-original-salary-predicate'),('order/0','s0','query-original-token-order')],
'prohibited-salary-order-empty':[('order/0','s0',None)],
'unsupported-salary-sum':[('outputs/0/argument','s0',None)]}
sha=lambda raw:hashlib.sha256(raw).hexdigest();checks=[]
assert sha((root/'fixtures/original-owner-source.json').read_bytes())=='2f826621bd3187857f1b4fbab5d9c818891b0df96d55d6a1d379638906e2ebda'
for name,uses in expected.items():
 raw=(root/'fixtures'/f'{name}.request.json').read_bytes();p=subprocess.run([str(binary)],input=raw,capture_output=True);assert p.returncode==0 and not p.stderr and p.stdout==(root/'fixtures'/f'{name}.report.json').read_bytes(),name
 r=json.loads(p.stdout);request=json.loads(raw);profile=next(s for s in request['opaqueSources'] if s['role']=='query-designation')
 assert r['sourceRequestSha256']==sha(raw) and r['sourceProfileJson']==profile['utf8'] and r['sourceProfileSha256']==profile['sha256']==sha(profile['utf8'].encode())
 assert r['authority'] is False and r['securityReady'] is False
 assert [(u['path'],u['scan'],u['originalAction']) for u in r['uses']]==uses,name
 assert r['primaryAction']=='read' and r['originalActionRequiredPurpose']=='original-authorized'
 assert all(not(set(u)&{'mode','permitted','authorized','valueSource'}) for u in r['uses'])
 checks.append({'case':name,'reportSha256':sha(p.stdout),'passed':True})
request=json.loads((root/'fixtures/mixed-original-uses.request.json').read_text())
for action in ['read','reverse','update','vault-write','missing']:
 bad=json.loads(json.dumps(request));source=next(s for s in bad['opaqueSources'] if s['role']=='query-designation');profile=json.loads(source['utf8']);profile['bindings'][0]['originalAction']=action;source['utf8']=json.dumps(profile,separators=(',',':'));source['sha256']=sha(source['utf8'].encode());p=subprocess.run([str(binary)],input=json.dumps(bad).encode(),capture_output=True);assert p.returncode and not p.stdout and p.stderr.startswith(b'WFT-CURRENT-QUERY-DESIGNATION:')
 checks.append({'unrelatedAction':action,'refused':True})
print(json.dumps({'scope':'Bounded original source/purpose designation; joint modes/effective values/native authority unresolved','binarySha256':sha(binary.read_bytes()),'checks':checks},indent=2))
