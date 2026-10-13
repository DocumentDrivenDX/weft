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
  assert actual==expected and len(h['uses'])==6
 observations.append({'id':identity,'status':status,'responseSha256':hashlib.sha256(native.encode()).hexdigest(),'correspondence':'byte-identical'})
 if identity=='five-operators':(output.parent/'handoff.json').write_text(native+'\n')
receipt={'status':'passed','observations':observations,'python':sys.version.split()[0],
 'binaries':{str(p):hashlib.sha256(p.read_bytes()).hexdigest() for p in [module_dir/'weft.so',cli]},
 'scope':'Imported original native cdylib and original CLI share Rust core mapping preparation. Opaque unqualified fixture mapping. No installed wheel, native SQL/role/effects, profile issuer, execution credential or B08 acceptance.'}
output.write_text(json.dumps(receipt,indent=2)+'\n');print(json.dumps({'passed':len(observations),'scope':'native Python/CLI transport only'}))
