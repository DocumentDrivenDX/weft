"""Closed current 0.4.57 three-wheel SDK inventory; no native execution grant.

Separate from verify-sdk-smoke.py: its historical five-wheel/26-case contract
is unchanged. Reports bind actual final-main R and actual producer/controller IDs.
"""
from pathlib import Path
import hashlib,io,json,re,zipfile

PROFILE='weft-current-three-wheel-sdk/1'
VERSION='0.4.57'
REPOSITORY='DocumentDrivenDX/weft'
TARGETS={
 'x86_64-unknown-linux-gnu':('manylinux_2_17_x86_64.manylinux2014_x86_64',62),
 'aarch64-unknown-linux-gnu':('manylinux_2_17_aarch64.manylinux2014_aarch64',183),
 'aarch64-apple-darwin':('macosx_11_0_arm64',0x100000c),
}
EXPECTED=[(n,s) for n,s in [
 ('read','compiled'),('read-check','compiled'),('page','compiled'),('page-check','compiled'),('export','compiled'),
 ('read-context-drift','blocked'),('page-context-drift','blocked'),('export-context-drift','blocked'),('export-program-drift','blocked'),
 ('distinct','compiled'),('distinct-null','compiled'),('not-distinct','compiled'),('not-distinct-null','compiled'),('distinct-carrier-mismatch','blocked')]]
for compiler in ['ashlar-spark4','ashlar-dbsql-20261016']:
 EXPECTED += [(compiler+'-'+n,s) for n,s in [
  ('read','compiled'),('read-check','compiled'),('page-unsupported-2.2','blocked'),
  ('current-profile-read','compiled'),('current-profile-read-check','compiled'),
  ('page','compiled'),('page-check','compiled'),('page-continuation','compiled'),
  ('page-sql-drift','blocked'),('page-hidden-key-drift','blocked'),('page-context-drift','blocked'),('page-binding-drift','blocked')]]
EXPECTED += [(n,'passed') for n in ['presence-issued-class','presence-model-requirements','presence-graph-requirements','presence-postgresql-lower-check']]
SDK_FIELDS={'version','sourceCommit','wheel','pinsBeforeAfter','controls','checks','nativeAdmission','securityReady','temporalQualification'}
ENVELOPE_FIELDS={'profile','sourceCommit','sourceTree','distribution','version','target','wheel','wheelSha256','nativeModuleSha256','shimSha256','sdkReportSha256','packagingReportSha256','controllerCodeSha256','producer','controller'}
IDENTITY_FIELDS={'repository','runId','workflowPath','sourceCommit','workflowCommit','headBranch','event','conclusion'}

class InvalidCurrentProfile(ValueError):pass
def need(condition,message):
 if not condition:raise InvalidCurrentProfile(message)
def closed(value,fields,where):need(type(value) is dict and set(value)==fields,where+' fields differ')
def digest(raw):return hashlib.sha256(raw).hexdigest()
def load_json(raw):
 def pairs(rows):
  out={}
  for k,v in rows:
   need(k not in out,'duplicate JSON key: '+k);out[k]=v
  return out
 def constant(value):raise InvalidCurrentProfile('non-JSON constant: '+value)
 return json.loads(raw,object_pairs_hook=pairs,parse_constant=constant)
def identity(value,source,role,workflow):
 closed(value,IDENTITY_FIELDS,role)
 need(value['repository']==REPOSITORY and value['workflowPath']==workflow,role+' repository/workflow differs')
 need(type(value['runId']) is int and value['runId']>0,role+' run ID invalid')
 need(value['sourceCommit']==value['workflowCommit']==source,role+' source/workflow is not final R')
 need(value['headBranch']=='main' and value['event']=='workflow_dispatch' and value['conclusion']=='success',role+' run was not successful exact-main dispatch')

