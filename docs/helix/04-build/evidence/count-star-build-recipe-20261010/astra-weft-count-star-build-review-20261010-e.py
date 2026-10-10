import hashlib, importlib.util, json, os, pathlib, subprocess, sys, tempfile, time
from unittest.mock import patch
sys.dont_write_bytecode=True
ROOT=pathlib.Path('/private/tmp/weft-count-star-build-20261010-b'); REPO=pathlib.Path('/private/tmp/ashlar-weft-distribution-d2d')
def descriptor(p):
 p=pathlib.Path(p);h=hashlib.sha256();n=0
 with p.open('rb') as f:
  for b in iter(lambda:f.read(262144),b''):h.update(b);n+=len(b)
 return {'path':str(p),'bytes':n,'sha256':h.hexdigest()}
plan=json.loads((ROOT/'command-e.json').read_bytes());assert descriptor(ROOT/'command-e.json')['sha256']=='a73bf24915efbabea6dbf6f4454ad9c956cd58b722b745b16e67cd4eb0d916e3'
for d in plan['resources']:assert descriptor(d['path'])==d
freeze=pathlib.Path('/private/tmp/weft-count-star-parity-harness-freeze-20261010-c.json');fj=json.loads(freeze.read_bytes())
for d in fj['files']:assert descriptor(d['path'])==d
opening=[descriptor(d['path']) for d in fj['files']]
spec=importlib.util.spec_from_file_location('bounded_launcher',ROOT/'launch-reviewed-build.py');m=importlib.util.module_from_spec(spec);spec.loader.exec_module(m);m.verify(plan)
full=json.loads((ROOT/'source-inventory.json').read_bytes()); closure=json.loads((ROOT/'build-closure-inventory.json').read_bytes())
assert full['sourceCommit']==closure['sourceCommit']==plan['sourceCommit']
tree=subprocess.check_output(['/usr/bin/git','-C',str(REPO),'ls-tree','-rz',plan['sourceCommit']])
entries={}
for row in tree.split(b'\0'):
 if not row:continue
 left,path=row.split(b'\t',1);mode,typ,blob=left.decode().split();assert typ=='blob';entries[path.decode()]={'mode':mode,'gitBlob':blob}
assert len(entries)==len(full['files'])==2295
for d in full['files']:assert entries[d['path']]=={k:d[k] for k in ['mode','gitBlob']}
assert sum(d['bytes'] for d in full['files'])==plan['sourceBytes']==393962138
size_rows=subprocess.check_output(['/usr/bin/git','-C',str(REPO),'cat-file','--batch-check=%(objectname) %(objecttype) %(objectsize)'],input=''.join(d['gitBlob']+'\n' for d in full['files']).encode()).decode().splitlines()
for d,row in zip(full['files'],size_rows):
 blob,typ,n=row.split();assert blob==d['gitBlob'] and typ=='blob' and int(n)==d['bytes']
by={d['path']:d for d in full['files']}
assert len(closure['files'])==271 and sum(d['bytes'] for d in closure['files'])==79610566
for d in closure['files']:assert by[d['path']]==d
assert '.cargo/config.toml' in by and '.cargo/config.toml' in {d['path'] for d in closure['files']}
assert 'HOME' not in plan['environment'] and 'RUSTFLAGS' not in plan['environment']
assert plan['environment']['CARGO_BUILD_JOBS']=='1'
for c in plan['commands']:
 for key in ['retainedBinary','stdout','stderr']:
  if key in c:assert not pathlib.Path(c[key]).exists(),c[key]
 if 'cargo' in pathlib.Path(c['argv'][0]).name:
  assert '--offline' in c['argv'] and '--locked' in c['argv'] and '-j1' in c['argv']
controls=[]
def run(code,maximum=4096,timeout=2,env=None):
 with tempfile.TemporaryDirectory() as tmp:
  result=m.capture([sys.executable,'-c',code],tmp,env or {'PATH':'/usr/bin:/bin'},tmp+'/out',tmp+'/err',timeout,maximum)
  return result,pathlib.Path(tmp+'/out').read_bytes(),pathlib.Path(tmp+'/err').read_bytes()
with patch.dict(os.environ,{'ASTRA_PARENT_SECRET_MARKER':'never-copy'}):
 result,out,err=run('import os;print(os.getenv("ASTRA_PARENT_SECRET_MARKER","absent"))');assert out==b'absent\n' and not err
