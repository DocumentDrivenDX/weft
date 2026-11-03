"""Closed 0.4.57 Mac controller-pair policy; never rewrites legacy /1 identities."""
from pathlib import Path
import hashlib,io,json,re,zipfile
from admit_current_sdk_0457 import _gate,VALIDATOR_SHA
need,closed,load_json,digest=(_gate[k] for k in ('need','closed','load_json','digest'))
R='c623b94b66b0b46d492d457c62faddc896e2e452'
RTREE='cc552d691c92c7abdd44fdf4e4ca585936ae68c5'
PROFILE='weft-current-macos-controller-pair/1'
MAC_PROFILE='weft-current-macos-controller-pair/1'
WORKFLOW='.github/workflows/release-macos-pair-0457.yml'
REGISTRY='scripts/release/reviewed-macos-pair-0457.json'
# Exact control/document paths only. All other tracked production bytes/modes
# (including every crate, dependency, feature, resource and packaging input)
# must be identical. No prefix/wildcard exemption.
ALLOWED_CHANGES={WORKFLOW,REGISTRY,
 'scripts/release/current_macos_pair_0457.py',
 'scripts/release/run_macos_pair_0457.py',
 'scripts/release/admit_controller_pairs_0457.py',
 'scripts/release/publish_macos_pair_0457.py',
 '.github/workflows/release.yml',
 'scripts/release/test_macos_pair_0457.py',
 'scripts/release/test_controller_pair_wiring_0457.py',
 'scripts/release/current_publication_0457.py',
 'docs/helix/05-deploy/releases/v0.4.57.md',
 'docs/helix/04-build/plans/current-0457-macos-controller-pair.md'}
MAC_ALLOWED_CHANGES=frozenset(ALLOWED_CHANGES)
MAC_CONTROLLER_FILES=frozenset(p for p in MAC_ALLOWED_CHANGES if p.endswith('.py') and not p.endswith('/test_controller_pair_wiring_0457.py'))
PAIR_FIELDS={'profile','status','sourceCommit','sourceTree','controllerCommit','controllerTree','workflowPath','workflowSha256','controllerHashes','changedPaths','controllerRef'}
def pair(value):
 closed(value,PAIR_FIELDS,'reviewed Mac pair')
 need(value['profile']==PROFILE and value['status']=='reviewed','Exact pair is not registered/reviewed')
 need(value['sourceCommit']==R and value['sourceTree']==RTREE,'Pair product R/tree differs')
 for k in ['controllerCommit','controllerTree']:need(type(value[k]) is str and re.fullmatch('[0-9a-f]{40}',value[k]) is not None,'Malformed '+k)
 need(value['controllerCommit']!=R and value['controllerTree']!=RTREE,'Controller/source identity collapsed')
 need(value['workflowPath']==WORKFLOW and value['controllerRef']=='codex/weft-0457-macos-controller','Closed Mac controller route differs')
 need(re.fullmatch('[0-9a-f]{64}',value['workflowSha256']) is not None,'Workflow bytes absent')
 need(type(value['controllerHashes']) is dict and set(value['controllerHashes'])==MAC_CONTROLLER_FILES,'Controller file inventory differs')
 need(all(re.fullmatch('[0-9a-f]{64}',h) is not None for h in value['controllerHashes'].values()),'Controller byte hash malformed')
 rows=value['changedPaths'];need(type(rows) is list and bool(rows),'Changed path proof absent')
 names=[]
 for row in rows:
  closed(row,{'path','before','after'},'changed path');names.append(row['path'])
  need(row['path'] in MAC_ALLOWED_CHANGES,'Undeclared controller/source difference')
  for k in ['before','after']:
   v=row[k]
   if v is not None:
    closed(v,{'mode','sha'},'changed Git entry');need(v['mode'] in ['100644','100755'] and re.fullmatch('[0-9a-f]{40}',v['sha']) is not None,'Unsafe changed Git entry')
  need(row['after'] is not None and row['before']!=row['after'],'Deleted or unchanged control path')
 need(names==sorted(set(names)) and WORKFLOW in names,'Duplicate/noncanonical changed path inventory')
 need('scripts/release/verify_current_sdk_0457.py' not in names,'Frozen /1 validator changed')
 return value