def validate(rows,source,tree,controller_code_sha):
 """rows: exactly3 (envelope, raw SDK report, raw wheel, raw packaging report).

 controller_code_sha must be derived from the controller script bytes in frozen
 R, not from a caller envelope. Producer/controller envelopes are assembled only after actual GitHub run and
 artifact admission. This pure validator does not turn caller JSON into an API
 observation or replace the existing hosted producer/publisher validators.
 """
 need(re.fullmatch('[0-9a-f]{40}',source) is not None and re.fullmatch('[0-9a-f]{40}',tree) is not None,'R commit/tree malformed')
 need(type(controller_code_sha) is str and re.fullmatch('[0-9a-f]{64}',controller_code_sha) is not None,'Controller code hash from frozen R is malformed')
 need(type(rows) is list and len(rows)==3,'exactly three current wheels required')
 seen=set();linux_runs=set();mac_runs=set();controller_runs=set();out=[]
 for envelope,report_raw,wheel_raw,packaging_raw in rows:
  closed(envelope,ENVELOPE_FIELDS,'current envelope')
  e=envelope;target=e['target'];need(target in TARGETS and target not in seen,'target missing/duplicate/unsupported');seen.add(target)
  need(e['profile']==PROFILE and e['sourceCommit']==source and e['sourceTree']==tree,'current profile is not exact final R')
  need(e['distribution']=='umf-weft' and e['version']==VERSION,'distribution/version differs')
  tag,machine=TARGETS[target];name=f'umf_weft-{VERSION}-cp39-abi3-{tag}.whl'
  need(e['wheel']==name and e['wheelSha256']==digest(wheel_raw),'actual wheel name/bytes differ')
  workflow='.github/workflows/release-hosted-macos.yml' if target=='aarch64-apple-darwin' else '.github/workflows/release.yml'
  identity(e['producer'],source,'producer',workflow)
  identity(e['controller'],source,'controller',workflow)
  need(e['controller']['runId']==e['producer']['runId'],'in-producer SDK controller run differs')
  need(e['controllerCodeSha256']==controller_code_sha,'SDK controller implementation differs')
  (mac_runs if target=='aarch64-apple-darwin' else linux_runs).add(e['producer']['runId']);controller_runs.add(e['controller']['runId'])
  need(e['sdkReportSha256']==digest(report_raw),'SDK report bytes differ')
  need(e['packagingReportSha256']==digest(packaging_raw),'fresh packaging report bytes differ')
  packaging=load_json(packaging_raw)
  closed(packaging,{'status','version','wheel','wheelSha256','nativeModuleSha256','python','scope'},'fresh installed packaging')
  need(packaging['status']=='passed' and packaging['version']==VERSION and packaging['wheel']==name,'fresh installed packaging failed/differs')
  need(packaging['wheelSha256']==e['wheelSha256'] and packaging['nativeModuleSha256']==e['nativeModuleSha256'],'fresh installed archive/module differs')
  need(type(packaging['python']) is str and bool(packaging['python']),'actual Python setting absent')
  need(packaging['scope']=='exact installed archive, RECORD, metadata, ABI tag and notices; native semantics separately gated','packaging scope differs')
  report=load_json(report_raw);closed(report,SDK_FIELDS,'SDK report')
  need(report['sourceCommit']==source and report['version']==VERSION and report['wheel']==name,'SDK source/wheel/version differs')
  need(type(report['controls']) is int and report['controls']==42,'SDK control count differs')
  need(report['checks']==[dict(id=n,status=s) for n,s in EXPECTED],'closed ordered42 SDK inventory differs')
  need(report['nativeAdmission'] is False and report['securityReady'] is False and report['temporalQualification']=='held','SDK authority/scope differs')
  closed(report['pinsBeforeAfter'],{'wheelSha256','nativeModuleSha256','shimSha256'},'SDK pins')
  with zipfile.ZipFile(io.BytesIO(wheel_raw)) as z:
   names=z.namelist();need(len(names)==len(set(names)),'duplicate wheel member')
   need(all(not n.startswith('/') and '..' not in Path(n).parts and '\\' not in n for n in names),'unsafe wheel member')
   natives=[n for n in names if n.endswith(('.so','.pyd','.dylib'))];need(len(natives)==1,'native module inventory differs')
   native=z.read(natives[0]);shim=z.read('weft/__init__.py')
   if target=='aarch64-apple-darwin':need(native[:4]==b'\xcf\xfa\xed\xfe' and int.from_bytes(native[4:8],'little')==machine,'native Mach-O architecture differs')
   else:need(native[:4]==b'\x7fELF' and native[4:6]==b'\x02\x01' and int.from_bytes(native[18:20],'little')==machine,'native ELF architecture differs')
   actual=dict(wheelSha256=digest(wheel_raw),nativeModuleSha256=digest(native),shimSha256=digest(shim))
   need(report['pinsBeforeAfter']==actual,'SDK actual wheel/module/shim hashes differ')
   need(e['nativeModuleSha256']==actual['nativeModuleSha256'] and e['shimSha256']==actual['shimSha256'],'current envelope module/shim differs')
  out.append(dict(target=target,wheel=name,**actual,SDKControls=42,producerRun=e['producer']['runId'],controllerRun=e['controller']['runId']))
 need(seen==set(TARGETS),'required current target roster differs')
 need(len(linux_runs)==len(mac_runs)==1 and controller_runs==(linux_runs|mac_runs),'producer/controller run cohort is split')
 need(not (linux_runs & mac_runs),'Linux and Mac producer roles collapsed')
 return dict(profile=PROFILE,status='checked',sourceCommit=source,sourceTree=tree,wheels=sorted(out,key=lambda x:x['target']),nativeAdmission=False,publicationAdmission=False)
