"""Local native-backend support audit; never trusts a green summary alone.
This host-side verifier reads pinned test reports. It is not compiler IO.
"""
from collections import Counter
import hashlib,json,re
from pathlib import Path
class Refused(Exception):pass
def require(value):
 if not value:raise Refused('unverified support claim')
def canonical(value):return json.dumps(value,ensure_ascii=False,sort_keys=True,separators=(',',':'),allow_nan=False)
def strict(raw):
 def pairs(items):
  result={}
  for key,value in items:require(key not in result);result[key]=value
  return result
 def invalid(value):raise Refused('non-JSON numeric constant')
 return json.loads(raw,object_pairs_hook=pairs,parse_constant=invalid)
def audit(claim):
 try:
  require(set(claim)=={'status','profile','requiredLayers','cases','evidence'} and claim['status']=='supported')
  profile=claim['profile'];keys={'compilerVersion','dialect','irVersion','backendVersion','targetProfile','engineVersion','layoutRevision','modelSha256','bindingSha256','settings'}
  require(set(profile)==keys and all(isinstance(profile[k],str) and profile[k] for k in keys-{'settings'}))
  require(isinstance(profile['settings'],dict) and profile['settings'])
  require(all(re.fullmatch(r'[0-9a-f]{64}',profile[k]) for k in ['modelSha256','bindingSha256']))
  require(not any(word in profile['engineVersion'].lower() for word in ['unqualified','unknown','zero build']))
  layers=claim['requiredLayers'];require(isinstance(layers,list) and layers and len(set(layers))==len(layers) and set(layers)<={'native','python','browser','library'} and 'native' in layers)
  expected={}
  for case in claim['cases']:
   require(set(case)=={'id','comparison','expected'} and isinstance(case['id'],str) and case['id'] and case['id'] not in expected and isinstance(case['expected'],list))
   require(case['comparison'] in ['bag','ordered'])
   rows=[canonical(row) for row in case['expected']]
   expected[case['id']]=(case['comparison'], Counter(rows) if case['comparison']=='bag' else rows)
  require(expected and claim['evidence']);covered={layer:set() for layer in layers};seen=set()
  for ref in claim['evidence']:
   require(set(ref)=={'path','sha256'} and isinstance(ref['path'],str))
   raw=Path(ref['path']).read_bytes();require(hashlib.sha256(raw).hexdigest()==ref['sha256'])
   report=strict(raw.decode('utf8'));require(set(report)=={'status','layer','profile','cases'} and report['status']=='passed' and canonical(report['profile'])==canonical(profile))
   layer=report['layer'];require(layer in layers)
   for case in report['cases']:
    require(set(case)=={'id','status','actual'} and case['status']=='passed' and isinstance(case['actual'],list))
    id=case['id'];require(id in expected and (layer,id) not in seen);seen.add((layer,id))
    comparison,wanted=expected[id];rows=[canonical(row) for row in case['actual']]
    require((Counter(rows) if comparison=='bag' else rows)==wanted);covered[layer].add(id)
  require(all(ids==set(expected) for ids in covered.values()))
  return dict(status='verified',cases=len(expected),layers=sorted(layers))
 except Refused:raise
 except (OSError,ValueError,TypeError,KeyError,UnicodeError) as error:raise Refused('unavailable or malformed support evidence') from error
