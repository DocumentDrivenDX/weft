# SPDX-License-Identifier: MIT OR Apache-2.0
"""Original imported native Python/CLI correspondence; no SQL effects or issuer.
@covers US-009-AC1
@covers US-009-AC2
"""
import copy,hashlib,json,subprocess,sys
from pathlib import Path
module_dir,cli,request_path,output=map(Path,sys.argv[1:])
sys.path.insert(0,str(module_dir));import weft
assert Path(weft.__file__).resolve()==(module_dir/'weft.so').resolve(), 'Original imported extension path required'
request=json.loads(request_path.read_text());cases=[('five-operators',copy.deepcopy(request),'prepared')]
for key in ['bindingJson','policyJson','ontologyJson','backendId','backendVersion','targetProfile']:
 changed=copy.deepcopy(request);changed[key]+=' ';cases.append((key+'-drift',changed,'refused'))
for key in ['originalAction','handoff']:
 changed=copy.deepcopy(request);changed[key]='caller-forged';cases.append((key+'-forgery',changed,'refused'))
changed=copy.deepcopy(request);changed['parameters']={'p':{'family':'integer','value':100}};cases.append(('numeric-carrier',changed,'refused'))
changed=copy.deepcopy(request);profile=json.loads(changed['queryProfileJson']);profile['bindings']=[];changed['queryProfileJson']=json.dumps(profile);cases.append(('missing-bindings',changed,'refused'))
changed=copy.deepcopy(request);changed['queryProfileJson']+=' ';cases.append(('profile-bytes-not-authentication',changed,'prepared'))
changed=copy.deepcopy(request);changed['parameters']={'p':{'family':'string','value':'x'*65536}};changed['sql']='SELECT r.resourceId FROM Resource r WHERE '+' AND '.join(['r.resourceId=:p']*300);cases.append(('repeated-parameter-budget',changed,'refused'))
changed=copy.deepcopy(request);changed['readProfile']={'version':'weft-truss-entity-read/0.1.0','subset':'complete-entity'};changed['sql']='SELECT r.* FROM Resource r';cases.append(('complete-entity-not-admitted',changed,'refused'))
related=copy.deepcopy(request);document=json.loads(related['modules'][0]['documentJson'])
document['modules'][0]['relationships']=[{'id':'resource-peer','name':'peer','source':[{'module':'m','element':'Resource'}],'target':[{'module':'m','element':'Resource','key':'pk'}],'sourceMultiplicity':{'min':0,'max':'*'},'targetMultiplicity':{'min':0,'max':'*'},'targetLifecycle':'independent','directed':True}]
model=json.dumps(document,separators=(',',':'),ensure_ascii=False);related['modules'][0]['documentJson']=model;related['modules'][0]['pin']['sha256']=hashlib.sha256(model.encode()).hexdigest()
profile=json.loads(related['queryProfileJson']);profile['modelPins']=[related['modules'][0]['pin']];related['queryProfileJson']=json.dumps(profile,separators=(',',':'),ensure_ascii=False)
related['sql']='SELECT r.resourceId FROM Resource r';related['parameters']={};cases.append(('relationship-model-baseline',copy.deepcopy(related),'prepared'))
related['sql']='SELECT r.resourceId FROM Resource r WHERE r.peer=:p';related['parameters']={'p':{'family':'string','value':'r1'}};cases.append(('relationship-shorthand-not-admitted',related,'refused'))
observations=[]
for identity,value,status in cases:
 raw=json.dumps(value,separators=(',',':'),ensure_ascii=False)
 native=weft.prepare_security_mapping_json(raw)
 process=subprocess.run([str(cli)],input=raw,text=True,capture_output=True,timeout=20);assert process.returncode==0,process.stderr
 assert native==process.stdout.rstrip('\n'),identity
 response=json.loads(native);assert response['status']==status and response['securityReady'] is False,(identity,response)
 if status=='refused':assert 'handoff' not in response
 else:
  h=response['handoff'];assert h['sqlSha256']==hashlib.sha256(value['sql'].encode()).hexdigest()
  assert h['profile']['sha256']==hashlib.sha256(value['queryProfileJson'].encode()).hexdigest()
  actual={(u['scan'],u['operator'],u['originalAction']) for u in h['uses']}
  expected={('s0',op,'query-original') for op in ['predicate','order','group','join','aggregate']}|{('s1','join','query-original')}
  if identity=='relationship-model-baseline':assert actual==set()
  else:assert actual==expected and len(h['uses'])==6
 observations.append({'id':identity,'status':status,'responseSha256':hashlib.sha256(native.encode()).hexdigest(),'correspondence':'byte-identical'})
 if identity=='complete-entity-not-admitted':assert response['diagnostics'][0]['code']=='WFT-SECURITY-MAPPING-TRANSPORT'
 if identity=='relationship-shorthand-not-admitted':assert response['diagnostics'][0]['code']=='WFT-SECURITY-QUERY-UNSUPPORTED'
 if identity=='five-operators':(output.parent/'handoff.json').write_text(native+'\n')
receipt={'status':'passed','observations':observations,'python':sys.version.split()[0],
 'binaries':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [module_dir/'weft.so',cli]},
 'scope':'Imported original native cdylib and original CLI share Rust core mapping preparation. Opaque unqualified fixture mapping. No installed wheel, native SQL/role/effects, profile issuer, execution credential or B08 acceptance.'}
output.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'passed':len(observations),'scope':'native Python/CLI transport only'}))
