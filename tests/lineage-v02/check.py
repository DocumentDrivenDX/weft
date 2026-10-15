"""Source-only candidate correspondence and read duties. No native authority.
@covers US-009-AC1 @covers US-009-AC2 @covers US-009-AC3
"""
import hashlib,json,os,pathlib,subprocess
root=pathlib.Path(__file__).resolve().parent
binary=pathlib.Path(os.environ['WEFT_APPLICATION_LINEAGE'])
expected={
 'projection-read':(1,[('s0','outputs/0/expression','token','projection')]),
 'hidden-read-false-zero':(1,[('s0','outputs/0/expression','resourceId','projection'),('s0','filters/1/left','token','predicate'),('s0','order/0','token','order')]),
 'distinct-selfjoin-false-zero':(2,[('s0','joins/0/on/0/left','token','join'),('s1','joins/0/on/0/right','token','join'),('s0','filters/1/left','token','predicate'),('s1','outputs/0/argument','token','aggregate')]),
 'field-free-false-zero':(1,[]),
 'distinct-required':(1,[('s0','outputs/0/argument','token','aggregate')]),
 'sum-unsupported':(1,[('s0','outputs/0/argument','salary','aggregate')]),
}
def invoke(raw):return subprocess.run([str(binary)],input=raw,capture_output=True)
rows=[]
for name,(scans,fields) in expected.items():
 path=root/'fixtures'/f'{name}.request.json';raw=path.read_bytes();p=invoke(raw)
 assert p.returncode==0 and not p.stderr and p.stdout==(root/'fixtures'/f'{name}.report.json').read_bytes(),name
 r=json.loads(p.stdout);assert r['authority'] is False and r['securityReady'] is False
 assert r['version']=='weft.application.lineage/0.2-candidate'
 assert r['sourceRequestJson'].encode()==raw and r['sourceRequestSha256']==hashlib.sha256(raw).hexdigest()
 plan=json.loads(r['applicationPlanJson']);assert plan['irVersion']=='weft-ir/application-lineage/0.2-candidate'
 reads=r['readRequirements'];assert len(reads['scans'])==scans
 assert [(f['scan'],f['path'],f['fieldIdentity']['element'],f['role']) for f in reads['fields']]==fields,name
 for f in reads['fields']:
  assert f['descriptor']['availability']=='required' and f['fieldIdentity']['revision']=='named-query-core-r1'
  assert not set(f)&{'mode','originalAction','action','authority'}
 assert all(s['kind']=='rowRead' and s['recordIdentity']['revision']=='named-query-core-r1' for s in reads['scans'])
 assert bool(reads['unsupportedSources'])==(name=='sum-unsupported')
 request=json.loads(raw)
 for key in ['readRequirements','applicationPlan','mode','originalAction']:
  bad=dict(request);bad[key]={};refused=invoke(json.dumps(bad).encode());assert refused.returncode and not refused.stdout and refused.stderr.startswith(b'WFT-')
 rows.append({'case':name,'requestSha256':hashlib.sha256(raw).hexdigest(),'reportSha256':hashlib.sha256(p.stdout).hexdigest(),'passed':True})
print(json.dumps({'scope':'Original compiler source-only correspondence; no native policy/caller/current authority','binarySha256':hashlib.sha256(binary.read_bytes()).hexdigest(),'cases':rows},indent=2))