controls.append('Closed child environment excludes parent-only marker')
result,out,err=run('import os;os.write(1,b"x"*128);os.write(2,b"y"*128)',maximum=128);assert out==b'x'*128 and err==b'y'*128
controls.append('Both exact stream limits preserve bytes')
try:run('import os;os.write(2,b"x"*129)',maximum=128)
except ValueError as e:assert str(e)=='Build output bound exceeded'
else:raise AssertionError('unbounded stderr')
controls.append('One extra stderr byte refuses')
started=time.monotonic()
try:run('import os,time;os.close(1);os.close(2);time.sleep(10)',timeout=.15)
except TimeoutError:pass
else:raise AssertionError('live leader escaped deadline')
assert time.monotonic()-started<2;controls.append('Closed pipes do not let live leader escape deadline')
primary=KeyboardInterrupt('independent cancellation');original=m.selectors.DefaultSelector
class Interrupted:
 def __init__(self):self.delegate=original()
 def register(self,*a):return self.delegate.register(*a)
 def get_map(self):return self.delegate.get_map()
 def select(self,*a):raise primary
 def close(self):self.delegate.close();raise RuntimeError('cleanup')
with patch.object(m.selectors,'DefaultSelector',Interrupted):
 try:run('import time;time.sleep(10)')
 except KeyboardInterrupt as e:assert e is primary and e.cleanup_failed
 else:raise AssertionError('lost cancellation')
controls.append('Original cancellation survives cleanup failure')
with tempfile.TemporaryDirectory() as tmp:
 out=pathlib.Path(tmp)/'out';out.write_bytes(b'prior')
 with patch.object(m.subprocess,'Popen',side_effect=AssertionError('must not launch')):
  try:m.capture([],tmp,{},str(out),tmp+'/err',1,1)
  except FileExistsError:pass
  else:raise AssertionError('existing output accepted')
 assert out.read_bytes()==b'prior'
controls.append('Existing output refuses before launch')
# Author tests are inert I/O tests; no native module or compiler is imported.
test=REPO/'scripts/distribution/test_paths_embedding.py'
r=subprocess.run([sys.executable,'-B','-W','error::ResourceWarning',str(test)],capture_output=True)
assert r.returncode==0,r.stderr.decode()
(ROOT.parent/'astra-weft-count-star-build-review-20261010-e-tests.stdout.log').write_bytes(r.stdout)
(ROOT.parent/'astra-weft-count-star-build-review-20261010-e-tests.stderr.log').write_bytes(r.stderr)
assert opening==[descriptor(d['path']) for d in fj['files']];m.verify(plan)
receipt={'verdict':'approved-exact-command-e-bounded-component-build; harness-c-source-approved-only','command':descriptor(ROOT/'command-e.json'),'freeze':descriptor(freeze),'harnessSources':opening,'sourceCommit':plan['sourceCommit'],'completeGitTreeFiles':len(entries),'materializedFiles':len(closure['files']),'materializedBytes':79610566,'resourcesRehashed':len(plan['resources']),'independentInertControls':controls,'authorInertTests':4,'findings':[], 'correctedPriorFinding': 'Command D omitted .cargo/config.toml; E restores exact tracked wasm32 entropy configuration.','limits':['No Cargo/Rust/Weft compiler/native extension/browser execution performed.','Cached dependencies/target, full TypeScript loader dependency chain, SDK/dynamic libraries are not fully inventoried; no hermetic or deterministic-byte claim.','Build target must have sole cooperating owner through retained output copy/hash.','Actual fresh artifacts, six phase exit results, full emitted metadata schema, 85-case corpus and Python/browser parity remain future qualification.', 'The 74 historical cases qualify their original 0.4 route; eleven new 0.4.1 semantic cases do not qualify all 49 new-profile capabilities. Full 0.4.1 profile coverage and index admission remain separate required work.','Harness approval assumes its explicit files and owning bounded outer launcher will be pinned independently in an exact parity command.'],'script':descriptor(__file__)}
path=ROOT.parent/'astra-weft-count-star-build-review-20261010-e.json';path.write_text(json.dumps(receipt,indent=2,sort_keys=True)+'\n');print(json.dumps(descriptor(path)))
