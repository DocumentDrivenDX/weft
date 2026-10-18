"""Compile the exact 1,000-type owner-captured request; test closed capacity bounds."""
import copy,gzip,hashlib,json,time
from pathlib import Path
import weft
request=json.loads(gzip.decompress(Path(__file__).with_name('genuine-thousand-type-request.json.gz').read_bytes()))

def compile_request(value):
    return json.loads(weft.compile_truss_runtime_json(json.dumps(value,separators=(',',':'))))

def refused(value,code):
    result=compile_request(value)
    assert result['status']=='blocked' and result['diagnostics'][0]['code']==code,result
    assert 'sql' not in result

def binding(value,raw):
    value['target']['bindingJson']=raw
    value['target']['bindingSha256']=hashlib.sha256(raw.encode()).hexdigest()

def document(value,source):
    raw=json.dumps(source,separators=(',',':'))
    digest=hashlib.sha256(raw.encode()).hexdigest()
    value['modules'][0]['documentJson']=raw
    value['modules'][0]['pin']['sha256']=digest
    native=json.loads(value['target']['bindingJson']);native['documents'][0]['sha256']=digest
    binding(value,json.dumps(native,separators=(',',':')))
    return raw

started=time.monotonic();result=compile_request(request);elapsed=time.monotonic()-started
assert result['status']=='compiled',result
assert hashlib.sha256(json.dumps(result,sort_keys=True,separators=(',',':'),ensure_ascii=False).encode()).hexdigest()=='21be0763aefc88ae2d78b083baca275fb7e6a9e57ce712488eb884726d5583f5'
assert result['bindingSha256']==request['target']['bindingSha256']
assert result['modelPins']==[item['pin'] for item in request['modules']]
assert any(p.get('value')==request['target']['bindingJson'] for p in result['parameters'])
# Existing registrations keep their old parse budget and their refusal.
old=copy.deepcopy(request);old['target']['backendVersion']='0.4.40';old['target']['targetProfile']='truss-runtime-pg18.6/0.4'
refused(old,'WFT-LIMIT')
# Larger bounded inputs must still reject duplicate JSON members before lowering.
duplicate=copy.deepcopy(request)
binding(duplicate,request['target']['bindingJson'][:-1]+',"schema":"truss"}')
refused(duplicate,'WFT-JSON-DUPLICATE')
# A compact model can reach the value limit while remaining inside four MiB.
over=copy.deepcopy(request);source=json.loads(over['modules'][0]['documentJson'])
source['extensions']['capacity-probe']=[0]*821_000
raw=document(over,source);assert len(raw.encode())<=4*1024*1024
refused(over,'WFT-LIMIT')
# Four MiB remains a byte bound, including UTF-8 whitespace.
over=copy.deepcopy(request);binding(over,request['target']['bindingJson']+' '*(4*1024*1024+1-len(request['target']['bindingJson'].encode())))
refused(over,'WFT-LIMIT')
print(json.dumps({'status':'passed','compiler':weft.__version__,'records':1000,'fieldsPerRecord':12,'compileSeconds':elapsed,'controls':5}))
