import ast, hashlib, importlib.util, json, os, stat, subprocess, sys, tempfile, unittest
from pathlib import Path
from unittest.mock import patch
FREEZE=Path('/private/tmp/weft-paths-keys-assembly-source-freeze-20261010-c')
REPO=Path('/private/tmp/ashlar-weft-distribution-d2d')
BASE='3d4dc41e0882d0911979b2c57d76d86f107ac7d2'
WORK=Path('/private/tmp/astra-paths-keys-assembly-review-source-20261010-c')
RESULT=Path('/private/tmp/astra-paths-keys-assembly-source-controls-20261010-c.json')
def digest(raw):return hashlib.sha256(raw).hexdigest()
def desc(path):
 raw=path.read_bytes();return dict(path=str(path),sha256=digest(raw),bytes=len(raw))
manifest=json.loads((FREEZE/'manifest.json').read_bytes());assert digest((FREEZE/'manifest.json').read_bytes())=='da609d5fddc10fdb6f21c8b642685e80a6f68581c946a20b0cb758768a8cc4ae'
WORK.mkdir(exist_ok=False);sources=[]
for d in manifest['files']:
 raw=(FREEZE/d['path']).read_bytes();assert digest(raw)==d['sha256'] and len(raw)==d['bytes'];assert (REPO/d['path']).read_bytes()==raw
 p=WORK/d['path'];p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw);sources.append(d)
old_names=['scripts/distribution/test_paths_assembly.py','scripts/distribution/assemble-distribution.py','tests/distribution-assembly/test_assembly.py']
for name in old_names:
 raw=subprocess.check_output(['git','show',BASE+':'+name],cwd=REPO);p=WORK/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw);sources.append(dict(path=name,sha256=digest(raw),bytes=len(raw),sourceCommit=BASE))
sys.path.insert(0,str(WORK/'scripts/distribution'))
spec=importlib.util.spec_from_file_location('reviewed_assembly',WORK/'scripts/distribution/assemble-paths-distribution.py');a=importlib.util.module_from_spec(spec);sys.modules[spec.name]=a;spec.loader.exec_module(a)
import paths_keys_distribution as k
controls=[]
def passed(name):controls.append(name)
old_raw=subprocess.check_output(['git','show',BASE+':scripts/distribution/assemble-paths-distribution.py'],cwd=REPO)
old_ast=ast.parse(old_raw);new_ast=ast.parse((WORK/'scripts/distribution/assemble-paths-distribution.py').read_bytes())
olddefs={n.name:n for n in old_ast.body if isinstance(n,(ast.FunctionDef,ast.ClassDef))};newdefs={n.name:n for n in new_ast.body if isinstance(n,(ast.FunctionDef,ast.ClassDef))}
for name in olddefs:
 if name not in ('Config','assemble','publish','main'):assert ast.dump(olddefs[name],include_attributes=False)==ast.dump(newdefs[name],include_attributes=False),name
new_assemble=newdefs['assemble'];assert isinstance(new_assemble.body[0],ast.If);new_assemble.body=new_assemble.body[1:]
assert ast.dump(olddefs['assemble'],include_attributes=False)==ast.dump(new_assemble,include_attributes=False)
passed('All original helper definitions unchanged except named Config/publish/main; original assemble body exact after new dispatch')
for executable in ('bin/weft-paths','bin/weft-paths-keys'):
 with tempfile.TemporaryDirectory() as d:
  root=Path(d);s=a.Snapshot();s.generated('bin/weft-paths',b'INERT OLD');s.generated('bin/weft-paths-keys',b'INERT NEW');s.generated('meta.json',b'{}');out=root/'out'
  a.publish(s,out,**({}if executable.endswith('/weft-paths')else dict(executable=executable)))
  for name,raw in s.artifacts.items():assert (out/name).read_bytes()==raw and stat.S_IMODE((out/name).stat().st_mode)==(0o555 if name==executable else 0o444)