def tree_rows(response,expected):
 need(response.get('sha')==expected and response.get('truncated') is False,'Complete actual Git tree required')
 rows=response.get('tree');need(type(rows) is list,'Git tree absent');out={}
 for row in rows:
  p=row.get('path');need(type(p) is str and p not in out and not p.startswith('/') and all(x not in ('','.','..') for x in p.split('/')),'Unsafe/duplicate Git tree path')
  kind=row.get('type');mode=row.get('mode');oid=row.get('sha')
  need(type(oid) is str and re.fullmatch('[0-9a-f]{40}',oid) is not None,'Malformed Git object identity')
  need((kind=='tree' and mode=='040000') or (kind=='blob' and mode in ['100644','100755']),'Malformed/unsupported Git entry type and mode')
  out[p]=dict(type=kind,mode=mode,sha=oid)
 # Both full inputs are validated before directory/unchanged filtering. Keep
 # authenticated leaf types in the comparison and inventory proof.
 return {p:v for p,v in out.items() if v['type']!='tree'}
def verify_closure(registration,source_tree,controller_tree):
 return closed_closure(registration,source_tree,controller_tree,'aarch64-apple-darwin')

def validate_payload(target,files):
 """Same closed42/raw archive checks, with no fabricated /1 envelope."""
 need(target in _gate['TARGETS'],'Wrong platform');tag,machine=_gate['TARGETS'][target];name=f'umf_weft-0.4.57-cp39-abi3-{tag}.whl'
 need(name in files and all(n==name for n in files if n.endswith('.whl')),'Exact platform wheel absent/duplicated')
 raw=files[name];report=load_json(files['sdk-smoke.json']);packaging=load_json(files['packaging.json'])
 closed(report,_gate['SDK_FIELDS'],'SDK report');need(report['sourceCommit']==R and report['version']=='0.4.57' and report['wheel']==name,'SDK product identity differs')
 need(type(report['controls']) is int and report['controls']==42 and report['checks']==[dict(id=n,status=s) for n,s in _gate['EXPECTED']],'Closed ordered SDK42 differs')
 need(report['nativeAdmission'] is False and report['securityReady'] is False and report['temporalQualification']=='held','SDK authority promoted')
 closed(packaging,{'status','version','wheel','wheelSha256','nativeModuleSha256','python','scope'},'packaging')
 need(packaging['status']=='passed' and packaging['version']=='0.4.57' and packaging['wheel']==name and type(packaging['python']) is str and bool(packaging['python']),'Installed packaging identity failed')
 need(packaging['scope']=='exact installed archive, RECORD, metadata, ABI tag and notices; native semantics separately gated','Packaging scope differs')
 with zipfile.ZipFile(io.BytesIO(raw)) as z:
  names=z.namelist();need(len(names)==len(set(names)) and all(not n.startswith('/') and '..' not in Path(n).parts and '\\' not in n for n in names),'Unsafe wheel inventory')
  native=[n for n in names if n.endswith(('.so','.pyd','.dylib'))];need(len(native)==1,'Native module inventory differs');module=z.read(native[0]);shim=z.read('weft/__init__.py')
  if target=='aarch64-apple-darwin':need(module[:4]==b'\xcf\xfa\xed\xfe' and int.from_bytes(module[4:8],'little')==machine,'Mach-O ARM64 differs')
  else:need(module[:4]==b'\x7fELF' and module[4:6]==b'\x02\x01' and int.from_bytes(module[18:20],'little')==machine,'ELF architecture differs')
 pins=dict(wheelSha256=digest(raw),nativeModuleSha256=digest(module),shimSha256=digest(shim))
 closed(report['pinsBeforeAfter'],set(pins),'SDK actual pins');need(report['pinsBeforeAfter']==pins and packaging['wheelSha256']==pins['wheelSha256'] and packaging['nativeModuleSha256']==pins['nativeModuleSha256'],'Actual wheel/native/shim binding differs')
 return dict(target=target,wheel=name,**pins,sdkReportSha256=digest(files['sdk-smoke.json']),packagingReportSha256=digest(files['packaging.json']))

