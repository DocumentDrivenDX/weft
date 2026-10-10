"""Fresh public security refusals across CLI/native ABI; no native activation."""
import copy,hashlib,json,os,pathlib,subprocess
import weft
import weft.weft as extension
ROOT=pathlib.Path(__file__).resolve().parents[2]
def text(v):return json.dumps(v,separators=(',',':'),ensure_ascii=False)
def main():
 out=pathlib.Path(os.environ['WEFT_FRESH_HOST_OUT']);binary=pathlib.Path(os.environ['WEFT_RESOURCE_BINARY'])
 cases=json.loads((out/'resource-cases.json').read_bytes())
 fixture=json.loads((ROOT/'crates/weft-core/tests/security-source-fixture.json').read_bytes());base=next(c['request'] for c in json.loads((ROOT/'docs/helix/03-test/fixtures/cases.json').read_bytes()) if c['expected']['status']=='compiled');request=copy.deepcopy(base)
 document=fixture['resolution']['documents'][0];rawdoc=text(document['document']);request.update(interfaceVersion='weft-compile/0.5.0',dialect='weft-sql/0.2.0',modules=[{'documentJson':rawdoc,'pin':{'documentId':document['document']['id'],'revision':document['revision'],'umfVersion':'0.8.0','sha256':hashlib.sha256(rawdoc.encode()).hexdigest()},'selectedModuleIds':['m']}],security={'version':'umf.security/0.1.0','policyJson':text(fixture['policy']),'ontologyJson':text(fixture['resolution']['ontology'])})
 vectors=[('security-unsupported',request,'WFT-SECURITY-UNSUPPORTED')]
 r=copy.deepcopy(request);r.pop('security');vectors.append(('security-missing',r,'WFT-INPUT'))
 r=copy.deepcopy(request);r['modules'][0]['pin']['sha256']='0'*64;vectors.append(('security-pin',r,'WFT-PIN'))
 r=copy.deepcopy(request);p=copy.deepcopy(fixture['policy']);p['future']=True;r['security']['policyJson']=text(p);vectors.append(('security-unknown',r,'WFT-SECURITY-SOURCE'))
 r=copy.deepcopy(request);p=copy.deepcopy(fixture['resolution']['ontology']);p['documents'][0]['revision']='other';r['security']['ontologyJson']=text(p);vectors.append(('security-stale',r,'WFT-SECURITY-PIN'))
 r=copy.deepcopy(request);r['interfaceVersion']='weft-compile/0.2.0';vectors.append(('security-downgrade',r,'WFT-INPUT'))
 for name,req,code in vectors:
  raw=text(req);run=subprocess.run([str(binary)],input=raw,text=True,capture_output=True,timeout=30)
  if run.returncode or run.stderr:raise RuntimeError()
  response=run.stdout.strip();parsed=json.loads(response)
  if parsed['status']!='blocked' or parsed['diagnostics'][0]['code']!=code or set(parsed)&{'sql','parameters','logicalPlan','columns','obligations'}:raise RuntimeError()
  cases.append({'id':name,'raw':raw,'expectedCode':code,'response':response})
 os.environ['PATH']=''
 def forbidden(*args,**kwargs):raise RuntimeError()
 subprocess.Popen=subprocess.run=subprocess.check_output=forbidden
 for case in cases:
  if weft.compile_json(case['raw'])!=case['response']:raise RuntimeError()
 (out/'resource-security-cases.json').write_text(json.dumps(cases)+'\n')
 summary={'status':'passed','resourceCases':7,'securityCases':len(vectors),'cases':len(cases),'byteParity':True,'subprocessDisabled':True,'extensionSha256':hashlib.sha256(pathlib.Path(extension.__file__).read_bytes()).hexdigest(),'scope':'Fresh exact whole-response ABI parity for raw resource controls and blocked security0.5 admission/refusals. No native security enforcement or database execution.'}
 (out/'security-summary.json').write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
if __name__=='__main__':main()