passed('Both closed executable profiles preserve exact inert bytes and selected0555/others0444; old default retained')
with tempfile.TemporaryDirectory() as d:
 s=a.Snapshot()
 with patch.object(s,'close',side_effect=AssertionError('effects before validation')):
  try:a.publish(s,Path(d)/'out',executable='bin/untrusted')
  except ValueError as e:assert str(e)=='closed-executable-selection'
  else:raise AssertionError('invalid selector admitted')
 assert not list(Path(d).iterdir())
passed('Unknown executable refuses before any close/publication effect')
class ReadOnlyCancel(KeyboardInterrupt):
 def __setattr__(self,name,value):
  if name=='cleanup_failed':raise RuntimeError('readonly marker')
  return super().__setattr__(name,value)
for executable in ('bin/weft-paths','bin/weft-paths-keys'):
 for primary in (KeyboardInterrupt('primary'),ReadOnlyCancel('primary')):
  with tempfile.TemporaryDirectory() as d:
   root=Path(d);s=a.Snapshot();s.generated(executable,b'inert');closes=[]
   class Fake:
    def write(self,raw):raise primary
    def close(self):closes.append(True);raise OSError('close')
   with patch.object(Path,'open',return_value=Fake()):
    try:a.publish(s,root/'out',executable=executable)
    except BaseException as e:assert e is primary
    else:raise AssertionError('primary suppressed')
   assert closes==[True] and not list(root.iterdir())
passed('Both profiles preserve write-cancellation identity through close failure, including readonly-marker primary; no final/stage remains')
for cleanup in (SystemExit('close'),GeneratorExit('close')):
 class FakeClose:
  def write(self,raw):return len(raw)
  def flush(self):pass
  def fileno(self):return 123
  def close(self):raise cleanup
 with patch.object(Path,'open',return_value=FakeClose()),patch.object(a.os,'fsync'):
  try:a.write_exclusive(Path('/unused-inert-path'),b'inert')
  except BaseException as e:assert e is cleanup
  else:raise AssertionError('cleanup-only cancellation suppressed')
passed('Cleanup-only SystemExit and GeneratorExit identities preserved')
with tempfile.TemporaryDirectory() as d:
 s=a.Snapshot();s.generated('meta.json',b'candidate');out=Path(d)/'out';error=ValueError('closing drift')
 with patch.object(s,'close',side_effect=[None,error]):
  try:a.publish(s,out)
  except ValueError as e:assert e is error
  else:raise AssertionError('closing drift admitted')
 assert not out.exists() and not list(Path(d).iterdir())
passed('Final custody failure refuses availability and cleans staged files')
for value in ('.','..','_candidate','-candidate','ébad','a/b','a'*129):
 try:a.Config(Path('/private/tmp'),Path('/private/tmp'),Path('/private/tmp'),Path('/private/tmp/not-created'),value,'paths-keys')
 except ValueError:pass
 else:raise AssertionError('invalid realization '+value)
passed('Realization IDs align with bounded public schema ASCII-leading rule')
# Pure reads and the already-reviewed pure record verifier: no assemble_keys call.
build=Path('/private/tmp/weft-paths-keys-3a2a79c-build-20261010-a');corpus=Path('/private/tmp/weft-paths-keys-qualification-20261010-d')
inputs=[build/'source-inventory.json',build/'backend-manifest.json',build/'command-b.json',corpus/'producer-output/receipt.json',Path('/private/tmp/weft-paths-keys-candidate-corpus-20261010-a/candidate-cases.json')]
for p,pin in zip(inputs,[k.INVENTORY,k.BACKEND,k.BUILD,k.RECEIPT,k.CASES]):assert digest(p.read_bytes())==pin
inventory=a.document(inputs[0].read_bytes());backend=a.document(inputs[1].read_bytes());receipt=a.document(inputs[3].read_bytes());bundle=a.document(inputs[4].read_bytes());rows=k.verify_records(receipt,bundle,backend,a)
assert len(rows)==74 and len(backend['capabilities'])==48
passed('Exact accepted actual74 receipt/candidate/backend/inventory/build pins pass pure full-record correspondence; no execution')
indexed={d['path']:d for d in inventory['files']};source=build/'source';selected={}
def check_source(name):
 raw=a.read(source/name);d=indexed[name];assert digest(raw)==d['sha256'] and len(raw)==d['bytes'];assert hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==d['gitBlob'];assert bool((source/name).stat().st_mode&0o111)==(d['mode']=='100755');selected[name]=dict(sha256=digest(raw),bytes=len(raw));return raw