def validate_parity(files,wheel):
 from finalize_hosted_macos import CASE_IDS,validate_refusal_rows
 cases=load_json(files['compiler-cases.json'])
 # Independent immutable R corpus, not caller-controlled case/receipt hashes.
 # Q/A closure forbids changes to this source; the literal SHA also validates
 # the local bytes before parsing, including in pure/offline validation.
 corpus=(Path(__file__).resolve().parents[2]/'tests/compile/fixtures/cases.json').read_bytes()
 need(digest(corpus)=='d0fe7b7592ab21d105cbf4e2a3f68e22e97a77e507c70395daa35dd899dd7851','Independent R corpus bytes differ')
 original=load_json(corpus);by_id={row['id']:row for row in original}
 need(len(by_id)==len(original) and all(i in by_id for i in CASE_IDS),'Independent R case inventory differs')
 canonical=lambda value:json.dumps(value,ensure_ascii=False,separators=(',',':')).encode()
 need(canonical(cases)==canonical([by_id[i] for i in CASE_IDS]),'Selected requests differ from independently authenticated immutable R corpus')
 roster=load_json(files['compiler-roster.json']);rust=load_json(files['compiler-reports.json']);py=load_json(files['python-compiler-parity.json']);browser=load_json(files['browser-compiler-parity.json']);receipts=load_json(files['browser-compiler-receipts.json'])
 need([v.get('id') for v in cases]==CASE_IDS and roster.get('cases')==CASE_IDS and roster.get('profile')=='nine-frozen-refusal-responses/1' and roster.get('selectedCasesSha256')==digest(files['compiler-cases.json']),'Frozen9 roster/custody differs')
 need(all(v.get('expected',{}).get('status')=='blocked' for v in cases),'Non-refusal case');validate_refusal_rows(cases,rust)
 need(py.get('status')=='passed' and py.get('distribution')=='umf-weft' and py.get('distributionVersion')=='0.4.57' and py.get('nativeModuleSha256')==wheel['nativeModuleSha256'] and [x.get('id') for x in py.get('cases',[])]==CASE_IDS,'Installed Python9 parity differs')
 need(browser.get('byteParity') is True and browser.get('cases')==9,'Chromium9 parity differs')
 need(type(receipts) is dict and receipts.get('summary')==browser and type(receipts.get('cases')) is list and len(receipts['cases'])==9,'Chromium byte receipts absent/incomplete')
 for case,result,row in zip(cases,rust,receipts['cases'],strict=True):
  closed(row,{'id','requestSha256','actualSha256','expectedSha256'},'Chromium case receipt')
  request=json.dumps(case['request'],ensure_ascii=False,separators=(',',':')).encode()
  need(row['id']==case['id'] and row['requestSha256']==digest(request) and row['actualSha256']==row['expectedSha256']==digest(result['raw'].encode()),'Chromium9 request/response bytes differ')
 return dict(profile='nine-frozen-refusal-responses-rust-python-chromium/1',cases=CASE_IDS,rust='passed',installedPython='passed',chromiumWasm='passed')

# Separate enrollment, never an extension of the Mac-only registration.
ARM_PROFILE='weft-current-arm64-controller-pair/1'
ARM_WORKFLOW='.github/workflows/release-arm64-pair-0457.yml'
COHORT_PROFILE='weft-current-three-wheel-sdk-controller-pairs/1'
ALLOWED_CHANGES.update({ARM_WORKFLOW,'scripts/release/run_arm64_pair_0457.py',
 'scripts/release/admit_controller_pairs_0457.py',
 '.github/workflows/release.yml'})

ARM_ALLOWED_CHANGES=frozenset(ALLOWED_CHANGES)
ARM_CONTROLLER_FILES=frozenset(p for p in ARM_ALLOWED_CHANGES if p.endswith('.py') and not p.endswith('/test_controller_pair_wiring_0457.py'))

def arm_pair(value):
 # Share field/type/closure machinery, not identities or a normalized /1 receipt.
 closed(value,PAIR_FIELDS,'reviewed ARM pair')
 need(value['profile']==ARM_PROFILE and value['status']=='reviewed','ARM pair is not separately registered')
 need(value['sourceCommit']==R and value['sourceTree']==RTREE,'ARM product R/tree differs')
 need(value['workflowPath']==ARM_WORKFLOW and value['controllerRef']=='codex/weft-0457-arm64-controller','Closed ARM controller route differs')
 for k in ['controllerCommit','controllerTree']:need(type(value[k]) is str and re.fullmatch('[0-9a-f]{40}',value[k]) is not None,'Malformed ARM '+k)
 need(value['controllerCommit']!=R and value['controllerTree']!=RTREE,'ARM controller/source collapsed')
 need(re.fullmatch('[0-9a-f]{64}',value['workflowSha256']) is not None,'ARM workflow bytes absent')
 need(type(value['controllerHashes']) is dict and set(value['controllerHashes'])==ARM_CONTROLLER_FILES and all(re.fullmatch('[0-9a-f]{64}',h) is not None for h in value['controllerHashes'].values()),'ARM controller inventory differs')
 names=[]
 for row in value['changedPaths']:
  closed(row,{'path','before','after'},'ARM changed path');names.append(row['path']);need(row['path'] in ALLOWED_CHANGES,'Undeclared ARM source difference')
  for entry in [row['before'],row['after']]:
   if entry is not None:
    closed(entry,{'mode','sha'},'ARM Git entry');need(entry['mode'] in ['100644','100755'] and re.fullmatch('[0-9a-f]{40}',entry['sha']) is not None,'Unsafe ARM Git entry')
  need(row['after'] is not None and row['before']!=row['after'],'Deleted/unchanged ARM path')
 need(names==sorted(set(names)) and ARM_WORKFLOW in names,'ARM changed roster differs')
 return value

