# SPDX-License-Identifier: MIT OR Apache-2.0
"""Replay independent before-witness expectations against repaired source-only compiler."""
import copy,hashlib,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parent
binary=os.environ['WEFT_APPLICATION_LINEAGE']
observations=[]
def run(raw):return subprocess.run([binary],input=raw.encode(),capture_output=True)
before=json.loads((root/'expanded-before.json').read_text())
assert before['status']=='failed' and sum(r['passed'] for r in before['observations'])==2
for case in before['observations']:
 raw=json.dumps(case['request'],ensure_ascii=False,separators=(',',':'));p=run(raw)
 if case['expected']=='refused':assert p.returncode and not p.stdout and b'WFT-APPLICATION-LINEAGE-AVAILABILITY' in p.stderr,(case['id'],p.stdout,p.stderr)
 else:
  assert p.returncode==0 and not p.stderr,(case['id'],p.stderr)
  report=json.loads(p.stdout);assert report['sourceRequestJson']==raw and report['sourceRequestSha256']==hashlib.sha256(raw.encode()).hexdigest()
  assert report['securityReady'] is False and report['authority'] is False
  if case['id']=='field-free-count':assert report['uses']==[] and list(report['scans'])==['s0']
 observations.append({'id':case['id'],'expected':case['expected'],'passed':True,'stderr':p.stderr.decode()})
# An unselected optional member cannot blanket-refuse a distinct required projection.
request=copy.deepcopy(next(c['request'] for c in before['observations'] if c['id']=='field-free-count'));request['sql']='SELECT r.resourceId FROM Resource r'
p=run(json.dumps(request));assert p.returncode==0 and not p.stderr
report=json.loads(p.stdout);assert report['uses']==[] and list(report['scans'])==['s0']
observations.append({'id':'required-projection-with-unselected-optional-token','expected':'passed','passed':True})
legacy=os.environ['WEFT_QUERY_PROPERTIES']
raw=(root/'legacy-absent.request.json').read_text();before_legacy=json.loads((root/'legacy-absent.before.json').read_text());assert before_legacy['exit']==0 and before_legacy['stdout'] and not before_legacy['stderr']
p=subprocess.run([legacy],input=raw.encode(),capture_output=True);assert p.returncode and not p.stdout and b'WFT-APPLICATION-LINEAGE-AVAILABILITY' in p.stderr
observations.append({'id':'legacy-property-hidden-and-projected-absence','expected':'refused','passed':True})
print(json.dumps({'scope' :'source-only selected availability refusal and scan preservation; no native policy or execution acceptance','producerSha256':hashlib.sha256(pathlib.Path(binary).read_bytes()).hexdigest(),'observations':observations},indent=2))
