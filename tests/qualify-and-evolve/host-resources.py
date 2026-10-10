"""@covers US-006-AC4: raw resource refusals through CLI and native Python."""
import copy,hashlib,json,os,pathlib,subprocess,sys
import sys
if sys.flags.optimize:raise RuntimeError('Qualification requires nonoptimized Python')
import weft
import weft.weft as extension
ROOT=pathlib.Path(__file__).resolve().parents[2]
base=json.loads((ROOT/'tests/compile/fixtures/cases.json').read_text())[0]['request']
sql=copy.deepcopy(base);sql['sql']=' '*65537
binding=copy.deepcopy(base);binding['target']['bindingJson']=' '*(4*1024*1024+1)
cases=[('duplicate','{"outer":{"x":1,"x":2}}','WFT-JSON-DUPLICATE'),('truncated','{"x":[1,','WFT-INPUT'),('nodes-over','['+','.join(['0']*100000)+']','WFT-LIMIT'),('request-at',' '*(16*1024*1024),'WFT-INPUT'),('request-over',' '*(16*1024*1024+1),'WFT-LIMIT'),('sql-over',json.dumps(sql),'WFT-LIMIT'),('binding-over',json.dumps(binding),'WFT-LIMIT')]
binary=pathlib.Path(os.environ['WEFT_RESOURCE_BINARY']);sha=hashlib.sha256(binary.read_bytes()).hexdigest()
reports=[]
for name,raw,code in cases:
 result=subprocess.run([str(binary)],input=raw,text=True,capture_output=True,timeout=30);assert result.returncode==0,result.stderr
 response=json.loads(result.stdout)
 assert response['status']=='blocked' and response['diagnostics'][0]['code']==code,(name,response)
 assert not set(response)&{'sql','parameters','logicalPlan','columns','obligations'}
 reports.append({'id':name,'raw':raw,'expectedCode':code,'response':result.stdout.strip()})
assert hashlib.sha256(binary.read_bytes()).hexdigest()==sha
os.environ['PATH']=''
def forbidden(*a,**k):raise AssertionError('Subprocess during native compile')
subprocess.Popen=subprocess.run=subprocess.check_output=forbidden
for report in reports:assert weft.compile_json(report['raw'])==report['response'],report['id']
pathlib.Path(os.environ['WEFT_RESOURCE_CASES']).write_text(json.dumps(reports))
summary={'status':'passed','cases':len(reports),'binarySha256':sha,'extensionSha256':hashlib.sha256(pathlib.Path(extension.__file__).read_bytes()).hexdigest(),'python':sys.version,'casesAndResponsesSha256':hashlib.sha256(pathlib.Path(os.environ['WEFT_RESOURCE_CASES']).read_bytes()).hexdigest(),'subprocessDisabled':True,'scope':'Raw malicious/resource requests; early boundary checks shared across backend feature builds. No native database execution.'}
pathlib.Path(os.environ['WEFT_RESOURCE_SUMMARY']).write_text(json.dumps(summary,indent=2)+'\n');print(json.dumps(summary))