def selected_pair(value,target):
 if target=='aarch64-apple-darwin':return pair(value)
 if target=='aarch64-unknown-linux-gnu':return arm_pair(value)
 raise ValueError('No controller-pair route for target')

def closed_closure(registration,source_tree,controller_tree,target):
 p=selected_pair(registration,target);a=tree_rows(source_tree,RTREE);b=tree_rows(controller_tree,p['controllerTree'])
 typed=[dict(path=n,before=a.get(n),after=b.get(n)) for n in sorted(a.keys()|b.keys()) if a.get(n)!=b.get(n)]
 entry=lambda value:None if value is None else {k:value[k] for k in ['mode','sha']}
 rows=[dict(path=r['path'],before=entry(r['before']),after=entry(r['after'])) for r in typed]
 need(rows==p['changedPaths'],'Complete R-to-Q path/byte/mode inventory differs')
 return dict(sourceCommit=R,sourceTree=RTREE,controllerCommit=p['controllerCommit'],controllerTree=p['controllerTree'],target=target,changedPaths=rows,validatedTypedChanges=typed,sourceTypedInventorySha256=digest(json.dumps(a,sort_keys=True,separators=(',',':')).encode()),controllerTypedInventorySha256=digest(json.dumps(b,sort_keys=True,separators=(',',':')).encode()),productionInputsUnchanged=True)

def validate_component(component,files,registration,target,run_id,attempt):
 p=selected_pair(registration,target);need(type(attempt) is int and attempt>0,'Invalid original attempt')
 need(component.get('profile')==p['profile'] and component.get('status')=='passed','Incomplete/wrong controller-pair component')
 closed(component,{'profile','status','sourceCommit','sourceTree','controllerCommit','controllerTree','workflowCommit','producerRunId','producerRunAttempt','producerJobName','target','cargoLockSha256','canonicalUmf','nativeAdmission','publicationAdmission','sdkStages','wheel','wheelSha256','nativeModuleSha256','shimSha256','sdkReportSha256','packagingReportSha256','parity'},'pair component')
 need(component.get('sourceCommit')==R and component.get('sourceTree')==RTREE and component.get('controllerCommit')==component.get('workflowCommit')==p['controllerCommit'] and component.get('controllerTree')==p['controllerTree'],'Truthful product/controller identity differs')
 need(component.get('producerRunId')==run_id and component.get('producerRunAttempt')==attempt and component.get('producerJobName')==('macos-pair' if target=='aarch64-apple-darwin' else 'arm64-pair'),'Component original job/attempt differs')
 need(component.get('target')==target and component.get('nativeAdmission') is False and component.get('publicationAdmission') is False,'Target/authority differs')
 need(component.get('cargoLockSha256')=='92974ab7df6beb8033fba4d3b535fee1f694229e0a6918fbd3d059d65162121d' and component.get('canonicalUmf')=='e3f38ef6f05ae636d2837f9d69f8d6805222664b','Canonical lock/source differs')
 names=['venv','normal-install','pip-check','packaging','native-origin','temporal14','routing5','sdk42']
 stages=component.get('sdkStages',[]);need([s.get('name') for s in stages]==names and all(type(s.get('exitCode')) is int and s['exitCode']==0 for s in stages),'Installed/temporal/routing/SDK stages incomplete')
 wheel=validate_payload(target,files)
 for k,v in wheel.items():need(component.get(k)==v,'Component actual byte binding differs: '+k)
 if target=='aarch64-apple-darwin':
  original=load_json(files['packaging-macos-arm64.json'])
  need(original.get('status')=='passed' and original.get('version')=='0.4.57' and original.get('distribution')=='umf-weft' and original.get('sourceCommit')==R and original.get('target')==target and original.get('wheel')==wheel['wheel'] and original.get('wheelSha256')==wheel['wheelSha256'] and original.get('nativeModuleSha256')==wheel['nativeModuleSha256'],'Original Mac installed/producer packaging differs')
  parity=validate_parity(files,wheel);need(component.get('parity')==parity,'Mac9 parity incomplete')
 else:need(component.get('parity')=={'scope':'original ARM platform has no separate nine-refusal Mac parity; shared raw temporal Node/Chromium proof remains separately required'},'ARM parity scope differs')
 return dict(profile=p['profile'],sourceCommit=R,sourceTree=RTREE,controllerCommit=p['controllerCommit'],controllerTree=p['controllerTree'],producerRunId=run_id,producerRunAttempt=attempt,**wheel,nativeAdmission=False,publicationAdmission=False)
