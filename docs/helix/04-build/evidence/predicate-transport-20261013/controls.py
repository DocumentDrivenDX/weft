from pathlib import Path
from hashlib import sha256
import copy,json,subprocess
out=Path(__file__).parent
binary=Path('/private/tmp/weft-042-release-target-a/debug/examples/security_predicate_spike')
root=Path('/private/tmp/truss-security-main-integration/docs/helix/04-build/evidence/security-predicate-native-20261013')
base={'request':json.loads((root/'request.json').read_bytes()),'assessment':json.loads((root/'assessment.json').read_bytes())}
pack=lambda v: json.dumps(v,separators=(',',':')).encode()
cases=[('validOriginalPacket',pack(base),True),('empty',b'',False),('malformed',b'{',False),('null',b'null',False),('array',b'[]',False),('boolean',b'true',False),('number',b'1',False),('emptyObject',b'{}',False),('invalidUtf8',b'\xff',False),('invalidStringSurrogate',b'"\\ud800"',False),('trailingGarbage',pack(base)+b'x',False),('duplicateOuter',b'{"request":{},"request":{},"assessment":{}}',False),('duplicateNested',b'{"request":{"modules":[],"modules":[]},"assessment":{}}',False)]
for name,value in [('requestArray',[]),('requestNull',None),('requestString','x'),('requestBoolean',True),('requestInteger',10)]:
 p=copy.deepcopy(base);p['request']=value;cases.append((name,pack(p),False))
for name,key,value in [('extraOuter','unexpected',None),('cutWithoutRequest','cut',{}),('requestWithoutCut','evaluationRequest',{})]:
 p=copy.deepcopy(base);p[key]=value;cases.append((name,pack(p),False))
for key in ['request','assessment']:
 p=copy.deepcopy(base);del p[key];cases.append(('missing_'+key,pack(p),False))
for key in ['modules','policyJson','ontologyJson']:
 p=copy.deepcopy(base);del p['request'][key];cases.append(('missingInner_'+key,pack(p),False))
for name,key,value in [('badModules','modules',{}),('badPolicyType','policyJson',{}),('badOntologyType','ontologyJson',False),('badPolicyText','policyJson','{'),('badOntologyText','ontologyJson','[]')]:
 p=copy.deepcopy(base);p['request'][key]=value;cases.append((name,pack(p),False))
for value in [None,[],{},'bad']:
 p=copy.deepcopy(base);p['assessment']=value;cases.append(('badAssessment_'+repr(value),pack(p),False))
p=copy.deepcopy(base);p.update(cut=None,evaluationRequest=None);cases.append(('nullSimulationPair',pack(p),False))
p=copy.deepcopy(base);p.update(cut={},evaluationRequest={});cases.append(('emptySimulationPair',pack(p),False))
p=copy.deepcopy(base);p['assessment']['futureNested']='['*200
# Deep actual JSON exceeds existing parser depth without attempting native execution.
cases.append(('depth200',b'{"request":'+b'['*200+b'0'+b']'*200+b',"assessment":{}}',False))
raw=pack(base);cases.append(('exact16MInput',raw+b' '*(16_000_000-len(raw)),True));cases.append(('over16MInput',raw+b' '*(16_000_001-len(raw)),False))
results=[]
for name,raw,positive in cases:
 r=subprocess.run([str(binary)],input=raw,capture_output=True,timeout=20)
 assert r.returncode==(0 if positive else 1),(name,r.returncode,r.stderr[:1000])
 if positive:
  value=json.loads(r.stdout);assert value['securityReady'] is False
  assert len(r.stdout.rstrip(b'\n'))<=16_000_000
  assert r.stderr==b''
  if name=='validOriginalPacket':baseline=r.stdout;(out/'baseline.json').write_bytes(r.stdout)
  else:assert baseline==r.stdout
 else:
  assert r.stdout==b'',(name,r.stdout[:100])
  value=json.loads(r.stderr);assert value['severity']=='error' and value['code'].startswith('WFT-')
  assert 'panicked' not in r.stderr.decode()
 results.append({'case':name,'inputBytes':len(raw),'passed':True,'exit':r.returncode,'diagnosticCode':None if positive else value['code']})
result={'scope':'Independent execution of current built diagnostic example; valid original retained packet and malformed transport controls, no native database calls','binary':str(binary),'binarySha256':sha256(binary.read_bytes()).hexdigest(),'passed':len(results),'cases':results}
(out/'results.json').write_text(json.dumps(result,indent=2)+'\n')
print(json.dumps({'passed':len(results),'path':str(out/'results.json')}))
