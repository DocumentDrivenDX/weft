import json,hashlib,sys,copy
sys.path.insert(0,'/private/tmp/subject-consumer-python')
import weft
request=json.load(open('/private/tmp/truss-security-main-integration/packages/python/tests/fixtures/security-mapping-request.json'))
selection={'sourceSha256':request['modules'][0]['pin']['sha256'],'identity':{'module':'m','element':'Staff','key':'pk'}}
cases=[]
def add(name,r,s,status):cases.append({'id':name,'request':r if type(r)is str else json.dumps(r),'selection':s if type(s)is str else json.dumps(s),'status':status})
add('original-subject',request,selection,'inspected')
for key,value in [('module','foreign'),('element','Resource'),('key','foreign')]:
 s=copy.deepcopy(selection);s['identity'][key]=value;add('foreign-'+key,request,s,'refused')
s=copy.deepcopy(selection);s['sourceSha256']='0'*64;add('foreign-source',request,s,'refused')
s=copy.deepcopy(selection);s['identity']['extra']=True;add('nested-extra',request,s,'refused')
s=copy.deepcopy(selection);s['extra']=True;add('extra',request,s,'refused')
add('duplicate-selection',request,json.dumps(selection)[:-1]+',"sourceSha256":"0"}','refused')
for key in ['policyJson','ontologyJson']:
 r=copy.deepcopy(request);r[key]+=' ';add('stale-'+key,r,selection,'refused')
r=copy.deepcopy(request);r['modules'][0]['documentJson']+=' ';add('stale-model',r,selection,'refused')
r=copy.deepcopy(request);r['sql']='SELECT missing FROM Resource';add('invalid-sql',r,selection,'refused')
add('selection-size',request,' '*16385,'refused')
add('malformed-request','{',selection,'refused')
for c in cases:
 raw=weft.inspect_security_mapping_subject_key_json(c['request'],c['selection']);v=json.loads(raw)
 assert v['interfaceVersion']=='weft-security-subject-selection/0.1.0' and v['status']==c['status'] and v['securityReady']is False,c['id']
 assert set(v)==({'interfaceVersion','status','securityReady'} if c['status']=='inspected' else {'interfaceVersion','status','securityReady','diagnostics'}),c['id']
 if c['status']=='refused':assert v['diagnostics'],c['id']
 c['response']=raw
out={'status':'passed','weftVersion':weft.__version__,'cases':cases,'moduleSha256':hashlib.sha256(open(weft.__file__,'rb').read()).hexdigest(),'scope':'Actual Python extension pure source correspondence; no native database authentication/admission'}
json.dump(out,open('/private/tmp/subject-consumer-python.json','w'),indent=2)
print('Passed',len(cases),'actual Python extension controls')