for d in receipt['sources']:
 raw=check_source(d['path']);assert digest(raw)==d['sha256'] and len(raw)==d['bytes']
history=bundle['historicalQualification'];base=Path(history['manifest']['path']).parent.as_posix();hm=a.document(check_source(history['manifest']['path']));hr=a.document(a.inflate(check_source(history['receipt']['path'])));hiname=base+'/'+hm['source']['inventory']['artifact']['path'];hiraw=a.inflate(check_source(hiname));hi=a.document(hiraw)
assert digest(hiraw)==hm['source']['inventory']['decodedSha256']==hr['sourceInventorySha256'];assert hi['sourceCommit']==hr['sourceCommit']==history['sourceCommit'];assert hm['realizationId']==history['realizationId'];old_index={d['path']:d for d in hi['files']}
for d in hr['sources']:
 raw=check_source(base+'/source-subset/'+d['path']);od=old_index[d['path']];assert digest(raw)==d['sha256']==od['sha256'];assert len(raw)==d['bytes']==od['bytes'];assert hashlib.sha1(b'blob '+str(len(raw)).encode()+b'\0'+raw).hexdigest()==od['gitBlob']
hp=base+'/source-subset/scripts/distribution/check-paths-cli.py';check_source(hp);a.verify_legacy(source/(base+'/source-subset'),source/hp,hr['cases'],hr['harnessSha256'])
assert len(hr['cases'])==512 and history['legacyCases']==463
schemas=[name for name in indexed if name.startswith('docs/helix/02-design/contracts/') and name.endswith('.schema.json')]
assert len(schemas)==20
for name in schemas:check_source(name)
passed('45 selected source descriptors, historical inventory/source/extractor463 original pairs, and20 closed public-schema bytes match Git3a inventory; historical530 retained separately')
commands=[]
env={'PATH':'/usr/bin:/bin','PYTHONDONTWRITEBYTECODE':'1'}
for executable,tail in [('/usr/bin/python3',['-B','-S','-W','error::ResourceWarning','-m','unittest','discover','-s','tests/distribution-paths-assembly','-q']),('/usr/bin/python3',['-B','-S','-W','error::ResourceWarning','-m','unittest','discover','-s','scripts/distribution','-p','test_paths_assembly.py','-q']),('/usr/bin/python3',['-B','-S','-W','error::ResourceWarning','-m','unittest','discover','-s','tests/distribution-assembly','-q']),('/private/tmp/ashlar-spark311/bin/python',['-B','-S','-W','error::ResourceWarning','-m','unittest','discover','-s','tests/distribution-paths-assembly','-q'])]:
 argv=[executable]+tail;r=subprocess.run(argv,cwd=WORK,env=env,stdout=subprocess.PIPE,stderr=subprocess.PIPE,timeout=90);commands.append(dict(argv=argv,cwd=str(WORK),environment=env,exitCode=r.returncode,stdout=r.stdout.decode('utf8'),stderr=r.stderr.decode('utf8')));assert r.returncode==0,commands[-1]
for d in manifest['files']:assert desc(FREEZE/d['path'])['sha256']==d['sha256'] and desc(REPO/d['path'])['sha256']==d['sha256']
output=dict(scope='Source-only independent tests and pure accepted proof correspondence. No real candidate assembly/compiler/Cargo/native/index/installation execution.',status='passed',freeze=desc(FREEZE/'manifest.json'),sources=sources,controls=controls,commands=commands,actualProofInputs=[desc(p)for p in inputs],selectedSourceCount=len(selected),selectedSourceBytes=sum(d['bytes']for d in selected.values()),script=desc(Path(__file__)),oldBaseCommit=BASE)
RESULT.write_text(json.dumps(output,indent=2,sort_keys=True)+'\n');print(json.dumps(dict(status='passed',controls=len(controls),suiteCounts=[9,12,14,9],result=str(RESULT),sha256=digest(RESULT.read_bytes()))))
