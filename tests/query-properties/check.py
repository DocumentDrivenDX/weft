"""Original source/AST correspondence; never native admission or execution."""
import copy, hashlib, json, os, pathlib, subprocess
root=pathlib.Path(__file__).resolve().parent
producer=os.environ['WEFT_QUERY_PROPERTIES']
legacy=os.environ['WEFT_SECURITY_MAPPING']
checks=[]
def run(binary,raw):return subprocess.run([binary],input=raw.encode(),capture_output=True)
def refused(raw):
 p=run(producer,raw);assert p.returncode!=0 and p.stdout==b'' and b'WFT-' in p.stderr
for name in ['operators','range','field-free-count']:
 raw=(root/'fixtures'/f'{name}.request.json').read_text();request=json.loads(raw)
 p=run(producer,raw);assert p.returncode==0 and not p.stderr
 assert p.stdout==(root/'fixtures'/f'{name}.report.json').read_bytes()
 report=json.loads(p.stdout);assert not report['authority'] and not report['securityReady']
 assert report['sql']==request['sql']
 for value,digest in [('sql','sqlSha256'),('sourceContextJson','sourceContextSha256')]:assert hashlib.sha256(report[value].encode()).hexdigest()==report[digest]
 source=json.loads(report['sourceContextJson'])
 for key in ['modules','policyJson','ontologyJson','queryProfileJson','bindingJson','backendId','backendVersion','targetProfile']:assert source[key]==request[key]
 for use in report['uses']:
  assert use['modeSource']=='legacy-ontology-query-profile'
  assert use['fieldIdentity']['revision']==request['modules'][0]['pin']['revision']
  assert use['mode']=='original-authorized' and use['originalAction']=='query-original'
 if name=='operators':
  expected={'joins/0/on/0/left':('s0','equality'),'joins/0/on/0/right':('s1','equality'),'filters/0/left':('s0','equality'),'groups/0':('s0','equality'),'order/0':('s0','order')}
  for path,(scan,prop) in expected.items():
   use=next(u for u in report['uses'] if u['path']==path);assert use['scan']==scan and use['required']==[prop]
  summed=next(u for u in report['uses'] if u['operation']=='sum');assert summed['required'] is None and summed['propertyStatus']=='unsupported' and summed['outputType']['family']=='integer'
 elif name=='range':
  assert [u['required'] for u in report['uses']]==[['order'],['equality'],['equality'],['order']]
  assert len({u['path'] for u in report['uses']})==4
 else:assert report['uses']==[] and list(report['scans'])==['s0']
 old=run(legacy,raw);assert old.returncode==0 and not old.stderr
 assert old.stdout==(root/'fixtures'/f'{name}.legacy.json').read_bytes()
 assert json.loads(run(legacy,p.stdout.decode()).stdout)['status']=='refused'
 for key in ['required','mode','originalAction','uses','authority']:
  bad=copy.deepcopy(request);bad[key]=[];refused(json.dumps(bad))
 bad=copy.deepcopy(request);bad['modules'][0]['pin']['sha256']='0'*64;refused(json.dumps(bad))
 checks.append(name+': exact source/hash/AST occurrence report; legacy bytes preserved; annotation and pin substitutions refused')
request=json.loads((root/'fixtures/operators.request.json').read_text())
for sql in ['SELECT COUNT(DISTINCT r.salary) FROM Resource r','SELECT COUNT(*) FROM Resource r LIMIT 0','SELECT COUNT(*) FROM Resource r WHERE FALSE']:
 request['sql']=sql;refused(json.dumps(request))
for raw in ['{}','[]','{"interfaceVersion":1,"interfaceVersion":2}',' ' *4_000_001]:refused(raw)
checks.append('unsupported distinct/zero-limit/constant-false and malformed/duplicate/oversized inputs refuse with no partial stdout')
print(json.dumps({'scope':'legacy source/AST requirements only; no catalog storage transform/current authority/native completeness or execution qualification','checks':checks},indent=2))
